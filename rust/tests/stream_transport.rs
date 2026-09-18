#![cfg(feature = "streaming")]
#![allow(missing_docs)]

#[path = "support/http.rs"]
mod http;

use core::time::Duration;

use agara_sdk::{
	AgaraClient, AgaraStreamClient, Channel, Frame, Reconnect,
	client::Anonymous,
	ids::{ConditionId, EventId, TokenId},
};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::{
	Message,
	protocol::{CloseFrame, frame::coding::CloseCode},
};

#[rstest::rstest]
#[case("orderbook")]
#[case("best_quote")]
#[case("market_status")]
#[case("event_status")]
#[case("trades")]
#[case("account_events")]
#[tokio::test]
async fn every_subscription_form_and_policy_close(#[case] kind: &str) {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let channel = match kind {
		"orderbook" => Channel::Orderbook(TokenId::new("123").unwrap()),
		"best_quote" => Channel::BestQuote(TokenId::new("123").unwrap()),
		"market_status" => Channel::MarketStatus(
			ConditionId::new("0x1111111111111111111111111111111111111111111111111111111111111111")
				.unwrap(),
		),
		"event_status" => Channel::EventMarketStatus(
			EventId::new("10000000-0000-4000-8000-000000000001").unwrap(),
		),
		"trades" => Channel::Trades(
			ConditionId::new("0x1111111111111111111111111111111111111111111111111111111111111111")
				.unwrap(),
		),
		_ => Channel::AccountEvents,
	};
	let server = tokio::spawn(async move {
		let (socket, _) = listener.accept().await.unwrap();
		let mut ws = tokio_tungstenite::accept_async(socket).await.unwrap();
		let text = ws.next().await.unwrap().unwrap().into_text().unwrap();
		let request: serde_json::Value = serde_json::from_str(&text).unwrap();
		let channel = &request["channels"][0];
		ws.send(Message::Text(serde_json::json!({"op":"subscribed","channel":channel["name"],"token_id":channel.get("token_id"),"condition_id":channel.get("condition_id"),"event_id":channel.get("event_id")}).to_string().into())).await.unwrap();
		ws.send(Message::Close(Some(CloseFrame {
			code: CloseCode::Policy,
			reason: "policy".into(),
		})))
		.await
		.unwrap();
		request
	});
	let mut client = AgaraStreamClient::builder()
		.base_url(url.into())
		.token("agt_fixture".into())
		.build()
		.unwrap();
	client.subscribe([channel]).unwrap();
	// Act
	let mut stream = client.connect().await.unwrap();
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::ConnectionOpened(_))
	));
	let subscribed = stream.next().await.unwrap();
	let closed = stream.next().await;
	let ended = tokio::time::timeout(Duration::from_secs(1), stream.next()).await.unwrap();
	let request = server.await.unwrap();
	// Assert
	core::assert!(core::matches!(subscribed, Frame::Subscribed(_)));
	core::assert!(core::matches!(
		closed,
		Some(Frame::ClientError(
			agara_sdk::stream::StreamClientError::Closed { code: Some(1008), .. }
		))
	));
	core::assert!(ended.is_none());
	core::assert_eq!(request["op"], "subscribe");
	if kind == "event_status" {
		core::assert_eq!(
			request["channels"][0]["event_id"],
			"10000000-0000-4000-8000-000000000001"
		);
	}
	if kind == "account_events" {
		core::assert_eq!(request["channels"][0]["token"], "agt_fixture");
	}
}

#[tokio::test]
async fn close_cancels_full_queue_without_draining() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let server = tokio::spawn(async move {
		let (socket, _) = listener.accept().await.unwrap();
		let mut ws = tokio_tungstenite::accept_async(socket).await.unwrap();
		let _ = ws.next().await;
		for _ in 0..2048 {
			if ws
				.send(Message::Text(
					"{\"op\":\"heartbeat\",\"server_time\":\"2026-09-18T00:00:00Z\"}".into(),
				))
				.await
				.is_err()
			{
				break;
			}
		}
	});
	let mut client = AgaraStreamClient::builder().base_url(url.into()).build().unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new("123").unwrap())]).unwrap();
	let stream = client.connect().await.unwrap();
	tokio::time::sleep(Duration::from_millis(100)).await;
	// Act
	let result = tokio::time::timeout(Duration::from_millis(100), stream.close()).await;
	// Assert
	core::assert!(result.is_ok());
	server.abort();
}

