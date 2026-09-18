#![cfg(feature = "streaming")]

//! Async WebSocket client for the market and account streams (feature
//! `streaming`).
//!
//! Subscribe, then consume frames either by pulling
//! ([`StreamHandle::next`]) or by handing a closure to
//! [`StreamHandle::run`]. Both auto-reconnect with backoff, replay the
//! subscription set on every reconnect, and act on server `error` frames
//! that carry a `resubscribe` / `reconnect` action. Public channels need
//! no auth; `account_events` needs the bearer passed to the builder.

mod connection;
mod settings;

pub use settings::{
	Reconnect, ReconnectField, StreamClientError, StreamEndpoint, StreamPhase, StreamUrlError,
};

use core::future::Future;

use std::borrow::Cow;

use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
	client::DEFAULT_BASE_URL,
	error::Result,
	frames::{Channel, Frame},
};

/// Per-connection subscription cap the server enforces.
pub const MAX_SUBSCRIPTIONS_PER_CONNECTION: usize = 64;

const FRAME_BUFFER_CAPACITY: usize = 1024;
const CONTROL_BUFFER_CAPACITY: usize = 64;

/// A configured, not-yet-connected stream client.
#[derive(Clone)]
pub struct AgaraStreamClient {
	token: Option<Cow<'static, str>>,
	base_url: reqwest::Url,
	reconnect: Reconnect,
	subscriptions: Vec<Channel>,
}

/// A live stream. Pull frames with [`StreamHandle::next`] or drive a
/// closure with [`StreamHandle::run`].
pub struct StreamHandle {
	frames: mpsc::Receiver<Frame>,
	market_ctrl: Option<mpsc::Sender<Control>>,
	account_ctrl: Option<mpsc::Sender<Control>>,
	tasks: Vec<JoinHandle<()>>,
	subscriptions: Vec<Channel>,
}

#[derive(Clone)]
enum Control {
	Subscribe(Vec<Channel>),
	Unsubscribe(Vec<Channel>),
	Ping,
	List,
}

fn reserve<'a>(
	sender: &'a Option<mpsc::Sender<Control>>,
	control: Option<&Control>,
	endpoint: StreamEndpoint,
) -> Result<Option<mpsc::Permit<'a, Control>>> {
	if control.is_none() {
		return Ok(None);
	}
	let sender = sender.as_ref().ok_or(StreamClientError::EndpointNotOpen { endpoint })?;

	sender.try_reserve().map(Some).map_err(|error| match error {
		mpsc::error::TrySendError::Full(_) => {
			StreamClientError::ControlBackpressure { endpoint }.into()
		},
		mpsc::error::TrySendError::Closed(_) => {
			StreamClientError::EndpointClosed { endpoint }.into()
		},
	})
}

fn validate_subscription_count(channels: &[Channel]) -> Result<()> {
	for endpoint in [StreamEndpoint::Market, StreamEndpoint::Account] {
		if channels
			.iter()
			.filter(|channel| channel.is_account() == (endpoint == StreamEndpoint::Account))
			.count() > MAX_SUBSCRIPTIONS_PER_CONNECTION
		{
			return Err(StreamClientError::SubscriptionLimit {
				endpoint,
				limit: MAX_SUBSCRIPTIONS_PER_CONNECTION,
			}
			.into());
		}
	}

	Ok(())
}

