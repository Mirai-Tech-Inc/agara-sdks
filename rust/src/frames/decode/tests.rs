#![cfg(test)]

use super::*;

#[rstest::rstest]
#[case("future_failure", "future_recovery", "reconnect")]
#[case("future_failure", "none", "resubscribe")]
fn future_failure_never_activates_known_action(
	#[case] code: &str,
	#[case] strategy: &str,
	#[case] action: &str,
) {
	// Arrange
	let raw = serde_json::json!({"op":"error","failure":{"code":code,"title":"Future failure","recovery":{"strategy":strategy}},"action":action});

	// Act
	let frame = try_decode_frame(raw).unwrap();

	// Assert
	let Frame::Error(error) = frame else {
		core::panic!("expected preserved future error")
	};
	core::assert!(error.automatic_action().is_none());
}

#[rstest::rstest]
fn malformed_known_recovery_is_a_typed_decode_failure() {
	// Arrange
	let raw = serde_json::json!({"op":"error","failure":{"code":"stream_subject_unavailable","title":"Stream subject temporarily unavailable","detail":"This stream subject is temporarily unavailable.","recovery":{"strategy":"retry_someday"}},"action":"resubscribe"});

	// Act
	let frame = decode_frame(raw);

	// Assert
	core::assert!(core::matches!(
		frame,
		Frame::Malformed { error: super::super::FrameDecodeError::Problem(_), .. }
	));
}