#[tokio::test]
async fn close_cancels_long_reconnect_backoff() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	drop(listener);
	let mut client = AgaraStreamClient::builder()
		.base_url(url.into())
		.reconnect(
			Reconnect::builder()
				.initial_delay(Duration::from_secs(60))
				.max_delay(Duration::from_secs(60))
				.jitter(0.0)
				.max_reconnects(None)
				.build()
				.unwrap(),
		)
		.build()
		.unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new("123").unwrap())]).unwrap();
	let mut stream = client.connect().await.unwrap();
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::ClientError(_))
	));
	// Act
	let result = tokio::time::timeout(Duration::from_millis(100), stream.close()).await;
	// Assert
	core::assert!(result.is_ok());
}

#[rstest::rstest]
#[case(false)]
#[case(true)]
#[tokio::test]
async fn both_sse_routes_use_real_transport_and_deliver_eof(#[case] multiple: bool) {
	// Arrange
	let body = "data: {\"parsed\":[{\"id\":\"Crypto.BTC/USD\",\"price\":{\"price\":\"12345\",\"expo\":-2,\"publish_time_ms\":1780038840123}}]}\n\n";
	let server = http::Server::new(std::vec![Some(std::format!(
		"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
		body.len()
	))]);
	let client =
		AgaraClient::<Anonymous>::anonymous().base_url(server.url.clone().into()).call().unwrap();
	// Act
	let mut stream = if multiple {
		client.stream_prices(&["Crypto.BTC/USD", "Crypto.ETH/USD"]).await.unwrap()
	} else {
		client.stream_price("Crypto.BTC/USD").await.unwrap()
	};
	let event = stream.next().await.unwrap().unwrap();
	let eof = stream.next().await.unwrap();
	let requests = server.finish();
	// Assert
	core::assert_eq!(event.frame.parsed[0].price.price.as_str(), "12345");
	core::assert!(eof.is_none());
	core::assert!(requests[0].contains(if multiple {
		"/api/v1/prices/stream?symbols="
	} else {
		"/api/v1/prices/pyth-pro/stream?symbol="
	}));
	core::assert!(requests[0].contains("accept: text/event-stream"));
}

#[tokio::test]
async fn sse_admission_problem_preserves_recovery() {
	// Arrange
	let problem = serde_json::json!({"type":"urn:agara:problem:rate-limited","title":"Too many requests","detail":"You are rate limited, please try again later.","status":429,"request_id":"8051f907-4eeb-4ca9-bd84-b447ea65a68c","code":"rate_limited","recovery":{"strategy":"retry_after","after_seconds":123}});
	let server = http::Server::new(std::vec![http::json(429, &problem)]);
	let client =
		AgaraClient::<Anonymous>::anonymous().base_url(server.url.clone().into()).call().unwrap();
	// Act
	let result = client.stream_price("Crypto.BTC/USD").await;
	// Assert
	let error = result.err().unwrap();
	core::assert_eq!(error.retry_after(), Some(Duration::from_secs(123)));
	core::assert_eq!(server.finish().len(), 1);
}

#[test]
fn sse_bound_is_per_event_not_per_chunk() {
	// Arrange
	let line = "data: {\"parsed\":[]}\n\n";
	let chunk = line.repeat(60_000);
	let mut decoder = agara_sdk::prices::SseDecoder::default();
	// Act
	let events = decoder.push(chunk.as_bytes()).unwrap();
	// Assert
	core::assert_eq!(events.len(), 60_000);
	core::assert!(decoder.push(&std::vec![b'x'; 1_048_577]).is_err());
}

