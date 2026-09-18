#![cfg(feature = "streaming")]
#![allow(missing_docs)]

use core::time::Duration;

use agara_sdk::{
	AgaraError, AgaraStreamClient, Channel, Frame, Reconnect,
	frames::{self, CollateralChanged, FrameDecodeError, OrderbookSnapshot, Scale, Subscribed},
	ids::TokenId,
	prices::{PriceMantissa, PriceStreamError, ProviderPrice, ProviderSymbol, SseDecoder},
	stream::{ReconnectField, StreamClientError, StreamEndpoint},
};

const TOKEN: &str = "123";
const CONDITION: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
const EVENT: &str = "10000000-0000-4000-8000-000000000001";
const MAX_EVENT_BYTES: usize = 1_048_576;

#[rstest::fixture]
fn snapshot() -> serde_json::Value {
	serde_json::json!({"op":"update","channel":"orderbook","token_id":TOKEN,"sequence":7,"data":{"kind":"snapshot","bids":[[50,100]],"asks":[],"tick_size":1,"price_scale":100,"size_scale":1000000}})
}

#[rstest::rstest]
#[case(f64::NAN)]
#[case(f64::INFINITY)]
#[case(f64::NEG_INFINITY)]
#[case(-0.01)]
#[case(1.01)]
fn reconnect_rejects_invalid_jitter_without_panicking(#[case] value: f64) {
	// Arrange
	let result = std::panic::catch_unwind(|| Reconnect::builder().jitter(value).build());

	// Act
	let error = result.expect("invalid inputs must not panic").unwrap_err();

	// Assert
	core::assert!(core::matches!(
		error,
		StreamClientError::InvalidReconnect { field: ReconnectField::Jitter }
	));
}

#[rstest::rstest]
#[case(Duration::ZERO, Duration::from_secs(1), ReconnectField::InitialDelay)]
#[case(
	Duration::from_secs(2),
	Duration::from_secs(1),
	ReconnectField::InitialDelay
)]
#[case(Duration::from_secs(1), Duration::MAX, ReconnectField::MaxDelay)]
fn reconnect_rejects_invalid_durations(
	#[case] initial: Duration,
	#[case] maximum: Duration,
	#[case] expected: ReconnectField,
) {
	// Arrange
	let result = std::panic::catch_unwind(|| {
		Reconnect::builder().initial_delay(initial).max_delay(maximum).build()
	});

	// Act
	let error = result.unwrap().unwrap_err();

	// Assert
	core::assert!(
		core::matches!(error, StreamClientError::InvalidReconnect { field } if field == expected)
	);
}

#[rstest::rstest]
fn reconnect_limit_cannot_overflow_or_bypass_shared_bounds() {
	// Arrange
	let invalid = Reconnect::builder().max_reconnects(Some(u32::MAX)).build();
	let one_shot = Reconnect::builder().max_reconnects(Some(0)).build().unwrap();
	let unlimited = Reconnect::builder().max_reconnects(None).build().unwrap();

	// Act / Assert
	core::assert!(core::matches!(
		invalid,
		Err(StreamClientError::InvalidReconnect { field: ReconnectField::MaxReconnects })
	));
	core::assert_eq!(one_shot.max_reconnects(), Some(0));
	core::assert_eq!(unlimited.max_reconnects(), None);
}

#[rstest::rstest]
#[case("ftp://localhost")]
#[case("https://user:secret@example.com")]
#[case("https://example.com?token=secret")]
#[case("https://example.com#fragment")]
#[case("not a URL")]
fn invalid_stream_urls_fail_before_connect(#[case] value: &'static str) {
	// Arrange
	let result = AgaraStreamClient::builder().base_url(value.into()).build();

	// Act / Assert
	core::assert!(core::matches!(
		result,
		Err(AgaraError::Stream(StreamClientError::InvalidBaseUrl(_)))
	));
}

