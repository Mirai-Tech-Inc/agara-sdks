mod value;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value};

use super::{
	ContractMember, FieldError, FieldErrorCode, FieldPathSegment, JsonKind, MAX_PROBLEM_BYTES,
	ProblemCode, ProblemDecodeError, ProblemDetails, ProblemField, ProblemRepresentation,
	PublicFailure, Recovery, registry,
};

const MAX_CODE_CHARS: usize = 64;
const MAX_URN_CHARS: usize = 96;
const MAX_TITLE_CHARS: usize = 128;
const MAX_DETAIL_CHARS: usize = 512;
const MAX_FIELD_ERRORS: usize = 32;
const MAX_FIELD_PATH: usize = 16;
const MAX_FIELD_NAME_CHARS: usize = 64;
const MAX_FIELD_MESSAGE_CHARS: usize = 256;
const MAX_FIELD_INDEX: u64 = i32::MAX as u64;
const MIN_HTTP_STATUS: u64 = 400;
const MAX_HTTP_STATUS: u64 = 599;
const UUID_TEXT_LENGTH: usize = 36;
const ORIGIN_FIELDS: &[&str] = &[
	"type",
	"title",
	"status",
	"code",
	"detail",
	"request_id",
	"recovery",
	"field_errors",
];
const EDGE_FIELDS: &[&str] = &["type", "title", "status", "code", "detail", "recovery"];
const FAILURE_FIELDS: &[&str] = &["code", "title", "detail", "recovery"];

pub(super) fn deserialize_value<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<Value, D::Error> {
	value::StrictValue::deserialize(deserializer).map(|value| value.0)
}

pub(super) fn json(bytes: &[u8]) -> Result<Value, ProblemDecodeError> {
	json_with_limit(bytes, MAX_PROBLEM_BYTES)
}

pub(super) fn json_with_limit(bytes: &[u8], limit: usize) -> Result<Value, ProblemDecodeError> {
	length_bound(bytes.len(), limit, ProblemField::Body)?;

	serde_json::from_slice::<value::StrictValue>(bytes)
		.map(|value| value.0)
		.map_err(|source| ProblemDecodeError::Json { source })
}

pub(super) fn public_failure(value: Value) -> Result<PublicFailure, ProblemDecodeError> {
	encoded_bound(&value)?;
	let object = object(&value, ProblemField::Body)?;
	let code = ProblemCode::parse(required_string(
		object,
		"code",
		ProblemField::Code,
		MAX_CODE_CHARS,
	)?)?;
	let title = required_string(object, "title", ProblemField::Title, MAX_TITLE_CHARS)?.to_owned();
	let detail = optional_detail(object)?;
	let raw_recovery = required(object, "recovery", ProblemField::Recovery)?.clone();
	let mut recovery = Recovery::parse(raw_recovery.clone())?;
	validate_failure(&code, &title, detail.as_deref(), &recovery)?;
	let extensions = extensions(object, FAILURE_FIELDS, code.known().is_some())?;

	if code.known().is_none() {
		recovery = recovery.into_inert(raw_recovery);
	}

	Ok(PublicFailure { code, title, detail, recovery, extensions })
}