#[tokio::test]
async fn websocket_controls_roundtrip_without_reopening_endpoint() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let server = tokio::spawn(async move {
		let (socket, _) = listener.accept().await.unwrap();
		let mut ws = tokio_tungstenite::accept_async(socket).await.unwrap();
		let mut ops = Vec::new();
		for _ in 0..5 {
			let frame = ws.next().await.unwrap().unwrap().into_text().unwrap();
			let frame: serde_json::Value = serde_json::from_str(&frame).unwrap();
			let op = frame["op"].as_str().unwrap();
			ops.push(op.to_owned());
			let response = match op {
				"ping" => serde_json::json!({"op":"pong","server_time":"2026-09-18T00:00:00Z"}),
				"list" => {
					serde_json::json!({"op":"subscription_list","channels":[{"channel":"orderbook","token_id":"123"}]})
				},
				"unsubscribe" => {
					serde_json::json!({"op":"unsubscribed","channel":"orderbook","token_id":"123"})
				},
				_ => serde_json::json!({"op":"subscribed","channel":"orderbook","token_id":"123"}),
			};
			ws.send(Message::Text(response.to_string().into())).await.unwrap();
		}
		ops
	});
	let channel = Channel::Orderbook(TokenId::new("123").unwrap());
	let mut client = AgaraStreamClient::builder().base_url(url.into()).build().unwrap();
	client.subscribe([channel.clone()]).unwrap();
	let mut stream = client.connect().await.unwrap();
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::ConnectionOpened(_))
	));
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::Subscribed(_))
	));
	// Act
	stream.ping().unwrap();
	core::assert!(core::matches!(stream.next().await, Some(Frame::Pong(_))));
	stream.list_subscriptions().unwrap();
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::SubscriptionList(_))
	));
	stream.unsubscribe([channel.clone()]).unwrap();
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::Unsubscribed(_))
	));
	stream.subscribe([channel]).unwrap();
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::Subscribed(_))
	));
	stream.close().await;
	// Assert
	core::assert_eq!(
		server.await.unwrap(),
		["subscribe", "ping", "list", "unsubscribe", "subscribe"]
	);
}

#[tokio::test]
async fn websocket_reconnect_replays_subscriptions_and_signals_reseed() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let server = tokio::spawn(async move {
		let mut requests = Vec::new();
		for attempt in 0..2 {
			let (socket, _) = listener.accept().await.unwrap();
			let mut ws = tokio_tungstenite::accept_async(socket).await.unwrap();
			requests.push(ws.next().await.unwrap().unwrap().into_text().unwrap().to_string());
			let response = if attempt == 0 {
				serde_json::json!({"op":"error","failure":{"code":"server_restarting","title":"Server restarting","detail":"The server is restarting. Reconnect after a short delay.","recovery":{"strategy":"retry"}},"action":"reconnect"})
			} else {
				serde_json::json!({"op":"subscribed","channel":"orderbook","token_id":"123"})
			};
			ws.send(Message::Text(response.to_string().into())).await.unwrap();
		}
		requests
	});
	let mut client = AgaraStreamClient::builder()
		.base_url(url.into())
		.reconnect(
			Reconnect::builder()
				.initial_delay(Duration::from_millis(1))
				.max_delay(Duration::from_millis(1))
				.jitter(0.0)
				.max_reconnects(Some(2))
				.build()
				.unwrap(),
		)
		.build()
		.unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new("123").unwrap())]).unwrap();
	let mut stream = client.connect().await.unwrap();
	// Act
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::ConnectionOpened(_))
	));
	core::assert!(core::matches!(stream.next().await, Some(Frame::Error(_))));
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::ConnectionOpened(_))
	));
	core::assert!(core::matches!(
		stream.next().await,
		Some(Frame::Subscribed(_))
	));
	stream.close().await;
	let requests = server.await.unwrap();
	// Assert
	core::assert_eq!(requests.len(), 2);
	core::assert_eq!(requests[0], requests[1]);
}

#[tokio::test]
async fn malformed_websocket_json_is_typed_and_does_not_reconnect() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let server = tokio::spawn(async move {
		let (socket, _) = listener.accept().await.unwrap();
		let mut socket = tokio_tungstenite::accept_async(socket).await.unwrap();
		let _ = socket.next().await;
		socket.send(Message::Text("{broken".into())).await.unwrap();
	});
	let mut client = AgaraStreamClient::builder().base_url(url.into()).build().unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new("123").unwrap())]).unwrap();
	let mut stream = client.connect().await.unwrap();
	let _ = stream.next().await;

	// Act
	let failure = stream.next().await;
	let end = stream.next().await;

	// Assert
	core::assert!(core::matches!(
		failure,
		Some(Frame::ClientError(
			agara_sdk::stream::StreamClientError::Decode {
				source: agara_sdk::frames::FrameDecodeError::Json(_),
				..
			}
		))
	));
	core::assert!(end.is_none());
	server.await.unwrap();
}

