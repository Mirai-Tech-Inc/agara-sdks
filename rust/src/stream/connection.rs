use core::time::Duration;

use std::{borrow::Cow, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use tokio::{sync::mpsc, time::Instant};
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};

use crate::{
	frames::{self, Channel, Frame, FrameDecodeError, WebSocketAction},
	problem::Recovery,
	retry,
};

use super::{Control, Reconnect, StreamClientError, StreamEndpoint, StreamPhase};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const PING_INTERVAL: Duration = Duration::from_secs(25);
const SILENCE_TIMEOUT: Duration = Duration::from_secs(35);
const RESERVED_ERROR_SLOTS: usize = 2;
const MAX_RESUBSCRIBES: u32 = 8;
const TERMINAL_CLOSE_CODES: [u16; 6] = [1002, 1003, 1007, 1008, 1009, 4001];

enum SessionEnd {
	Stopped,
	Retry(Option<StreamClientError>),
	Terminal(StreamClientError),
}

pub(super) async fn run(
	endpoint: StreamEndpoint,
	url: reqwest::Url,
	token: Option<Cow<'static, str>>,
	mut subscriptions: Vec<Channel>,
	config: Reconnect,
	frames: mpsc::Sender<Frame>,
	mut controls: mpsc::Receiver<Control>,
) {
	let mut reconnects = 0u32;
	loop {
		if frames.is_closed() || controls.is_closed() {
			return;
		}

		let end = match tokio::time::timeout(
			CONNECT_TIMEOUT,
			tokio_tungstenite::connect_async_with_config(
				url.as_str(),
				Some(
					tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
						.max_message_size(Some(frames::MAX_FRAME_BYTES))
						.max_frame_size(Some(frames::MAX_FRAME_BYTES)),
				),
				false,
			),
		)
		.await
		{
			Ok(Ok((socket, _))) => {
				if !emit(
					&frames,
					endpoint,
					Frame::ConnectionOpened(subscriptions.clone()),
				) {
					return;
				}

				pump(
					endpoint,
					socket,
					token.as_deref(),
					&mut subscriptions,
					&frames,
					&mut controls,
				)
				.await
			},
			Ok(Err(source)) => transport_end(endpoint, StreamPhase::Connect, source),
			Err(_) => SessionEnd::Retry(Some(StreamClientError::ConnectTimeout { endpoint })),
		};

		match end {
			SessionEnd::Stopped => return,
			SessionEnd::Terminal(error) => {
				emit(&frames, endpoint, Frame::ClientError(error));

				return;
			},
			SessionEnd::Retry(error) => {
				if let Some(error) = error
					&& !emit(&frames, endpoint, Frame::ClientError(error))
				{
					return;
				}
			},
		}

		if config.max_reconnects().is_some_and(|maximum| reconnects >= maximum)
			|| reconnects == u32::MAX
		{
			emit(
				&frames,
				endpoint,
				Frame::ClientError(StreamClientError::ReconnectExhausted { endpoint, reconnects }),
			);

			return;
		}
		reconnects += 1;
		let delay = retry::jittered_backoff(
			config.initial_delay(),
			config.max_delay(),
			config.jitter(),
			reconnects,
		)
		.min(config.max_delay());
		let Some(deadline) = Instant::now().checked_add(delay) else {
			emit(
				&frames,
				endpoint,
				Frame::ClientError(StreamClientError::DeadlineOverflow),
			);

			return;
		};
		tokio::time::sleep_until(deadline).await;
	}
}

fn emit(sender: &mpsc::Sender<Frame>, endpoint: StreamEndpoint, frame: Frame) -> bool {
	// Reserve one error slot per endpoint so backpressure is observable.
	if sender.capacity() <= RESERVED_ERROR_SLOTS {
		let _ = sender.try_send(Frame::ClientError(StreamClientError::Backpressure {
			endpoint,
		}));

		return false;
	}

	sender.try_send(frame).is_ok()
}

fn transport(
	endpoint: StreamEndpoint,
	phase: StreamPhase,
	source: WebSocketError,
) -> StreamClientError {
	StreamClientError::Transport { endpoint, phase, source: Arc::new(source) }
}

fn transport_end(
	endpoint: StreamEndpoint,
	phase: StreamPhase,
	source: WebSocketError,
) -> SessionEnd {
	let retryable = core::matches!(
		&source,
		WebSocketError::Io(_) | WebSocketError::ConnectionClosed | WebSocketError::AlreadyClosed
	);
	let error = transport(endpoint, phase, source);

	if retryable {
		SessionEnd::Retry(Some(error))
	} else {
		SessionEnd::Terminal(error)
	}
}

