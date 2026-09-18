#![cfg(test)]

use super::*;

const REQUEST_ID: &str = "10000000-0000-4000-8000-000000000001";

fn manifest() -> serde_json::Value {
	serde_json::from_str(core::include_str!(
		"../../tests/fixtures/problem_manifest.json"
	))
	.unwrap()
}

fn snapshot() -> serde_json::Value {
	serde_json::from_str(core::include_str!("registry_snapshot.json")).unwrap()
}

fn registered_failure(code: &str) -> serde_json::Value {
	let snapshot = snapshot();
	let entry =
		snapshot["codes"].as_array().unwrap().iter().find(|entry| entry["code"] == code).unwrap();
	let strategy = entry["recovery"]["strategy"].as_str().unwrap();
	let recovery = match strategy {
		"retry_after" => serde_json::json!({"strategy": strategy, "after_seconds": 2}),
		"check_status" => {
			serde_json::json!({"strategy": strategy, "resource": {"kind":"batch", "batch_hash":std::format!("0x{}", "11".repeat(32))}})
		},
		_ => serde_json::json!({"strategy": strategy}),
	};
	let mut failure = serde_json::json!({"code":code, "title":entry["title"], "recovery":recovery});

	if let Some(detail) = snapshot["public_details"][code].as_str() {
		failure["detail"] = detail.into();
	}

	failure
}

fn origin(code: &str) -> serde_json::Value {
	let mut value = registered_failure(code);
	let known = KnownProblemCode::from_wire(code).unwrap();
	value["type"] = std::format!("urn:agara:problem:{}", code.replace('_', "-")).into();
	value["status"] = known.http_status().unwrap().into();
	value["request_id"] = REQUEST_ID.into();

	value
}

#[test]
fn canonical_current_and_future_fixtures_enforce_the_published_contract() {
	// Arrange
	let fixture_manifest = manifest();

	for fixture in fixture_manifest["fixtures"].as_array().unwrap() {
		let schema = fixture["schema"].as_str().unwrap();
		let value = fixture["value"].clone();
		let expected = fixture["contract_valid"].as_bool().unwrap();

		// Act
		let actual = match schema {
			"origin-problem-details.schema.json" => {
				ProblemDetails::parse_as(value.clone(), ProblemRepresentation::OriginHttp)
					.map(|_| ())
			},
			"edge-problem.schema.json" => {
				ProblemDetails::parse_as(value.clone(), ProblemRepresentation::EdgeHttp).map(|_| ())
			},
			"public-failure.schema.json" => PublicFailure::parse(value.clone()).map(|_| ()),
			_ => continue,
		};

		// Assert
		core::assert_eq!(actual.is_ok(), expected, "{}: {actual:?}", fixture["name"]);

		if expected && schema == "origin-problem-details.schema.json" {
			let decoded: ProblemDetails = serde_json::from_value(value).unwrap();

			if decoded.code.known().is_none() {
				core::assert!(!decoded.is_retryable());
				core::assert!(core::matches!(decoded.recovery, Recovery::Unknown { .. }));
			}
		}
	}
}

#[test]
fn every_registered_code_retains_its_closed_variant_and_exact_metadata() {
	// Arrange
	let snapshot = snapshot();

	for entry in snapshot["codes"].as_array().unwrap() {
		let code = entry["code"].as_str().unwrap();

		// Act
		let failure = PublicFailure::parse(registered_failure(code)).unwrap();

		// Assert
		core::assert!(failure.is_known());
		core::assert_eq!(failure.code.as_str(), code);
		core::assert_eq!(failure.code.known().unwrap().title(), entry["title"]);
		core::assert_eq!(failure.code.known().unwrap().as_str(), code);

		for representation in entry["representations"].as_array().unwrap() {
			if representation == "origin_http" {
				core::assert!(
					ProblemDetails::parse_as(origin(code), ProblemRepresentation::OriginHttp)
						.is_ok()
				);
			} else if representation == "edge_http" {
				let mut edge = origin(code);
				edge.as_object_mut().unwrap().remove("request_id");
				core::assert!(
					ProblemDetails::parse_as(edge, ProblemRepresentation::EdgeHttp).is_ok()
				);
			}
		}
	}
}

#[rstest::rstest]
#[case("title", serde_json::json!("invented title"), ContractMember::Title)]
#[case("detail", serde_json::json!("private provider diagnostic"), ContractMember::Detail)]
#[case("recovery", serde_json::json!({"strategy":"none"}), ContractMember::Recovery)]
fn known_problem_metadata_mismatches_are_typed(
	#[case] field: &str,
	#[case] bad: serde_json::Value,
	#[case] member: ContractMember,
) {
	// Arrange
	let mut body = origin("settlements_pending");
	body[field] = bad;

	// Act
	let error = ProblemDetails::parse(body).unwrap_err();

	// Assert
	core::assert!(
		core::matches!(error, ProblemDecodeError::RegistryMismatch { code: KnownProblemCode::SettlementsPending, member: actual } if actual == member)
	);
}