#[rstest::rstest]
#[case("")]
#[case(" ")]
#[case("agt_abc\n")]
#[case("agt_abc\0")]
fn invalid_bearers_fail_without_io(#[case] value: &'static str) {
	// Arrange
	let result = AgaraStreamClient::builder().token(value.into()).build();

	// Act / Assert
	core::assert!(core::matches!(
		result,
		Err(AgaraError::Stream(StreamClientError::InvalidToken))
	));
}

#[tokio::test]
async fn rejected_subscription_batch_does_not_mutate_state() {
	// Arrange
	let mut client = AgaraStreamClient::builder().build().unwrap();

	// Act
	let subscription = client.subscribe([
		Channel::Orderbook(TokenId::new(TOKEN).unwrap()),
		Channel::AccountEvents,
	]);
	let connection = client.connect().await;

	// Assert
	core::assert!(core::matches!(
		subscription,
		Err(AgaraError::Stream(StreamClientError::MissingToken))
	));
	core::assert!(core::matches!(
		connection,
		Err(AgaraError::Stream(StreamClientError::NoSubscriptions))
	));
}

#[tokio::test]
async fn control_queue_reports_backpressure_instead_of_discarding_commands() {
	// Arrange
	let mut client =
		AgaraStreamClient::builder().base_url("http://127.0.0.1:9".into()).build().unwrap();
	client.subscribe([Channel::Orderbook(TokenId::new(TOKEN).unwrap())]).unwrap();
	let stream = client.connect().await.unwrap();

	// Act
	for _ in 0..64 {
		stream.ping().unwrap();
	}
	let extra = stream.ping();

	// Assert
	core::assert!(core::matches!(
		extra,
		Err(AgaraError::Stream(StreamClientError::ControlBackpressure {
			endpoint: StreamEndpoint::Market
		}))
	));
	stream.close().await;
}

#[rstest::rstest]
#[case("price_scale", serde_json::json!(0))]
#[case("price_scale", serde_json::json!(-1))]
#[case("size_scale", serde_json::json!(4294967296u64))]
#[case("tick_size", serde_json::json!(0))]
fn known_frame_scalar_violations_are_typed(
	mut snapshot: serde_json::Value,
	#[case] field: &str,
	#[case] value: serde_json::Value,
) {
	// Arrange
	snapshot["data"][field] = value;

	// Act
	let result = frames::try_decode_frame(snapshot);

	// Assert
	core::assert!(core::matches!(result, Err(FrameDecodeError::Payload(_))));
}

#[rstest::rstest]
fn unsigned_native_values_preserve_the_complete_u64_range(mut snapshot: serde_json::Value) {
	// Arrange
	snapshot["data"]["bids"][0][1] = serde_json::json!(u64::MAX);

	// Act
	let frame = frames::try_decode_frame(snapshot).unwrap();

	// Assert
	let Frame::OrderbookSnapshot(book) = frame else {
		core::panic!("expected snapshot")
	};
	core::assert_eq!(book.bids[0].size, u64::MAX);
}

#[rstest::rstest]
fn negative_size_is_not_coerced_to_zero(mut snapshot: serde_json::Value) {
	// Arrange
	snapshot["data"]["bids"][0][1] = serde_json::json!(-1);

	// Act
	let frame = frames::decode_frame(snapshot.clone());

	// Assert
	core::assert!(
		core::matches!(frame, Frame::Malformed { error: FrameDecodeError::Payload(_), raw } if raw == snapshot)
	);
}

#[rstest::rstest]
#[case(serde_json::json!({"channel":"orderbook","token_id":TOKEN,"condition_id":CONDITION}))]
#[case(serde_json::json!({"channel":"orderbook"}))]
#[case(serde_json::json!({"channel":"market_status","event_id":"not-a-uuid"}))]
#[case(serde_json::json!({"channel":"account_events","event_id":EVENT}))]
#[case(serde_json::json!({"channel":"garbage","token_id":TOKEN}))]
fn control_subject_serde_enforces_exact_scope(#[case] raw: serde_json::Value) {
	// Arrange / Act
	let result = serde_json::from_value::<Subscribed>(raw);

	// Assert
	core::assert!(result.is_err());
}