pub(super) fn problem(
	value: Value,
	representation: ProblemRepresentation,
) -> Result<ProblemDetails, ProblemDecodeError> {
	encoded_bound(&value)?;
	let object = object(&value, ProblemField::Body)?;
	let code = ProblemCode::parse(required_string(
		object,
		"code",
		ProblemField::Code,
		MAX_CODE_CHARS,
	)?)?;
	let type_uri =
		required_string(object, "type", ProblemField::TypeUri, MAX_URN_CHARS)?.to_owned();
	let title = required_string(object, "title", ProblemField::Title, MAX_TITLE_CHARS)?.to_owned();
	let status = required_unsigned(object, "status", ProblemField::Status)?;
	number_range(
		status,
		MIN_HTTP_STATUS,
		MAX_HTTP_STATUS,
		ProblemField::Status,
	)?;
	let detail = optional_detail(object)?;
	let raw_recovery = required(object, "recovery", ProblemField::Recovery)?.clone();
	let mut recovery = Recovery::parse(raw_recovery.clone())?;
	let request_id = match representation {
		ProblemRepresentation::OriginHttp => {
			let id = required_string(
				object,
				"request_id",
				ProblemField::RequestId,
				UUID_TEXT_LENGTH,
			)?;

			Some(uuid(id, ProblemField::RequestId)?)
		},
		ProblemRepresentation::EdgeHttp => {
			if object.contains_key("request_id") || object.contains_key("field_errors") {
				return Err(ProblemDecodeError::unexpected_field(
					if object.contains_key("request_id") {
						"request_id"
					} else {
						"field_errors"
					},
				));
			}

			None
		},
	};
	let field_errors = if let Some(value) = object.get("field_errors") {
		let fields = value.as_array().ok_or(ProblemDecodeError::WrongType {
			field: ProblemField::FieldErrors,
			expected: JsonKind::Array,
		})?;
		length_bound(fields.len(), MAX_FIELD_ERRORS, ProblemField::FieldErrors)?;
		fields.iter().cloned().map(field_error).collect::<Result<Vec<_>, _>>()?
	} else {
		Vec::new()
	};
	let allowed = match representation {
		ProblemRepresentation::OriginHttp => ORIGIN_FIELDS,
		ProblemRepresentation::EdgeHttp => EDGE_FIELDS,
	};
	let extensions = extensions(object, allowed, code.known().is_some())?;

	if code.known().is_none() {
		recovery = recovery.into_inert(raw_recovery);
	}

	let problem = ProblemDetails {
		type_uri,
		title,
		status: status as u16,
		code,
		detail,
		request_id,
		recovery,
		field_errors,
		representation,
		extensions,
	};
	validate_problem(&problem)?;

	Ok(problem)
}

pub(super) fn validate_failure(
	code: &ProblemCode,
	title: &str,
	detail: Option<&str>,
	recovery: &Recovery,
) -> Result<(), ProblemDecodeError> {
	bounded_text(title, MAX_TITLE_CHARS, ProblemField::Title)?;

	if let Some(detail) = detail {
		bounded_text(detail, MAX_DETAIL_CHARS, ProblemField::Detail)?;
	}

	recovery.validate()?;

	if let Some(code) = code.known() {
		let metadata = registry::metadata(code);
		let mismatch = if title != metadata.title {
			Some(ContractMember::Title)
		} else if detail != metadata.detail {
			Some(ContractMember::Detail)
		} else if recovery.known_strategy() != Some(metadata.recovery) {
			Some(ContractMember::Recovery)
		} else if recovery.resource_kind() != metadata.resource {
			Some(ContractMember::ResourceKind)
		} else {
			None
		};

		if let Some(member) = mismatch {
			return Err(ProblemDecodeError::RegistryMismatch { code, member });
		}
	}

	Ok(())
}

pub(super) fn validate_problem(problem: &ProblemDetails) -> Result<(), ProblemDecodeError> {
	validate_failure(
		&problem.code,
		&problem.title,
		problem.detail.as_deref(),
		&problem.recovery,
	)?;
	number_range(
		u64::from(problem.status),
		MIN_HTTP_STATUS,
		MAX_HTTP_STATUS,
		ProblemField::Status,
	)?;
	let expected_urn = std::format!(
		"urn:agara:problem:{}",
		problem.code.as_str().replace('_', "-")
	);

	if problem.type_uri != expected_urn {
		return Err(ProblemDecodeError::InvalidUrn);
	}

	match (problem.representation, problem.request_id) {
		(ProblemRepresentation::OriginHttp, None) => {
			return Err(ProblemDecodeError::MissingField { field: ProblemField::RequestId });
		},
		(ProblemRepresentation::OriginHttp, Some(id)) => {
			uuid(&id.to_string(), ProblemField::RequestId)?;
		},
		(ProblemRepresentation::EdgeHttp, Some(_)) => {
			return Err(ProblemDecodeError::unexpected_field("request_id"));
		},
		(ProblemRepresentation::EdgeHttp, None) if !problem.field_errors.is_empty() => {
			return Err(ProblemDecodeError::unexpected_field("field_errors"));
		},
		_ => {},
	}

	length_bound(
		problem.field_errors.len(),
		MAX_FIELD_ERRORS,
		ProblemField::FieldErrors,
	)?;

	for field in &problem.field_errors {
		validate_field_error(field)?;
	}

	if let Some(code) = problem.code.known() {
		let metadata = registry::metadata(code);
		let supported = match problem.representation {
			ProblemRepresentation::OriginHttp => metadata.origin,
			ProblemRepresentation::EdgeHttp => metadata.edge,
		};

		if !supported {
			return Err(ProblemDecodeError::RegistryMismatch {
				code,
				member: ContractMember::Representation,
			});
		}

		if metadata.status != Some(problem.status) {
			return Err(ProblemDecodeError::RegistryMismatch {
				code,
				member: ContractMember::Status,
			});
		}

		if let Some((name, _)) = problem.extensions.first_key_value() {
			return Err(ProblemDecodeError::unexpected_field(name.clone()));
		}
	}

	Ok(())
}

