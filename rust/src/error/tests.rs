#![cfg(test)]

use crate::{
	problem::{ContractMember, KnownProblemCode, ProblemCode, ProblemDecodeError},
	validation::{Field, IdentifierReason, ValidationError},
};

use super::*;

fn problem(code: KnownProblemCode, recovery: serde_json::Value) -> ProblemDetails {
	let mut value = serde_json::json!({
		"type":std::format!("urn:agara:problem:{}", code.as_str().replace('_', "-")),
		"title":code.title(), "status":code.http_status().unwrap(), "code":code.as_str(),
		"request_id":"10000000-0000-4000-8000-000000000001", "recovery":recovery,
	});

	if let Some(detail) = code.public_detail() {
		value["detail"] = detail.into();
	}

	ProblemDetails::parse(value).unwrap()
}

#[test]
fn typed_recovery_controls_retry_instead_of_http_status() {
	// Arrange
	let terminal = problem(
		KnownProblemCode::InternalError,
		serde_json::json!({"strategy":"none"}),
	);
	let transient = problem(
		KnownProblemCode::PnlNotReady,
		serde_json::json!({"strategy":"retry"}),
	);

	// Act
	let terminal = AgaraError::from_response(500, serde_json::json!({}), None, Some(terminal));
	let transient = AgaraError::from_response(425, serde_json::json!({}), None, Some(transient));

	// Assert
	core::assert!(!terminal.is_retryable());
	core::assert!(transient.is_retryable());
	core::assert_eq!(
		transient.problem().unwrap().code,
		ProblemCode::Known(KnownProblemCode::PnlNotReady)
	);
}

#[test]
fn http_status_mismatch_cannot_activate_registered_retry_advice() {
	// Arrange
	let problem = problem(
		KnownProblemCode::PnlNotReady,
		serde_json::json!({"strategy":"retry"}),
	);
	let body = serde_json::json!({"preserved":"evidence"});

	// Act
	let error = AgaraError::from_response(500, body.clone(), None, Some(problem));

	// Assert
	core::assert!(!error.is_retryable());
	core::assert_eq!(error.status_code(), Some(500));
	core::assert!(
		core::matches!(error, AgaraError::Response(ResponseError::Problem { status:500, body:actual, source:ProblemDecodeError::HttpStatusMismatch { actual:500, declared:425 } }) if actual == body)
	);
}

#[test]
fn mutated_known_problem_metadata_is_rejected_before_http_classification() {
	// Arrange
	let mut problem = problem(
		KnownProblemCode::PnlNotReady,
		serde_json::json!({"strategy":"retry"}),
	);
	problem.title = "unregistered title".to_owned();

	// Act
	let error = AgaraError::from_response(425, serde_json::json!({}), None, Some(problem));

	// Assert
	core::assert!(core::matches!(
		error,
		AgaraError::Response(ResponseError::Problem {
			source: ProblemDecodeError::RegistryMismatch { member: ContractMember::Title, .. },
			..
		})
	));
}

#[test]
fn unknown_http_status_retains_the_entire_response() {
	// Arrange
	let body = serde_json::json!({"new_failure":{"opaque":[1,2,3]}});

	// Act
	let error = AgaraError::from_response(
		418,
		body.clone(),
		Some(RetryAfter::Delay(Duration::from_secs(2))),
		None,
	);

	// Assert
	core::assert!(core::matches!(
		error,
		AgaraError::UnexpectedStatus { status: 418, .. }
	));
	core::assert_eq!(error.http_failure().unwrap().body(), &body);
	core::assert_eq!(error.retry_after(), Some(Duration::from_secs(2)));
	core::assert!(!error.is_retryable());
}

#[rstest::rstest]
#[case(400)]
#[case(401)]
#[case(403)]
#[case(404)]
#[case(405)]
#[case(409)]
#[case(410)]
#[case(413)]
#[case(415)]
#[case(422)]
#[case(424)]
#[case(425)]
#[case(426)]
#[case(429)]
#[case(503)]
fn recognized_http_statuses_keep_dedicated_classes(#[case] status: u16) {
	// Arrange
	let body = serde_json::json!({"error":"legacy response"});

	// Act
	let error = AgaraError::from_response(status, body.clone(), None, None);

	// Assert
	core::assert_eq!(error.status_code(), Some(status));
	core::assert_eq!(error.http_failure().unwrap().body(), &body);
	core::assert!(!core::matches!(error, AgaraError::UnexpectedStatus { .. }));
	core::assert!(!error.is_retryable());
}

#[test]
fn json_source_is_not_replaced_by_a_display_string() {
	// Arrange
	let source = serde_json::from_str::<serde_json::Value>("{ invalid").unwrap_err();

	// Act
	let error = AgaraError::from(ResponseError::Json { source });

	// Assert
	let source = std::error::Error::source(&error).unwrap();
	core::assert!(source.downcast_ref::<serde_json::Error>().unwrap().is_syntax());
}

#[test]
fn validation_errors_are_matchable_by_family_field_and_reason() {
	// Arrange
	let source = uuid::Uuid::parse_str("invalid").unwrap_err();
	let validation =
		ValidationError::identifier(Field::OrderId, IdentifierReason::InvalidUuid { source });

	// Act
	let error = AgaraError::from(validation);

	// Assert
	core::assert!(core::matches!(
		error,
		AgaraError::Validation(ValidationError::Identifier(
			crate::validation::IdentifierError {
				field: Field::OrderId,
				reason: IdentifierReason::InvalidUuid { .. }
			}
		))
	));
}

#[test]
fn uuid_validation_preserves_the_original_parser_error() {
	// Arrange

	// Act
	let error = crate::ids::OrderId::new("not-a-uuid").unwrap_err();

	// Assert
	let ValidationError::Identifier(crate::validation::IdentifierError {
		field: Field::OrderId,
		reason: IdentifierReason::InvalidUuid { source },
	}) = error
	else {
		core::panic!("expected typed UUID source")
	};
	core::assert!(!source.to_string().is_empty());
}

#[test]
fn date_field_retargeting_preserves_chrono_source_and_query_category() {
	// Arrange

	// Act
	let error = crate::input::calendar("2026-02-29", Field::From).unwrap_err();

	// Assert
	let ValidationError::Query(crate::validation::QueryError {
		field: Field::From,
		reason: crate::validation::QueryReason::InvalidDate { source: Some(source) },
	}) = error
	else {
		core::panic!("expected retained calendar parser source")
	};
	core::assert_eq!(source.kind(), chrono::format::ParseErrorKind::OutOfRange);
}

#[test]
fn timestamp_field_retargeting_preserves_chrono_source() {
	// Arrange

	// Act
	let error = crate::input::timestamp("2026-09-18T00:00:00", Field::To).unwrap_err();

	// Assert
	let ValidationError::Query(crate::validation::QueryError {
		field: Field::To,
		reason: crate::validation::QueryReason::InvalidTimestamp { source },
	}) = error
	else {
		core::panic!("expected retained timestamp parser source")
	};
	core::assert_eq!(source.kind(), chrono::format::ParseErrorKind::TooShort);
}