#[rstest::rstest]
fn event_scope_requires_no_engine_sequence() {
	// Arrange
	let raw = serde_json::json!({"op":"update","channel":"market_status","event_id":EVENT,"sequence":1,"data":{"kind":"current_market_changed"}});

	// Act
	let result = frames::try_decode_frame(raw);

	// Assert
	core::assert!(core::matches!(result, Err(FrameDecodeError::Field { .. })));
}

#[rstest::rstest]
fn serde_cannot_bypass_positive_scales_or_complete_provenance(snapshot: serde_json::Value) {
	// Arrange
	let mut payload = snapshot["data"].clone();
	payload["token_id"] = serde_json::json!(TOKEN);
	payload["sequence"] = serde_json::json!(7);
	payload["price_scale"] = serde_json::json!(0);
	let partial = serde_json::json!({"sequence":1,"amount_micro":"1","cash_balance_micro":"2","batch_hash":CONDITION});

	// Act
	let scale = Scale::new(0);
	let book = serde_json::from_value::<OrderbookSnapshot>(payload);
	let collateral = serde_json::from_value::<CollateralChanged>(partial);

	// Assert
	core::assert!(core::matches!(scale, Err(FrameDecodeError::ZeroScale)));
	core::assert!(book.is_err());
	core::assert!(collateral.is_err());
}

#[rstest::rstest]
fn duplicate_frame_properties_do_not_disappear_before_validation() {
	// Arrange
	let text = r#"{"op":"heartbeat","op":"pong","server_time":"2026-09-18T00:00:00Z"}"#;

	// Act
	let result = frames::try_decode_text(text);

	// Assert
	core::assert!(core::matches!(result, Err(FrameDecodeError::Json(_))));
}

#[rstest::rstest]
fn future_payloads_retain_their_complete_envelopes() {
	// Arrange
	let unknown = serde_json::json!({"op":"update","channel":"market_status","event_id":EVENT,"data":{"kind":"future_event","new":123}});

	// Act
	let frame = frames::try_decode_frame(unknown.clone()).unwrap();

	// Assert
	core::assert!(core::matches!(frame, Frame::Unknown(raw) if raw == unknown));
}

#[rstest::rstest]
#[case("")]
#[case("-")]
#[case("NaN")]
#[case("inf")]
#[case("+1")]
#[case("1.2")]
#[case("1e3")]
#[case(" 1")]
fn mantissa_construction_and_serde_share_strict_validation(#[case] value: &str) {
	// Arrange
	let json = serde_json::json!({"price":value,"expo":-2,"publish_time_ms":1});

	// Act
	let constructor = PriceMantissa::new(value);
	let wire = serde_json::from_value::<ProviderPrice>(json);

	// Assert
	core::assert!(core::matches!(
		constructor,
		Err(PriceStreamError::InvalidMantissa(_))
	));
	core::assert!(wire.is_err());
}

#[rstest::rstest]
#[case(serde_json::json!(-1))]
#[case(serde_json::json!(1.5))]
#[case(serde_json::json!("1"))]
fn price_timestamp_must_be_an_unsigned_wire_integer(#[case] timestamp: serde_json::Value) {
	// Arrange
	let json = serde_json::json!({"price":"123","expo":-2,"publish_time_ms":timestamp});

	// Act
	let result = serde_json::from_value::<ProviderPrice>(json);

	// Assert
	core::assert!(result.is_err());
}

#[rstest::rstest]
#[case("")]
#[case("BTC USD")]
#[case("日本")]
#[case("BTC?token=secret")]
fn provider_symbols_are_validated_at_construction(#[case] value: &str) {
	// Arrange / Act
	let result = ProviderSymbol::new(value);

	// Assert
	core::assert!(core::matches!(
		result,
		Err(PriceStreamError::InvalidSymbol(_))
	));
}