pub(super) fn field_error(value: Value) -> Result<FieldError, ProblemDecodeError> {
	let object = object(&value, ProblemField::FieldError)?;
	exact_fields(object, &["path", "code", "message"])?;
	let code = FieldErrorCode::parse(required_string(
		object,
		"code",
		ProblemField::FieldErrorCode,
		MAX_CODE_CHARS,
	)?)?;
	let message = required_string(
		object,
		"message",
		ProblemField::FieldErrorMessage,
		MAX_FIELD_MESSAGE_CHARS,
	)?
	.to_owned();
	let path = required(object, "path", ProblemField::FieldPath)?.as_array().ok_or(
		ProblemDecodeError::WrongType { field: ProblemField::FieldPath, expected: JsonKind::Array },
	)?;
	length_bound(path.len(), MAX_FIELD_PATH, ProblemField::FieldPath)?;
	let path = path
		.iter()
		.map(|value| {
			if let Some(name) = value.as_str() {
				bounded_text(name, MAX_FIELD_NAME_CHARS, ProblemField::FieldPathSegment)?;

				Ok(FieldPathSegment::Field(name.to_owned()))
			} else {
				let index = value.as_u64().ok_or(ProblemDecodeError::WrongType {
					field: ProblemField::FieldPathSegment,
					expected: JsonKind::UnsignedInteger,
				})?;
				number_range(index, 0, MAX_FIELD_INDEX, ProblemField::FieldPathSegment)?;

				Ok(FieldPathSegment::Index(index as u32))
			}
		})
		.collect::<Result<Vec<_>, ProblemDecodeError>>()?;

	Ok(FieldError { path, code, message })
}

pub(super) fn code_name(value: &str, field: ProblemField) -> Result<(), ProblemDecodeError> {
	let mut bytes = value.bytes();
	let valid = bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
		&& value.len() <= MAX_CODE_CHARS
		&& bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');

	if !valid {
		return Err(ProblemDecodeError::InvalidCode { field });
	}

	Ok(())
}

pub(super) fn uuid(value: &str, field: ProblemField) -> Result<uuid::Uuid, ProblemDecodeError> {
	let parsed = uuid::Uuid::parse_str(value)
		.map_err(|source| ProblemDecodeError::InvalidUuid { field, source })?;
	let hyphenated = value.len() == UUID_TEXT_LENGTH
		&& value.bytes().enumerate().all(|(index, byte)| {
			if core::matches!(index, 8 | 13 | 18 | 23) {
				byte == b'-'
			} else {
				byte.is_ascii_hexdigit()
			}
		});
	let sentinel = parsed.is_nil() || parsed.as_bytes().iter().all(|byte| *byte == u8::MAX);
	let rfc = (1..=8).contains(&parsed.get_version_num())
		&& parsed.get_variant() == uuid::Variant::RFC4122;

	if !hyphenated || (!sentinel && !rfc) {
		return Err(ProblemDecodeError::NonCanonicalUuid { field });
	}

	Ok(parsed)
}

pub(super) fn object(
	value: &Value,
	field: ProblemField,
) -> Result<&Map<String, Value>, ProblemDecodeError> {
	value.as_object().ok_or(ProblemDecodeError::WrongType { field, expected: JsonKind::Object })
}