async fn pump(
	endpoint: StreamEndpoint,
	socket: tokio_tungstenite::WebSocketStream<
		tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
	>,
	token: Option<&str>,
	subscriptions: &mut Vec<Channel>,
	frames: &mpsc::Sender<Frame>,
	controls: &mut mpsc::Receiver<Control>,
) -> SessionEnd {
	let (mut write, mut read) = socket.split();
	if let Err(source) = send_op(&mut write, "subscribe", subscriptions, token).await {
		return SessionEnd::Retry(Some(transport(endpoint, StreamPhase::Write, source)));
	}

	let mut last_rx = Instant::now();
	let Some(first_ping) = last_rx.checked_add(PING_INTERVAL) else {
		return SessionEnd::Terminal(StreamClientError::DeadlineOverflow);
	};
	let mut ping = tokio::time::interval_at(first_ping, PING_INTERVAL);
	ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
	let mut resubscribes = 0;
	loop {
		let Some(idle_deadline) = last_rx.checked_add(SILENCE_TIMEOUT) else {
			return SessionEnd::Terminal(StreamClientError::DeadlineOverflow);
		};

		tokio::select! {
			_ = tokio::time::sleep_until(idle_deadline) => return SessionEnd::Retry(Some(StreamClientError::IdleTimeout { endpoint })),
			message = read.next() => match message {
				Some(Ok(Message::Text(text))) => {
					last_rx = Instant::now();
					let raw = match crate::problem::parse_json_value_with_limit(text.as_bytes(), frames::MAX_FRAME_BYTES) {
						Ok(value) => value,
						Err(source) => return SessionEnd::Terminal(StreamClientError::Decode { endpoint, source: FrameDecodeError::Json(Arc::new(source)) }),
					};
					let frame = frames::decode_frame(raw);
					if core::matches!(&frame, Frame::Malformed { .. }) {
						emit(frames, endpoint, frame);

						return SessionEnd::Stopped;
					}

					let recovery = if let Frame::Error(error) = &frame {
						let refresh_identity = error.failure.is_known() && core::matches!(&error.failure.recovery, Recovery::RefreshIdentityToken);
						if refresh_identity {
							emit(frames, endpoint, frame);

							return SessionEnd::Stopped;
						}

						match error.automatic_action() {
							Some(WebSocketAction::Reconnect) => {
								emit(frames, endpoint, frame);

								return SessionEnd::Retry(None);
							},
							Some(WebSocketAction::Resubscribe) => {
								let affected = subscriptions.iter().filter(|subject| {
									let wire = subject.to_wire(None);
									error.channel.as_ref().is_none_or(|channel| subject.name() == channel)
										&& error.token_id.as_ref().is_none_or(|id| wire.get("token_id").and_then(serde_json::Value::as_str) == Some(id.as_str()))
										&& error.condition_id.as_ref().is_none_or(|id| wire.get("condition_id").and_then(serde_json::Value::as_str) == Some(id.as_str()))
										&& error.event_id.as_ref().is_none_or(|id| wire.get("event_id").and_then(serde_json::Value::as_str) == Some(id.as_str()))
								}).cloned().collect::<Vec<_>>();

								Some(affected)
							},
							_ => None,
						}
					} else { None };

					if !emit(frames, endpoint, frame) {
						return SessionEnd::Stopped;
					}
					if let Some(subjects) = recovery {
						if subjects.is_empty() { continue; }
						if resubscribes >= MAX_RESUBSCRIBES {
							return SessionEnd::Terminal(StreamClientError::ResubscribeExhausted { endpoint, attempts: resubscribes });
						}
						resubscribes += 1;
						tokio::time::sleep(retry::DEFAULT_INITIAL_BACKOFF).await;
						for operation in ["unsubscribe", "subscribe"] {
							if let Err(source) = send_op(&mut write, operation, &subjects, token).await {
								return SessionEnd::Retry(Some(transport(endpoint, StreamPhase::Write, source)));
							}
						}
					}
				},
				Some(Ok(Message::Ping(payload))) => {
					last_rx = Instant::now();
					if let Err(source) = write.send(Message::Pong(payload)).await {
						return SessionEnd::Retry(Some(transport(endpoint, StreamPhase::Write, source)));
					}
				},
				Some(Ok(Message::Pong(_))) => last_rx = Instant::now(),
				Some(Ok(Message::Close(frame))) => {
					let code = frame.as_ref().map(|frame| u16::from(frame.code));
					let reason = frame.map(|frame| Cow::Owned(frame.reason.to_string()));
					let error = StreamClientError::closed().endpoint(endpoint).maybe_code(code).maybe_reason(reason).call();

					return if code.is_some_and(|code| TERMINAL_CLOSE_CODES.contains(&code)) { SessionEnd::Terminal(error) } else { SessionEnd::Retry(Some(error)) };
				},
				Some(Ok(_)) => return SessionEnd::Terminal(StreamClientError::Decode { endpoint, source: FrameDecodeError::UnsupportedMessage }),
				Some(Err(source)) => return transport_end(endpoint, StreamPhase::Read, source),
				None => return SessionEnd::Retry(Some(StreamClientError::closed().endpoint(endpoint).call())),
			},
			_ = ping.tick() => {
				if let Err(source) = write.send(Message::Text("{\"op\":\"ping\"}".into())).await {
					return SessionEnd::Retry(Some(transport(endpoint, StreamPhase::Write, source)));
				}
			},
			control = controls.recv() => {
				let result = match control {
					Some(Control::Subscribe(channels)) => {
						for channel in &channels {
							if !subscriptions.contains(channel) { subscriptions.push(channel.clone()); }
						}

						send_op(&mut write, "subscribe", &channels, token).await
					},
					Some(Control::Unsubscribe(channels)) => {
						subscriptions.retain(|channel| !channels.contains(channel));

						send_op(&mut write, "unsubscribe", &channels, token).await
					},
					Some(Control::Ping) => write.send(Message::Text("{\"op\":\"ping\"}".into())).await,
					Some(Control::List) => write.send(Message::Text("{\"op\":\"list\"}".into())).await,
					None => return SessionEnd::Stopped,
				};
				if let Err(source) = result {
					return SessionEnd::Retry(Some(transport(endpoint, StreamPhase::Write, source)));
				}
			},
		}
	}
}

async fn send_op<S>(
	write: &mut S,
	operation: &str,
	channels: &[Channel],
	token: Option<&str>,
) -> Result<(), WebSocketError>
where
	S: futures_util::Sink<Message, Error = WebSocketError> + Unpin,
{
	if channels.is_empty() {
		return Ok(());
	}
	let channels: Vec<_> = channels.iter().map(|channel| channel.to_wire(token)).collect();
	let message = serde_json::json!({"op": operation, "channels": channels}).to_string();

	write.send(Message::Text(message.into())).await
}