#[rstest::rstest]
#[case(b"data: {]\n\n".as_slice(), "json")]
#[case(b"data: {\"parsed\":false}\n\n".as_slice(), "payload")]
#[case(b"data: {\"parsed\":[],\"parsed\":[]}\n\n".as_slice(), "json")]
#[case(b"\xff\n".as_slice(), "utf8")]
#[case(b"retry: 18446744073709551616\n\n".as_slice(), "retry")]
fn sse_failure_causes_are_machine_matchable(#[case] bytes: &[u8], #[case] expected: &str) {
	// Arrange
	let mut decoder = SseDecoder::default();

	// Act
	let error = decoder.push(bytes).unwrap_err();

	// Assert
	let actual = match error {
		PriceStreamError::Json(_) => "json",
		PriceStreamError::Payload(_) => "payload",
		PriceStreamError::Utf8(_) => "utf8",
		PriceStreamError::RetryOverflow => "retry",
		other => core::panic!("unexpected: {other:?}"),
	};
	core::assert_eq!(actual, expected);
	core::assert!(core::matches!(
		decoder.push(b""),
		Err(PriceStreamError::DecoderFailed)
	));
}

#[rstest::rstest]
#[case("\n")]
#[case("\r\n")]
#[case("\r")]
fn sse_handles_every_line_ending_at_every_chunk_boundary(#[case] newline: &str) {
	// Arrange
	let text = std::format!(
		"\u{feff}: heartbeat{newline}{newline}event: 界{newline}id: 4{newline}data: {{\"parsed\":[]}}{newline}{newline}"
	);
	let mut decoder = SseDecoder::default();
	let mut events = Vec::new();

	// Act
	for byte in text.as_bytes() {
		events.extend(decoder.push(&[*byte]).unwrap());
	}
	let end = decoder.finish();

	// Assert
	core::assert!(end.is_ok());
	core::assert_eq!(events.len(), 1);
	core::assert_eq!(events[0].event.as_deref(), Some("界"));
	core::assert_eq!(events[0].id.as_deref(), Some("4"));
}

#[rstest::rstest]
fn sse_truncation_and_bounds_are_distinct_errors() {
	// Arrange
	let mut truncated = SseDecoder::default();
	let mut oversized = SseDecoder::default();
	truncated.push(b"data: {").unwrap();

	// Act
	let eof = truncated.finish();
	let limit = oversized.push(&std::vec![b'x'; MAX_EVENT_BYTES + 1]);

	// Assert
	core::assert!(core::matches!(
		eof,
		Err(PriceStreamError::UnexpectedEof { pending_bytes: 7 })
	));
	core::assert!(core::matches!(
		limit,
		Err(PriceStreamError::TooLarge { limit: MAX_EVENT_BYTES })
	));
}

#[rstest::rstest]
#[case(
	"not a URL",
	agara_sdk::stream::StreamUrlError::Parse(url::ParseError::RelativeUrlWithoutBase)
)]
#[case(
	"ftp://localhost",
	agara_sdk::stream::StreamUrlError::UnsupportedScheme
)]
#[case(
	"https://user:secret@example.com",
	agara_sdk::stream::StreamUrlError::Credentials
)]
#[case("https://example.com?x=1", agara_sdk::stream::StreamUrlError::Query)]
#[case("https://example.com#x", agara_sdk::stream::StreamUrlError::Fragment)]
fn stream_url_failure_preserves_its_machine_reason(
	#[case] input: &'static str,
	#[case] expected: agara_sdk::stream::StreamUrlError,
) {
	// Arrange
	let result = AgaraStreamClient::builder().base_url(input.into()).build();

	// Act / Assert
	core::assert!(
		core::matches!(result, Err(AgaraError::Stream(StreamClientError::InvalidBaseUrl(reason))) if reason == expected)
	);
}