pub(super) fn required<'a>(
	object: &'a Map<String, Value>,
	key: &str,
	field: ProblemField,
) -> Result<&'a Value, ProblemDecodeError> {
	object.get(key).ok_or(ProblemDecodeError::MissingField { field })
}

pub(super) fn required_string<'a>(
	object: &'a Map<String, Value>,
	key: &str,
	field: ProblemField,
	maximum: usize,
) -> Result<&'a str, ProblemDecodeError> {
	let value = required(object, key, field)?
		.as_str()
		.ok_or(ProblemDecodeError::WrongType { field, expected: JsonKind::String })?;
	bounded_text(value, maximum, field)?;

	Ok(value)
}

pub(super) fn required_unsigned(
	object: &Map<String, Value>,
	key: &str,
	field: ProblemField,
) -> Result<u64, ProblemDecodeError> {
	required(object, key, field)?
		.as_u64()
		.ok_or(ProblemDecodeError::WrongType { field, expected: JsonKind::UnsignedInteger })
}

pub(super) fn length_bound(
	actual: usize,
	limit: usize,
	field: ProblemField,
) -> Result<(), ProblemDecodeError> {
	if actual > limit {
		return Err(ProblemDecodeError::LimitExceeded { field, limit, actual });
	}

	Ok(())
}

pub(super) fn number_range(
	actual: u64,
	min: u64,
	max: u64,
	field: ProblemField,
) -> Result<(), ProblemDecodeError> {
	if !(min..=max).contains(&actual) {
		return Err(ProblemDecodeError::NumberOutOfRange { field, min, max, actual });
	}

	Ok(())
}

pub(super) fn exact_fields(
	object: &Map<String, Value>,
	allowed: &[&str],
) -> Result<(), ProblemDecodeError> {
	if let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
		return Err(ProblemDecodeError::unexpected_field(key.clone()));
	}

	Ok(())
}

fn bounded_text(
	value: &str,
	maximum: usize,
	field: ProblemField,
) -> Result<(), ProblemDecodeError> {
	if value.is_empty() {
		return Err(ProblemDecodeError::EmptyText { field });
	}

	length_bound(value.chars().count(), maximum, field)
}

fn optional_detail(object: &Map<String, Value>) -> Result<Option<String>, ProblemDecodeError> {
	if object.contains_key("detail") {
		return required_string(object, "detail", ProblemField::Detail, MAX_DETAIL_CHARS)
			.map(|value| Some(value.to_owned()));
	}

	Ok(None)
}

fn extensions(
	object: &Map<String, Value>,
	allowed: &[&str],
	known: bool,
) -> Result<BTreeMap<String, Value>, ProblemDecodeError> {
	if known {
		exact_fields(object, allowed)?;
	}

	Ok(object
		.iter()
		.filter(|(key, _)| !allowed.contains(&key.as_str()))
		.map(|(key, value)| (key.clone(), value.clone()))
		.collect())
}

fn encoded_bound(value: &Value) -> Result<(), ProblemDecodeError> {
	let bytes = serde_json::to_vec(value).map_err(|source| ProblemDecodeError::Json { source })?;

	length_bound(bytes.len(), MAX_PROBLEM_BYTES, ProblemField::Body)
}

fn validate_field_error(field: &FieldError) -> Result<(), ProblemDecodeError> {
	length_bound(field.path.len(), MAX_FIELD_PATH, ProblemField::FieldPath)?;
	code_name(field.code.as_str(), ProblemField::FieldErrorCode)?;
	bounded_text(
		&field.message,
		MAX_FIELD_MESSAGE_CHARS,
		ProblemField::FieldErrorMessage,
	)?;

	for segment in &field.path {
		match segment {
			FieldPathSegment::Field(name) => {
				bounded_text(name, MAX_FIELD_NAME_CHARS, ProblemField::FieldPathSegment)?
			},
			FieldPathSegment::Index(index) => number_range(
				u64::from(*index),
				0,
				MAX_FIELD_INDEX,
				ProblemField::FieldPathSegment,
			)?,
		}
	}

	Ok(())
}
