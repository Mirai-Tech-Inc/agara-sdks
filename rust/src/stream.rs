//! Async WebSocket client for the market and account streams (feature
//! `streaming`).
//!
//! Subscribe, then consume frames either by pulling
//! ([`StreamHandle::next`]) or by handing a closure to
//! [`StreamHandle::run`]. Both auto-reconnect with backoff, replay the
//! subscription set on every reconnect, and act on server `error` frames
//! that carry a `resubscribe` / `reconnect` action. Public channels need
//! no auth; `account_events` needs the bearer passed to the builder.

use core::future::Future;
use core::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;

use crate::client::DEFAULT_BASE_URL;
use crate::error::{AgaraError, Result};
use crate::frames::{Channel, Frame, StreamError, decode_frame};
use crate::retry::{
	DEFAULT_INITIAL_BACKOFF, DEFAULT_JITTER, DEFAULT_MAX_BACKOFF, jittered_backoff,
};

const MARKET_PATH: &str = "/trade/v1/market-stream";
const ACCOUNT_PATH: &str = "/trade/v1/account-stream";

/// Per-connection subscription cap the server enforces.
pub const MAX_SUBSCRIPTIONS_PER_CONNECTION: usize = 64;

const PING_INTERVAL: Duration = Duration::from_secs(25);
const SILENCE_TIMEOUT: Duration = Duration::from_secs(35);

/// A connection must stay up at least this long to count as healthy and
/// clear the reconnect backoff; shorter sessions keep backing off.
const HEALTHY_CONNECTION: Duration = Duration::from_secs(30);

/// Reconnect backoff. `max_attempts = None` retries forever; `Some(0)`
/// disables reconnect (one shot).
#[derive(Clone, Debug)]
pub struct Reconnect {
	pub initial_delay: Duration,
	pub max_delay: Duration,
	pub jitter: f64,
	pub max_attempts: Option<u32>,
}

impl Default for Reconnect {
	fn default() -> Self {
		Self {
			initial_delay: DEFAULT_INITIAL_BACKOFF,
			max_delay: DEFAULT_MAX_BACKOFF,
			jitter: DEFAULT_JITTER,
			max_attempts: None,
		}
	}
}

/// A configured, not-yet-connected stream client.
#[derive(Clone, Debug)]
pub struct AgaraStreamClient {
	token: Option<String>,
	base_url: String,
	reconnect: Reconnect,
	subscriptions: Vec<Channel>,
}

#[bon::bon]
impl AgaraStreamClient {
	/// Build a stream client. Pass `token` to use `account_events`.
	#[builder]
	pub fn new(
		token: Option<String>,
		#[builder(default = DEFAULT_BASE_URL.to_owned())] base_url: String,
		#[builder(default)] reconnect: Reconnect,
	) -> Self {
		Self { token, base_url, reconnect, subscriptions: Vec::new() }
	}

	/// Add channels to the subscription set before connecting.
	pub fn subscribe(&mut self, channels: impl IntoIterator<Item = Channel>) -> Result<()> {
		for ch in channels {
			if self.subscriptions.contains(&ch) {
				continue;
			}

			if ch.is_account() && self.token.is_none() {
				return Err(AgaraError::Validation(
					"account_events requires a bearer token".to_owned(),
				));
			}

			if self.subscriptions.len() >= MAX_SUBSCRIPTIONS_PER_CONNECTION {
				return Err(AgaraError::Validation(format!(
					"subscription set exceeds MAX_SUBSCRIPTIONS_PER_CONNECTION ({MAX_SUBSCRIPTIONS_PER_CONNECTION})"
				)));
			}

			self.subscriptions.push(ch);
		}

		Ok(())
	}

	/// Open the needed endpoint(s) and start receiving. `market` opens
	/// iff a public channel is subscribed; `account` iff `account_events`
	/// is.
	pub async fn connect(self) -> Result<StreamHandle> {
		if self.subscriptions.is_empty() {
			return Err(AgaraError::Validation(
				"connect() needs at least one subscription".to_owned(),
			));
		}

		let (frames_tx, frames_rx) = mpsc::unbounded_channel();
		let mut handle = StreamHandle {
			frames: frames_rx,
			market_ctrl: None,
			account_ctrl: None,
			tasks: Vec::new(),
		};

		let market: Vec<Channel> =
			self.subscriptions.iter().filter(|c| !c.is_account()).cloned().collect();
		let account: Vec<Channel> =
			self.subscriptions.iter().filter(|c| c.is_account()).cloned().collect();

		if !market.is_empty() {
			let (ctrl, task) = self.spawn_endpoint(MARKET_PATH, market, frames_tx.clone());
			handle.market_ctrl = Some(ctrl);
			handle.tasks.push(task);
		}

		if !account.is_empty() {
			let (ctrl, task) = self.spawn_endpoint(ACCOUNT_PATH, account, frames_tx);
			handle.account_ctrl = Some(ctrl);
			handle.tasks.push(task);
		}

		Ok(handle)
	}