#[bon::bon]
impl AgaraStreamClient {
	/// Build a validated client; credentials are required only for account subscriptions.
	/// Invalid URLs or empty/whitespace-bearing credentials fail before any task is spawned.
	#[builder]
	pub fn new(
		token: Option<Cow<'static, str>>,
		#[builder(default = Cow::Borrowed(DEFAULT_BASE_URL))] base_url: Cow<'static, str>,
		#[builder(default)] reconnect: Reconnect,
	) -> Result<Self> {
		let url = reqwest::Url::parse(&base_url)
			.map_err(|error| StreamClientError::invalid_base_url(StreamUrlError::Parse(error)))?;
		let url_error = if !core::matches!(url.scheme(), "http" | "https" | "ws" | "wss") {
			Some(StreamUrlError::UnsupportedScheme)
		} else if url.host_str().is_none() {
			Some(StreamUrlError::MissingHost)
		} else if !url.username().is_empty() || url.password().is_some() {
			Some(StreamUrlError::Credentials)
		} else if url.query().is_some() {
			Some(StreamUrlError::Query)
		} else if url.fragment().is_some() {
			Some(StreamUrlError::Fragment)
		} else {
			None
		};
		if let Some(reason) = url_error {
			return Err(StreamClientError::invalid_base_url(reason).into());
		}

		if token.as_ref().is_some_and(|token| {
			token.is_empty()
				|| token.chars().any(char::is_whitespace)
				|| token.chars().any(char::is_control)
		}) {
			return Err(StreamClientError::InvalidToken.into());
		}
		reconnect.validate()?;

		Ok(Self { token, base_url: url, reconnect, subscriptions: Vec::new() })
	}

	/// Add valid channels atomically. Each endpoint accepts at most 64 distinct subscriptions.
	pub fn subscribe(&mut self, channels: impl IntoIterator<Item = Channel>) -> Result<()> {
		let mut candidate = self.subscriptions.clone();
		for channel in channels {
			if channel.is_account() && self.token.is_none() {
				return Err(StreamClientError::MissingToken.into());
			}
			if !candidate.contains(&channel) {
				candidate.push(channel);
				validate_subscription_count(&candidate)?;
			}
		}
		validate_subscription_count(&candidate)?;
		self.subscriptions = candidate;

		Ok(())
	}

	/// Spawn only the endpoints needed by the initial subscription set.
	/// Local asynchronous failures arrive as `Frame::ClientError`, never invented server codes.
	pub async fn connect(self) -> Result<StreamHandle> {
		self.reconnect.validate()?;
		if self.subscriptions.is_empty() {
			return Err(StreamClientError::NoSubscriptions.into());
		}
		validate_subscription_count(&self.subscriptions)?;
		let (frames_tx, frames_rx) = mpsc::channel(FRAME_BUFFER_CAPACITY);
		let mut handle = StreamHandle {
			frames: frames_rx,
			market_ctrl: None,
			account_ctrl: None,
			tasks: Vec::new(),
			subscriptions: self.subscriptions.clone(),
		};
		let (account, market): (Vec<Channel>, Vec<Channel>) =
			self.subscriptions.iter().cloned().partition(Channel::is_account);
		if !market.is_empty() {
			let (control, task) =
				self.spawn_endpoint(StreamEndpoint::Market, market, frames_tx.clone())?;
			handle.market_ctrl = Some(control);
			handle.tasks.push(task);
		}
		if !account.is_empty() {
			let (control, task) =
				self.spawn_endpoint(StreamEndpoint::Account, account, frames_tx)?;
			handle.account_ctrl = Some(control);
			handle.tasks.push(task);
		}

		Ok(handle)
	}

	fn spawn_endpoint(
		&self,
		endpoint: StreamEndpoint,
		subscriptions: Vec<Channel>,
		frames: mpsc::Sender<Frame>,
	) -> Result<(mpsc::Sender<Control>, JoinHandle<()>)> {
		let (control_tx, control_rx) = mpsc::channel(CONTROL_BUFFER_CAPACITY);
		let mut url = self.base_url.clone();
		let scheme = match url.scheme() {
			"https" | "wss" => "wss",
			_ => "ws",
		};
		url.set_scheme(scheme)
			.map_err(|_| StreamClientError::invalid_base_url(StreamUrlError::SchemeConversion))?;
		url.set_path(&std::format!(
			"{}{}",
			url.path().trim_end_matches('/'),
			endpoint.path()
		));
		let config = self.reconnect.clone();
		let token = self.token.clone();
		let task = tokio::spawn(connection::run(
			endpoint,
			url,
			token,
			subscriptions,
			config,
			frames,
			control_rx,
		));

		Ok((control_tx, task))
	}
}

