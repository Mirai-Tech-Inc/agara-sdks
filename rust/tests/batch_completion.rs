//! Completion evidence and forward-compatible batch classification invariants.

use agara_sdk::batches::{
	AccountBatchStatusDto, BatchField, BatchGroupStatusDto, BatchOrigin, BatchStatus,
	BatchValidationError, FutureBatchOrigin, FutureBatchStatus, WireValueError,
};

const HASH: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SECOND_HASH: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const THIRD_HASH: &str = "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const GROUP: &str = "10000000-0000-4000-8000-000000000001";
const TIMESTAMP: &str = "2026-09-17T00:00:00Z";

fn failure() -> serde_json::Value {
	serde_json::json!({"code":"internal_error","title":"Something went wrong","detail":"We hit an unexpected problem. Please try again in a moment.","recovery":{"strategy":"none"}})
}

fn settled() -> AccountBatchStatusDto {
	serde_json::from_value(serde_json::json!({
		"batch_hash":HASH,"status":"SETTLED","seq":4,"deadline_unix_seconds":1800000000,
		"origin":"PRESIGNED","tx_hash":HASH,"executed_at":TIMESTAMP,"created_at":TIMESTAMP,
	}))
	.unwrap()
}

fn group() -> BatchGroupStatusDto {
	serde_json::from_value(serde_json::json!({
		"group_id":GROUP,"op_count":21,"chunk_count":2,"completed_at":TIMESTAMP,"as_of":TIMESTAMP,
		"chunks":[
			{"chunk_index":0,"batch_hash":HASH,"status":"FAILED","tx_hash":null,"failure":failure()},
			{"chunk_index":0,"batch_hash":SECOND_HASH,"status":"SETTLED","tx_hash":HASH,"failure":null},
			{"chunk_index":1,"batch_hash":THIRD_HASH,"status":"SETTLED","tx_hash":HASH,"failure":null},
		],
	}))
	.unwrap()
}

#[test]
fn settled_label_without_each_confirmed_marker_never_completes() {
	// Arrange
	let valid = settled();
	let mut missing_transaction = valid.clone();
	missing_transaction.tx_hash = None;
	let mut missing_execution = valid.clone();
	missing_execution.executed_at = None;
	let mut conflicting_failure = valid.clone();
	conflicting_failure.failure = Some(serde_json::from_value(failure()).unwrap());

	// Act
	let complete = valid.is_terminal();
	let missing_transaction = missing_transaction.is_terminal();
	let missing_execution = missing_execution.is_terminal();
	let conflicting_failure = conflicting_failure.is_terminal();

	// Assert
	core::assert!(complete);
	core::assert!(!missing_transaction);
	core::assert!(!missing_execution);
	core::assert!(!conflicting_failure);
}

#[test]
fn failed_completion_requires_current_producer_failure_and_clean_unwind() {
	// Arrange
	let mut batch = settled();
	batch.status = BatchStatus::Failed;
	batch.tx_hash = None;
	batch.executed_at = None;
	batch.failure = Some(serde_json::from_value(failure()).unwrap());

	// Act
	let unwinding = batch.is_terminal();
	batch.unwound_at = Some(agara_sdk::values::Timestamp::new(TIMESTAMP).unwrap());
	let unwound = batch.is_terminal();
	batch.failure = None;
	let incomplete_failure = batch.is_terminal();
	batch.status = BatchStatus::FailedDivergent;
	batch.unwound_at = None;
	let incomplete_divergence = batch.is_terminal();
	batch.failure = Some(serde_json::from_value(failure()).unwrap());
	let divergent = batch.is_terminal();

	// Assert
	core::assert!(!unwinding);
	core::assert!(unwound);
	core::assert!(!incomplete_failure);
	core::assert!(!incomplete_divergence);
	core::assert!(divergent);
}

#[rstest::rstest]
#[case("PENDING")]
#[case("SUBMITTED")]
#[case("SETTLED")]
#[case("FAILED")]
#[case("FAILED_DIVERGENT")]
fn known_status_cannot_hide_inside_unknown_payload(#[case] known: &str) {
	// Arrange
	let json = serde_json::to_string(known).unwrap();

	// Act
	let constructed = FutureBatchStatus::new(known);
	let deserialized = serde_json::from_str::<FutureBatchStatus>(&json);

	// Assert
	core::assert!(core::matches!(
		constructed,
		Err(BatchValidationError::Field {
			field: BatchField::Status,
			source: WireValueError::KnownClassification
		})
	));
	core::assert!(deserialized.is_err());
	core::assert!(!core::matches!(
		BatchStatus::new(known).unwrap(),
		BatchStatus::Unknown(_)
	));
}