	fn spawn_endpoint(
		&self,
		path: &str,
		subs: Vec<Channel>,
		frames: mpsc::UnboundedSender<Frame>,
	) -> (mpsc::UnboundedSender<Control>, JoinHandle<()>) {
		let (ctrl_tx, ctrl_rx) = mpsc::unbounded_channel();
		let url = ws_url(&self.base_url, path);
		let cfg = self.reconnect.clone();
		let token = self.token.clone();
		let task = tokio::spawn(async move {
			run_endpoint(url, token, subs, cfg, frames, ctrl_rx).await;
		});
		(ctrl_tx, task)
	}
}

/// Control ops sent from a [`StreamHandle`] to a running endpoint task.
#[derive(Clone)]
enum Control {
	Subscribe(Vec<Channel>),
	Unsubscribe(Vec<Channel>),
	Ping,
	List,
	Stop,
}

/// A live stream. Pull frames with [`StreamHandle::next`] or drive a
/// closure with [`StreamHandle::run`].
pub struct StreamHandle {
	frames: mpsc::UnboundedReceiver<Frame>,
	market_ctrl: Option<mpsc::UnboundedSender<Control>>,
	account_ctrl: Option<mpsc::UnboundedSender<Control>>,
	tasks: Vec<JoinHandle<()>>,
}

impl StreamHandle {
	/// Await the next frame, or `None` once every endpoint has stopped.
	pub async fn next(&mut self) -> Option<Frame> {
		self.frames.recv().await
	}

	/// Dispatch every frame to `handler` until the stream ends.
	pub async fn run<F, Fut>(mut self, mut handler: F)
	where
		F: FnMut(Frame) -> Fut,
		Fut: Future<Output = ()>,
	{
		while let Some(frame) = self.frames.recv().await {
			handler(frame).await;
		}
	}

	/// Add channels to a live stream (routed to the right endpoint).
	pub fn subscribe(&self, channels: impl IntoIterator<Item = Channel>) {
		let (account, market): (Vec<Channel>, Vec<Channel>) =
			channels.into_iter().partition(Channel::is_account);
		self.send_to(&self.market_ctrl, Control::Subscribe(market));
		self.send_to(&self.account_ctrl, Control::Subscribe(account));
	}

	/// Remove channels from a live stream.
	pub fn unsubscribe(&self, channels: impl IntoIterator<Item = Channel>) {
		let (account, market): (Vec<Channel>, Vec<Channel>) =
			channels.into_iter().partition(Channel::is_account);
		self.send_to(&self.market_ctrl, Control::Unsubscribe(market));
		self.send_to(&self.account_ctrl, Control::Unsubscribe(account));
	}

	/// Send an application `ping` on every open endpoint.
	pub fn ping(&self) {
		self.broadcast(Control::Ping);
	}

	/// Ask every endpoint for its `subscription_list`.
	pub fn list_subscriptions(&self) {
		self.broadcast(Control::List);
	}

	/// Stop all endpoints and wait for their tasks to finish.
	pub async fn close(mut self) {
		self.broadcast(Control::Stop);
		for task in self.tasks.drain(..) {
			let _ = task.await;
		}
	}

	fn broadcast(&self, msg: Control) {
		self.send_to(&self.market_ctrl, msg.clone());
		self.send_to(&self.account_ctrl, msg);
	}

	fn send_to(&self, ctrl: &Option<mpsc::UnboundedSender<Control>>, msg: Control) {
		if has_payload(&msg)
			&& let Some(tx) = ctrl
		{
			let _ = tx.send(msg);
		}
	}
}

fn has_payload(msg: &Control) -> bool {
	match msg {
		Control::Subscribe(chs) | Control::Unsubscribe(chs) => !chs.is_empty(),
		_ => true,
	}
}