#[tokio::test]
async fn acknowledged_short_connections_do_not_reset_reconnect_budget() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let server = tokio::spawn(async move {
		for _ in 0..2 {
			let (socket, _) = listener.accept().await.unwrap();
			let mut socket = tokio_tungstenite::accept_async(socket).await.unwrap();
			let _ = socket.next().await;
			socket
				.send(Message::Text(
					serde_json::json!({"op":"subscribed","channel":"orderbook","token_id":"123"})
						.to_string()
						.into(),
				))
				.await
				.unwrap();
			socket
				.send(Message::Close(Some(CloseFrame {
					code: CloseCode::Away,
					reason: "restart".into(),
				})))
				.await
				.unwrap();
		}
	});
	let mut client = AgaraStreamClient::builder()
		.base_url(url.into())
		.reconnect(
			Reconnect::builder()
				.initial_delay(Duration::from_millis(1))
				.max_delay(Duration::from_millis(1))
				.jitter(0.0)
				.max_reconnects(Some(1))
				.build()
				.unwrap(),
		)
		.build()
		.unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new("123").unwrap())]).unwrap();
	let mut stream = client.connect().await.unwrap();
	let mut opened = 0;
	let mut exhausted = false;

	// Act
	while let Some(frame) = stream.next().await {
		match frame {
			Frame::ConnectionOpened(_) => opened += 1,
			Frame::ClientError(agara_sdk::stream::StreamClientError::ReconnectExhausted {
				reconnects: 1,
				..
			}) => exhausted = true,
			_ => {},
		}
	}

	// Assert
	core::assert_eq!(opened, 2);
	core::assert!(exhausted);
	server.await.unwrap();
}

#[tokio::test]
async fn output_backpressure_is_observable_before_termination() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let (done, sent) = tokio::sync::oneshot::channel();
	let server = tokio::spawn(async move {
		let (socket, _) = listener.accept().await.unwrap();
		let mut socket = tokio_tungstenite::accept_async(socket).await.unwrap();
		let _ = socket.next().await;
		for _ in 0..2048 {
			if socket
				.send(Message::Text(
					"{\"op\":\"heartbeat\",\"server_time\":\"2026-09-18T00:00:00Z\"}".into(),
				))
				.await
				.is_err()
			{
				break;
			}
		}
		let _ = done.send(());
	});
	let mut client = AgaraStreamClient::builder().base_url(url.into()).build().unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new("123").unwrap())]).unwrap();
	let mut stream = client.connect().await.unwrap();
	sent.await.unwrap();
	tokio::time::sleep(Duration::from_millis(20)).await;
	let mut backpressure = false;

	// Act
	while let Some(frame) = stream.next().await {
		if core::matches!(
			frame,
			Frame::ClientError(agara_sdk::stream::StreamClientError::Backpressure { .. })
		) {
			backpressure = true;
		}
	}

	// Assert
	core::assert!(backpressure);
	server.await.unwrap();
}

#[tokio::test]
async fn websocket_idle_deadline_is_independent_of_ping_ticks() {
	// Arrange
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let (release, hold) = tokio::sync::oneshot::channel::<()>();
	let server = tokio::spawn(async move {
		let (socket, _) = listener.accept().await.unwrap();
		let mut socket = tokio_tungstenite::accept_async(socket).await.unwrap();
		let _ = socket.next().await;
		socket
			.send(Message::Text(
				serde_json::json!({"op":"subscribed","channel":"orderbook","token_id":"123"})
					.to_string()
					.into(),
			))
			.await
			.unwrap();
		let _ = hold.await;
	});
	let mut client = AgaraStreamClient::builder()
		.base_url(url.into())
		.reconnect(Reconnect::builder().max_reconnects(Some(0)).build().unwrap())
		.build()
		.unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new("123").unwrap())]).unwrap();
	let mut stream = client.connect().await.unwrap();
	let _ = stream.next().await;
	let _ = stream.next().await;
	tokio::time::pause();

	// Act
	tokio::time::advance(Duration::from_secs(36)).await;
	let failure = stream.next().await;

	// Assert
	core::assert!(core::matches!(
		failure,
		Some(Frame::ClientError(
			agara_sdk::stream::StreamClientError::IdleTimeout { .. }
		))
	));
	stream.close().await;
	let _ = release.send(());
	server.await.unwrap();
}