#[rstest::rstest]
#[case("PRIVY")]
#[case("PRESIGNED")]
fn known_origin_cannot_hide_inside_unknown_payload(#[case] known: &str) {
	// Arrange
	let json = serde_json::to_string(known).unwrap();

	// Act
	let constructed = FutureBatchOrigin::new(known);
	let deserialized = serde_json::from_str::<FutureBatchOrigin>(&json);

	// Assert
	core::assert!(core::matches!(
		constructed,
		Err(BatchValidationError::Field {
			field: BatchField::Origin,
			source: WireValueError::KnownClassification
		})
	));
	core::assert!(deserialized.is_err());
}

#[rstest::rstest]
#[case("")]
#[case("future")]
#[case("FUTURE VALUE")]
#[case("9FUTURE")]
#[case("FUTURE_é")]
#[case("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")]
fn future_classification_constructors_and_serde_reject_invalid_text(#[case] value: &str) {
	// Arrange
	let json = serde_json::to_string(value).unwrap();

	// Act
	let status = FutureBatchStatus::new(value);
	let origin = FutureBatchOrigin::new(value);
	let status_json = serde_json::from_str::<FutureBatchStatus>(&json);
	let origin_json = serde_json::from_str::<FutureBatchOrigin>(&json);

	// Assert
	core::assert!(status.is_err());
	core::assert!(origin.is_err());
	core::assert!(status_json.is_err());
	core::assert!(origin_json.is_err());
}

#[test]
fn future_classifications_roundtrip_without_changing_semantics() {
	// Arrange
	let status = BatchStatus::Unknown(FutureBatchStatus::new("FUTURE_DONE").unwrap());
	let origin = BatchOrigin::Unknown(FutureBatchOrigin::new("FUTURE_AUTHORITY").unwrap());
	let mut snapshot = settled();
	snapshot.status = status.clone();

	// Act
	let status_roundtrip: BatchStatus =
		serde_json::from_str(&serde_json::to_string(&status).unwrap()).unwrap();
	let origin_roundtrip: BatchOrigin =
		serde_json::from_str(&serde_json::to_string(&origin).unwrap()).unwrap();

	// Assert
	core::assert_eq!(status, status_roundtrip);
	core::assert_eq!(origin, origin_roundtrip);
	core::assert!(!snapshot.is_terminal());
}

#[test]
fn completed_group_allows_prior_failed_attempts_and_requires_each_settled_slot() {
	// Arrange
	let valid = group();
	let mut missing_slot = valid.clone();
	missing_slot.chunks.pop();
	let mut missing_proof = valid.clone();
	missing_proof.chunks[2].tx_hash = None;
	let mut duplicate_slot = valid.clone();
	duplicate_slot.chunks[2].chunk_index = 0;
	let mut unresolved_divergence = valid.clone();
	unresolved_divergence.chunks[0].status = BatchStatus::FailedDivergent;
	let mut duplicate_identity = valid.clone();
	duplicate_identity.chunks[2].batch_hash = valid.chunks[1].batch_hash.clone();
	let mut unconfirmed = valid.clone();
	unconfirmed.chunk_count = None;

	// Act
	let complete = valid.is_complete();

	// Assert
	core::assert!(complete);
	core::assert!(!missing_slot.is_complete());
	core::assert!(!missing_proof.is_complete());
	core::assert!(!duplicate_slot.is_complete());
	core::assert!(!unconfirmed.is_complete());
	core::assert!(!duplicate_identity.is_complete());
	core::assert!(!unresolved_divergence.is_complete());
}

#[test]
fn confirmed_empty_group_is_complete_without_invented_chunk_requirements() {
	// Arrange
	let empty: BatchGroupStatusDto = serde_json::from_value(serde_json::json!({
        "group_id":GROUP,"op_count":0,"chunk_count":0,"completed_at":TIMESTAMP,"as_of":TIMESTAMP,"chunks":[],
    })).unwrap();
	let mut not_closed = empty.clone();
	not_closed.completed_at = None;

	// Act
	let complete = empty.is_complete();

	// Assert
	core::assert!(complete);
	core::assert!(!not_closed.is_complete());
}