async fn run_endpoint(
	url: String,
	token: Option<String>,
	mut subs: Vec<Channel>,
	cfg: Reconnect,
	frames: mpsc::UnboundedSender<Frame>,
	mut ctrl: mpsc::UnboundedReceiver<Control>,
) {
	let mut failures = 0u32;
	loop {
		match tokio_tungstenite::connect_async(&url).await {
			Ok((ws, _)) => {
				let connected_at = Instant::now();
				let stopped = pump(ws, &token, &mut subs, &frames, &mut ctrl).await;
				if stopped {
					return;
				}

				// Only clear the failure count once the session proved healthy.
				// Resetting on connect alone lets an accept-then-drop server pin
				// the backoff at `initial_delay` and reconnect-storm forever.
				if connected_at.elapsed() >= HEALTHY_CONNECTION {
					failures = 0;
				}
			},
			Err(e) => {
				let _ = frames.send(Frame::Error(StreamError {
					code: "transport".to_owned(),
					message: e.to_string(),
					action: None,
					channel: None,
					token_id: None,
					condition_id: None,
				}));
			},
		}

		failures += 1;
		if matches!(cfg.max_attempts, Some(cap) if failures >= cap) {
			return;
		}

		tokio::time::sleep(jittered_backoff(
			cfg.initial_delay,
			cfg.max_delay,
			cfg.jitter,
			failures,
		))
		.await;
	}
}

/// Drive one live connection until it drops or a `Stop` arrives. Returns
/// `true` when the caller asked to stop (don't reconnect).
async fn pump(
	ws: tokio_tungstenite::WebSocketStream<
		tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
	>,
	token: &Option<String>,
	subs: &mut Vec<Channel>,
	frames: &mpsc::UnboundedSender<Frame>,
	ctrl: &mut mpsc::UnboundedReceiver<Control>,
) -> bool {
	let (mut write, mut read) = ws.split();
	if send_op(&mut write, "subscribe", subs, token).await.is_err() {
		return false;
	}

	let mut last_rx = Instant::now();
	// Start the first tick one interval out — `interval` would fire immediately,
	// racing a ping against the subscribe we just sent.
	let mut ping = tokio::time::interval_at(Instant::now() + PING_INTERVAL, PING_INTERVAL);
	ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
	loop {
		tokio::select! {
			msg = read.next() => match msg {
				Some(Ok(Message::Text(text))) => {
					last_rx = Instant::now();
					let Ok(value) = serde_json::from_str(&text) else { continue };
					let frame = decode_frame(value);
					if let Frame::Error(err) = &frame {
						match err.action.as_deref() {
							Some("resubscribe") => {
								let _ = send_op(&mut write, "subscribe", subs, token).await;
							}
							Some("reconnect") => {
								let _ = frames.send(frame);
								return false;
							}
							_ => {}
						}
					}

					if frames.send(frame).is_err() {
						return true;
					}
				}
				Some(Ok(Message::Ping(payload))) => {
					last_rx = Instant::now();
					let _ = write.send(Message::Pong(payload)).await;
				}
				Some(Ok(_)) => last_rx = Instant::now(),
				Some(Err(_)) | None => return false,
			},
			_ = ping.tick() => {
				if last_rx.elapsed() > SILENCE_TIMEOUT {
					return false;
				}
				if write.send(Message::Text("{\"op\":\"ping\"}".into())).await.is_err() {
					return false;
				}
			}
			control = ctrl.recv() => match control {
				Some(Control::Subscribe(chs)) => {
					for ch in &chs {
						if !subs.contains(ch) {
							subs.push(ch.clone());
						}
					}
					let _ = send_op(&mut write, "subscribe", &chs, token).await;
				}
				Some(Control::Unsubscribe(chs)) => {
					subs.retain(|c| !chs.contains(c));
					let _ = send_op(&mut write, "unsubscribe", &chs, token).await;
				}
				Some(Control::Ping) => {
					let _ = write.send(Message::Text("{\"op\":\"ping\"}".into())).await;
				}
				Some(Control::List) => {
					let _ = write.send(Message::Text("{\"op\":\"list\"}".into())).await;
				}
				Some(Control::Stop) | None => {
					let _ = write.send(Message::Close(None)).await;
					return true;
				}
			}
		}
	}
}

async fn send_op<S>(
	write: &mut S,
	op: &str,
	channels: &[Channel],
	token: &Option<String>,
) -> core::result::Result<(), ()>
where
	S: SinkExt<Message> + Unpin,
{
	if channels.is_empty() {
		return Ok(());
	}

	let wire: Vec<serde_json::Value> =
		channels.iter().map(|c| c.to_wire(token.as_deref())).collect();
	let payload = serde_json::json!({ "op": op, "channels": wire }).to_string();
	write.send(Message::Text(payload.into())).await.map_err(|_| ())
}

fn ws_url(base_url: &str, path: &str) -> String {
	let trimmed = base_url.trim_end_matches('/');
	if let Some(rest) = trimmed.strip_prefix("https://") {
		format!("wss://{rest}{path}")
	} else if let Some(rest) = trimmed.strip_prefix("http://") {
		format!("ws://{rest}{path}")
	} else {
		format!("{trimmed}{path}")
	}
}