#[rstest::rstest]
#[case(None)]
#[case(Some("application/json"))]
#[case(Some("text/event-streaming"))]
#[tokio::test]
async fn sse_content_type_errors_preserve_the_actual_header(#[case] content_type: Option<&str>) {
	// Arrange
	let header = content_type.map_or(String::new(), |value| {
		std::format!("Content-Type: {value}\r\n")
	});
	let server = http::Server::new(std::vec![Some(std::format!(
		"HTTP/1.1 200 OK\r\n{header}Content-Length: 0\r\nConnection: close\r\n\r\n"
	))]);
	let client =
		AgaraClient::<Anonymous>::anonymous().base_url(server.url.clone().into()).call().unwrap();

	// Act
	let error = client.stream_price("BTC/USD").await.err().unwrap();

	// Assert
	core::assert!(
		core::matches!(error, agara_sdk::AgaraError::PriceStream(agara_sdk::prices::PriceStreamError::InvalidContentType(value)) if value.as_deref() == content_type)
	);
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn sse_header_body_status_disagreement_is_not_a_retryable_problem() {
	// Arrange
	let problem = serde_json::json!({"type":"urn:agara:problem:rate-limited","title":"Too many requests","detail":"You are rate limited, please try again later.","status":429,"request_id":"8051f907-4eeb-4ca9-bd84-b447ea65a68c","code":"rate_limited","recovery":{"strategy":"retry_after","after_seconds":123}});
	let server = http::Server::new(std::vec![http::json(503, &problem)]);
	let client =
		AgaraClient::<Anonymous>::anonymous().base_url(server.url.clone().into()).call().unwrap();

	// Act
	let error = client.stream_price("BTC/USD").await.err().unwrap();

	// Assert
	core::assert!(core::matches!(
		error,
		agara_sdk::AgaraError::PriceStream(agara_sdk::prices::PriceStreamError::InvalidProblem {
			source: agara_sdk::problem::ProblemDecodeError::HttpStatusMismatch {
				actual: 503,
				declared: 429
			},
			..
		})
	));
	core::assert_eq!(server.finish().len(), 1);
}

#[tokio::test]
async fn sse_idle_failure_keeps_the_price_stream_error_type() {
	// Arrange
	let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
	let url = std::format!("http://{}", listener.local_addr().unwrap());
	let (release, hold) = std::sync::mpsc::channel::<()>();
	let server = std::thread::spawn(move || {
		let (mut socket, _) = listener.accept().unwrap();
		socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
		let mut request = Vec::new();
		let mut buffer = [0u8; 1024];
		while !request.windows(4).any(|window| window == b"\r\n\r\n") {
			let read = std::io::Read::read(&mut socket, &mut buffer).unwrap();
			core::assert!(read > 0);
			request.extend_from_slice(&buffer[..read]);
		}
		std::io::Write::write_all(&mut socket, b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n").unwrap();
		let _ = hold.recv();
	});
	let client = AgaraClient::<Anonymous>::anonymous().base_url(url.into()).call().unwrap();
	let mut stream = client.stream_price("Crypto.BTC/USD").await.unwrap();
	tokio::time::pause();
	let pending = tokio::spawn(async move { stream.next().await });
	tokio::task::yield_now().await;

	// Act
	tokio::time::advance(Duration::from_secs(46)).await;
	let result = pending.await.unwrap();

	// Assert
	core::assert!(core::matches!(
		result,
		Err(agara_sdk::AgaraError::PriceStream(
			agara_sdk::prices::PriceStreamError::IdleTimeout
		))
	));
	let _ = release.send(());
	server.join().unwrap();
}