impl StreamHandle {
	/// Await a server frame or typed local failure; `None` means every endpoint stopped.
	pub async fn next(&mut self) -> Option<Frame> {
		self.frames.recv().await
	}

	/// Dispatch server and local-error frames until all endpoint tasks stop.
	pub async fn run<F, Fut>(mut self, mut handler: F)
	where
		F: FnMut(Frame) -> Fut,
		Fut: Future<Output = ()>,
	{
		while let Some(frame) = self.frames.recv().await {
			handler(frame).await;
		}
	}

	/// Queue additions atomically across endpoints; `Subscribed` frames acknowledge server acceptance.
	/// Absent, closed or full endpoint control queues return an error without partial dispatch.
	pub fn subscribe(&mut self, channels: impl IntoIterator<Item = Channel>) -> Result<()> {
		let mut candidate = self.subscriptions.clone();
		let mut additions = Vec::new();
		for channel in channels {
			if !candidate.contains(&channel) {
				candidate.push(channel.clone());
				validate_subscription_count(&candidate)?;
				additions.push(channel);
			}
		}
		validate_subscription_count(&candidate)?;
		let (account, market): (Vec<_>, Vec<_>) =
			additions.into_iter().partition(Channel::is_account);
		self.dispatch(
			Some(Control::Subscribe(market)),
			Some(Control::Subscribe(account)),
		)?;
		self.subscriptions = candidate;

		Ok(())
	}

	/// Queue removals atomically. Earlier frames may still arrive until the server acknowledges them.
	pub fn unsubscribe(&mut self, channels: impl IntoIterator<Item = Channel>) -> Result<()> {
		let mut unique = Vec::new();
		for channel in channels {
			if !unique.contains(&channel) {
				unique.push(channel);
				validate_subscription_count(&unique)?;
			}
		}
		let channels = unique;
		let (account, market): (Vec<_>, Vec<_>) =
			channels.iter().cloned().partition(Channel::is_account);
		self.dispatch(
			Some(Control::Unsubscribe(market)),
			Some(Control::Unsubscribe(account)),
		)?;
		self.subscriptions.retain(|channel| !channels.contains(channel));

		Ok(())
	}

	/// Send an application ping to each active endpoint, reporting closed or full queues.
	pub fn ping(&self) -> Result<()> {
		self.broadcast(Control::Ping)
	}

	/// Request current server subscriptions from each active endpoint.
	pub fn list_subscriptions(&self) -> Result<()> {
		self.broadcast(Control::List)
	}

	/// Cancel all endpoint tasks, including blocked I/O, backoff and a full output queue.
	pub async fn close(mut self) {
		for task in self.tasks.drain(..) {
			task.abort();
			let _ = task.await;
		}
	}

	fn broadcast(&self, control: Control) -> Result<()> {
		self.dispatch(
			self.market_ctrl.as_ref().map(|_| control.clone()),
			self.account_ctrl.as_ref().map(|_| control),
		)
	}

	fn dispatch(&self, market: Option<Control>, account: Option<Control>) -> Result<()> {
		let market = market.filter(Control::has_payload);
		let account = account.filter(Control::has_payload);
		let market_permit = reserve(&self.market_ctrl, market.as_ref(), StreamEndpoint::Market)?;
		let account_permit = reserve(
			&self.account_ctrl,
			account.as_ref(),
			StreamEndpoint::Account,
		)?;
		if let (Some(permit), Some(control)) = (market_permit, market) {
			permit.send(control);
		}
		if let (Some(permit), Some(control)) = (account_permit, account) {
			permit.send(control);
		}

		Ok(())
	}
}

impl Control {
	fn has_payload(&self) -> bool {
		match self {
			Self::Subscribe(channels) | Self::Unsubscribe(channels) => !channels.is_empty(),
			_ => true,
		}
	}
}

impl Drop for StreamHandle {
	fn drop(&mut self) {
		for task in &self.tasks {
			task.abort();
		}
	}
}