#[test]
fn uuid_and_json_parse_errors_retain_original_typed_sources() {
	// Arrange
	let mut body = origin("settlements_pending");
	body["request_id"] = "bad".into();

	// Act
	let uuid_error = ProblemDetails::parse(body).unwrap_err();
	let json_error = ProblemDetails::parse_json(b"{ invalid").unwrap_err();

	// Assert
	core::assert!(core::matches!(
		uuid_error,
		ProblemDecodeError::InvalidUuid { field: ProblemField::RequestId, .. }
	));
	let ProblemDecodeError::Json { source } = json_error else {
		core::panic!("expected JSON parser error")
	};
	core::assert!(source.is_syntax());
	core::assert!(source.column() > 0);
}

#[test]
fn unknown_code_and_strategy_preserve_extensions_without_activating_recovery() {
	// Arrange
	let body = serde_json::json!({"code":"future_failure", "title":"Future failure", "recovery":{"strategy":"retry",}, "future_context":{"opaque":true}});

	// Act
	let failure = PublicFailure::parse(body).unwrap();

	// Assert
	core::assert!(core::matches!(failure.code, ProblemCode::Unknown(_)));
	core::assert!(!failure.is_known());
	core::assert_eq!(failure.websocket_action(), None);
	core::assert_eq!(failure.extensions["future_context"]["opaque"], true);
	core::assert!(!failure.recovery.is_retryable());
	let Recovery::Unknown { raw, .. } = failure.recovery else {
		core::panic!("future recovery must be inert")
	};
	core::assert_eq!(raw["strategy"], "retry");
}

#[rstest::rstest]
#[case(serde_json::json!({"strategy":"retry", "extra":true}), ProblemField::Recovery)]
#[case(serde_json::json!({"strategy":"retry_after", "after_seconds":86401}), ProblemField::AfterSeconds)]
#[case(serde_json::json!({"strategy":"retry_after", "after_seconds":1.5}), ProblemField::AfterSeconds)]
#[case(serde_json::json!({"strategy":"retry_after", "after_seconds":true}), ProblemField::AfterSeconds)]
fn malformed_known_recovery_is_an_error_instead_of_a_fabricated_failure(
	#[case] body: serde_json::Value,
	#[case] _field: ProblemField,
) {
	// Arrange

	// Act
	let decoded = Recovery::parse(body);

	// Assert
	core::assert!(decoded.is_err());
}

#[test]
fn check_status_enforces_the_registered_resource_kind() {
	// Arrange
	let mut body = origin("dependency_outcome_unknown");
	body["recovery"]["resource"] = serde_json::json!({"kind":"order", "order_id":REQUEST_ID});

	// Act
	let error = ProblemDetails::parse(body).unwrap_err();

	// Assert
	core::assert!(core::matches!(
		error,
		ProblemDecodeError::RegistryMismatch { member: ContractMember::ResourceKind, .. }
	));
}

#[test]
fn duplicate_fields_cannot_override_failure_or_recovery_classification() {
	// Arrange
	let inputs = [
		r#"{"code":"future_failure","code":"rate_limited","title":"Future","recovery":{"strategy":"none"}}"#,
		r#"{"code":"future_failure","title":"Future","recovery":{"strategy":"none","strategy":"retry"}}"#,
	];

	for input in inputs {
		// Act
		let decoded = PublicFailure::parse_json(input.as_bytes());
		let via_serde = serde_json::from_str::<PublicFailure>(input);

		// Assert
		core::assert!(core::matches!(
			decoded,
			Err(ProblemDecodeError::Json { .. })
		));
		core::assert!(via_serde.is_err());
	}
}

#[test]
fn typed_field_codes_and_path_bounds_survive_current_responses() {
	// Arrange
	let mut body = origin("validation_failed");
	body["field_errors"] = serde_json::json!([{"path":["orders",0,"price_micro"],"code":"invalid_value","message":"Invalid request value."}]);

	// Act
	let problem = ProblemDetails::parse(body.clone()).unwrap();
	body["field_errors"][0]["path"][1] = serde_json::json!(2147483648_u64);
	let invalid = ProblemDetails::parse(body).unwrap_err();

	// Assert
	core::assert!(core::matches!(
		problem.field_errors[0].code,
		FieldErrorCode::Known(KnownFieldErrorCode::InvalidValue)
	));
	core::assert!(core::matches!(
		invalid,
		ProblemDecodeError::NumberOutOfRange { field: ProblemField::FieldPathSegment, .. }
	));
}

#[test]
fn malformed_public_failure_never_becomes_internal_error() {
	// Arrange
	let bodies = [
		serde_json::json!("legacy private reason"),
		serde_json::json!({"code":"internal_error"}),
	];

	for body in bodies {
		// Act
		let decoded = PublicFailure::parse(body);

		// Assert
		core::assert!(decoded.is_err());
	}
}
