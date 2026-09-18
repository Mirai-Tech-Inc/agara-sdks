use core::fmt;

use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{ProblemDecodeError, ProblemField, decode, registry};

/// Every problem code registered by the pinned platform contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KnownProblemCode {
	/// Account frozen. Trading on your account is paused right now. Contact support to find out more.
	AccountFrozen,

	/// Account quarantined. Your account is under review. Contact support for more details.
	AccountQuarantined,

	/// Something is still processing. An earlier split or merge is still going through. Please wait a
	/// moment and try again.
	BatchInFlight,

	/// Batch not found.
	BatchNotFound,

	/// Batch superseded. A newer batch replaced this batch.
	BatchSuperseded,

	/// Category not found.
	CategoryNotFound,

	/// Chain execution failed. Transaction failed, please retry.
	ChainExecutionFailed,

	/// Credentials required.
	CredentialsMissing,

	/// Dependency credentials rejected. Third-party service authorization failed, please try again
	/// later.
	DependencyCredentialsRejected,

	/// Dependency returned an invalid response. Third-party service returned an invalid response,
	/// please try again later.
	DependencyInvalidResponse,

	/// Submission status is uncertain. Your request may have succeeded. Check your positions before
	/// trying again.
	DependencyOutcomeUnknown,

	/// Dependency temporarily unavailable. A third-party service is down, please try again later.
	DependencyUnavailable,

	/// Duplicate order. An order with this identity already exists.
	DuplicateOrder,

	/// Endpoint retired. This endpoint is no longer available.
	EndpointRetired,

	/// Event not found.
	EventNotFound,

	/// This feature is not available here. This option is not turned on in this environment.
	FeatureNotConfigured,

	/// Permission denied.
	ForbiddenScope,

	/// Batch group not found.
	GroupNotFound,

	/// Your session has expired. Please log in again to continue.
	IdentityTokenExpired,

	/// Insufficient balance. Available balance is lower than the order value.
	InsufficientBalance,

	/// Insufficient shares. Available shares are lower than the requested amount.
	InsufficientShares,

	/// Something went wrong. We hit an unexpected problem. Please try again in a moment.
	InternalError,

	/// Invalid batch. The batch does not satisfy submission requirements.
	InvalidBatch,

	/// Invalid identity token.
	InvalidIdentityToken,

	/// Invalid WebSocket message. The message is not valid for this connection.
	InvalidMessage,

	/// Invalid path parameter. One or more path parameters are invalid.
	InvalidPathParameter,

	/// Invalid personal access token.
	InvalidPersonalAccessToken,

	/// Invalid service credential.
	InvalidServiceCredential,

	/// Invalid signature. The submitted signature could not be verified.
	InvalidSignature,

	/// LP payout cannot be resumed. This LP payout run cannot be resumed from its current state.
	LpPayoutNotRecoverable,

	/// LP payout run not found. The requested LP payout run does not exist.
	LpPayoutRunNotFound,

	/// Malformed JSON. The request body is not valid JSON.
	MalformedJson,

	/// Malformed WebSocket message. The WebSocket message is not valid JSON.
	MalformedWebsocketMessage,

	/// Market closed. Market is not accepting orders.
	MarketClosed,

	/// Market not found.
	MarketNotFound,

	/// Method not allowed. This route does not support the requested method.
	MethodNotAllowed,

	/// Order expiration too soon. Good-til-date orders must expire at least 30 seconds from now.
	OrderExpirationTooSoon,

	/// Order funding failed. Order not successful, please try again.
	OrderFundingFailed,

	/// Order cannot be cancelled. The order is no longer cancellable.
	OrderNotCancellable,

	/// Order cannot be filled. Order cannot be filled right now.
	OrderNotFillable,

	/// Order not found.
	OrderNotFound,

	/// Order price out of range. Limit price must be above 0 and below 1 collateral unit.
	OrderPriceOutOfRange,

	/// Order size above maximum. Maximum market sell size is 100,000 shares.
	OrderSizeAboveMaximum,

	/// Order size below minimum. Minimum market sell size is 1 share.
	OrderSizeBelowMinimum,

	/// Order submission failed. Order could not be submitted, please try again later.
	OrderSubmissionFailed,

	/// Order value above maximum. Maximum order value is $100,000.
	OrderValueAboveMaximum,

	/// Order value below minimum. Minimum order value is $0.10.
	OrderValueBelowMinimum,

	/// PnL not ready. PnL is not available yet.
	PnlNotReady,

	/// PnL unavailable. PnL is unavailable for this wallet.
	PnlUnavailable,

	/// Post-only order would cross. The order would execute immediately instead of resting.
	PostOnlyWouldCross,

	/// Price feed not found. No price feed is configured for the requested security and provider.
	PriceFeedNotFound,

	/// Price service not configured. The price service is not configured on this deployment.
	PriceServiceNotConfigured,

	/// Price stream capacity exceeded. The shared price-stream symbol limit has been reached.
	PriceStreamCapacityExceeded,

	/// Price stream not configured. Price streaming is not configured on this deployment.
	PriceStreamNotConfigured,

	/// Price temporarily unavailable. The requested price is not available yet.
	PriceTemporarilyUnavailable,

	/// Profile not found.
	ProfileNotFound,

	/// Too many requests. You are rate limited, please try again later.
	RateLimited,

	/// Request body too large.
	RequestBodyTooLarge,

	/// Route not found.
	RouteNotFound,

	/// Security not found.
	SecurityNotFound,

	/// Sequence conflict. Order submission failed, please retry.
	SequenceConflict,

	/// Server restarting. The server is restarting. Reconnect after a short delay.
	ServerRestarting,

	/// Settlement requires review. Settlement effects do not match the expected operation set.
	SettlementDivergent,

	/// Settlements pending. Fills for this market can still settle; payouts are held until they
	/// drain.
	SettlementsPending,

	/// Finish wallet setup. Complete the wallet approval step, then try again.
	SignerAuthorizationRequired,

	/// Client is reading too slowly. The connection was closed because messages were not read quickly
	/// enough.
	SlowConsumer,

	/// Stream subject not found.
	StreamSubjectNotFound,

	/// Stream subject temporarily unavailable. This stream subject is temporarily unavailable.
	StreamSubjectUnavailable,

	/// Subscription limit exceeded.
	SubscriptionLimitExceeded,

	/// Token not found.
	TokenNotFound,

	/// Trading day not found. No trading-day coverage exists for the requested date.
	TradingDayNotFound,

	/// Unsupported media type. Send the request body as application/json.
	UnsupportedMediaType,

	/// Unsupported WebSocket data. Send WebSocket messages as JSON text frames.
	UnsupportedWebsocketData,

	/// Request validation failed. One or more fields are invalid.
	ValidationFailed,

	/// Value cap exceeded. Order value exceeds the configured maximum cap.
	ValueCapExceeded,

	/// Venue not found.
	VenueNotFound,

	/// Wallet not registered. Finish wallet registration and then retry.
	WalletNotRegistered,

	/// Wallet setup failed. Wallet setup did not complete successfully.
	WalletSetupFailed,

	/// Wallet setup incomplete. Finish wallet registration and then retry.
	WalletSetupIncomplete,

	/// Wallet setup outcome is uncertain. Wallet setup may have been submitted and requires
	/// reconciliation.
	WalletSetupOutcomeUnknown,

	/// WebSocket message too large.
	WebsocketMessageTooLarge,

	/// WebSocket protocol error. The WebSocket protocol state is invalid.
	WebsocketProtocolError,

	/// WebSocket upgrade required. This route requires a WebSocket upgrade.
	WebsocketUpgradeRequired,

	/// Wrong credential type.
	WrongCredentialType,
}

/// A syntactically valid future code, retained without enabling automatic recovery.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FutureCode(Cow<'static, str>);

/// Machine-readable registered code or a bounded future extension.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProblemCode {
	/// A registered problem with canonical metadata in this SDK release.
	Known(KnownProblemCode),

	/// A future code whose recovery advice is always inert.
	Unknown(FutureCode),
}

/// Registered request-field validation classifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnownFieldErrorCode {
	/// A supplied field violates its request schema.
	InvalidValue,

	/// Further field failures were omitted because the bounded list was full.
	AdditionalErrors,
}

/// Registered or future classification of one request-field error.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FieldErrorCode {
	/// A field-error classification whose message is fixed by the platform.
	Known(KnownFieldErrorCode),

	/// Future field-error code, retained for diagnostics only.
	Unknown(FutureCode),
}

impl KnownProblemCode {
	/// Parse a registered wire name without interpreting future names as a known failure.
	pub fn from_wire(value: &str) -> Option<Self> {
		match value {
			"account_frozen" => Some(Self::AccountFrozen),
			"account_quarantined" => Some(Self::AccountQuarantined),
			"batch_in_flight" => Some(Self::BatchInFlight),
			"batch_not_found" => Some(Self::BatchNotFound),
			"batch_superseded" => Some(Self::BatchSuperseded),
			"category_not_found" => Some(Self::CategoryNotFound),
			"chain_execution_failed" => Some(Self::ChainExecutionFailed),
			"credentials_missing" => Some(Self::CredentialsMissing),
			"dependency_credentials_rejected" => Some(Self::DependencyCredentialsRejected),
			"dependency_invalid_response" => Some(Self::DependencyInvalidResponse),
			"dependency_outcome_unknown" => Some(Self::DependencyOutcomeUnknown),
			"dependency_unavailable" => Some(Self::DependencyUnavailable),
			"duplicate_order" => Some(Self::DuplicateOrder),
			"endpoint_retired" => Some(Self::EndpointRetired),
			"event_not_found" => Some(Self::EventNotFound),
			"feature_not_configured" => Some(Self::FeatureNotConfigured),
			"forbidden_scope" => Some(Self::ForbiddenScope),
			"group_not_found" => Some(Self::GroupNotFound),
			"identity_token_expired" => Some(Self::IdentityTokenExpired),
			"insufficient_balance" => Some(Self::InsufficientBalance),
			"insufficient_shares" => Some(Self::InsufficientShares),
			"internal_error" => Some(Self::InternalError),
			"invalid_batch" => Some(Self::InvalidBatch),
			"invalid_identity_token" => Some(Self::InvalidIdentityToken),
			"invalid_message" => Some(Self::InvalidMessage),
			"invalid_path_parameter" => Some(Self::InvalidPathParameter),
			"invalid_personal_access_token" => Some(Self::InvalidPersonalAccessToken),
			"invalid_service_credential" => Some(Self::InvalidServiceCredential),
			"invalid_signature" => Some(Self::InvalidSignature),
			"lp_payout_not_recoverable" => Some(Self::LpPayoutNotRecoverable),
			"lp_payout_run_not_found" => Some(Self::LpPayoutRunNotFound),
			"malformed_json" => Some(Self::MalformedJson),
			"malformed_websocket_message" => Some(Self::MalformedWebsocketMessage),
			"market_closed" => Some(Self::MarketClosed),
			"market_not_found" => Some(Self::MarketNotFound),
			"method_not_allowed" => Some(Self::MethodNotAllowed),
			"order_expiration_too_soon" => Some(Self::OrderExpirationTooSoon),
			"order_funding_failed" => Some(Self::OrderFundingFailed),
			"order_not_cancellable" => Some(Self::OrderNotCancellable),
			"order_not_fillable" => Some(Self::OrderNotFillable),
			"order_not_found" => Some(Self::OrderNotFound),
			"order_price_out_of_range" => Some(Self::OrderPriceOutOfRange),
			"order_size_above_maximum" => Some(Self::OrderSizeAboveMaximum),
			"order_size_below_minimum" => Some(Self::OrderSizeBelowMinimum),
			"order_submission_failed" => Some(Self::OrderSubmissionFailed),
			"order_value_above_maximum" => Some(Self::OrderValueAboveMaximum),
			"order_value_below_minimum" => Some(Self::OrderValueBelowMinimum),
			"pnl_not_ready" => Some(Self::PnlNotReady),
			"pnl_unavailable" => Some(Self::PnlUnavailable),
			"post_only_would_cross" => Some(Self::PostOnlyWouldCross),
			"price_feed_not_found" => Some(Self::PriceFeedNotFound),
			"price_service_not_configured" => Some(Self::PriceServiceNotConfigured),
			"price_stream_capacity_exceeded" => Some(Self::PriceStreamCapacityExceeded),
			"price_stream_not_configured" => Some(Self::PriceStreamNotConfigured),
			"price_temporarily_unavailable" => Some(Self::PriceTemporarilyUnavailable),
			"profile_not_found" => Some(Self::ProfileNotFound),
			"rate_limited" => Some(Self::RateLimited),
			"request_body_too_large" => Some(Self::RequestBodyTooLarge),
			"route_not_found" => Some(Self::RouteNotFound),
			"security_not_found" => Some(Self::SecurityNotFound),
			"sequence_conflict" => Some(Self::SequenceConflict),
			"server_restarting" => Some(Self::ServerRestarting),
			"settlement_divergent" => Some(Self::SettlementDivergent),
			"settlements_pending" => Some(Self::SettlementsPending),
			"signer_authorization_required" => Some(Self::SignerAuthorizationRequired),
			"slow_consumer" => Some(Self::SlowConsumer),
			"stream_subject_not_found" => Some(Self::StreamSubjectNotFound),
			"stream_subject_unavailable" => Some(Self::StreamSubjectUnavailable),
			"subscription_limit_exceeded" => Some(Self::SubscriptionLimitExceeded),
			"token_not_found" => Some(Self::TokenNotFound),
			"trading_day_not_found" => Some(Self::TradingDayNotFound),
			"unsupported_media_type" => Some(Self::UnsupportedMediaType),
			"unsupported_websocket_data" => Some(Self::UnsupportedWebsocketData),
			"validation_failed" => Some(Self::ValidationFailed),
			"value_cap_exceeded" => Some(Self::ValueCapExceeded),
			"venue_not_found" => Some(Self::VenueNotFound),
			"wallet_not_registered" => Some(Self::WalletNotRegistered),
			"wallet_setup_failed" => Some(Self::WalletSetupFailed),
			"wallet_setup_incomplete" => Some(Self::WalletSetupIncomplete),
			"wallet_setup_outcome_unknown" => Some(Self::WalletSetupOutcomeUnknown),
			"websocket_message_too_large" => Some(Self::WebsocketMessageTooLarge),
			"websocket_protocol_error" => Some(Self::WebsocketProtocolError),
			"websocket_upgrade_required" => Some(Self::WebsocketUpgradeRequired),
			"wrong_credential_type" => Some(Self::WrongCredentialType),
			_ => None,
		}
	}

	/// Stable lower-snake-case name sent by the server.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::AccountFrozen => "account_frozen",
			Self::AccountQuarantined => "account_quarantined",
			Self::BatchInFlight => "batch_in_flight",
			Self::BatchNotFound => "batch_not_found",
			Self::BatchSuperseded => "batch_superseded",
			Self::CategoryNotFound => "category_not_found",
			Self::ChainExecutionFailed => "chain_execution_failed",
			Self::CredentialsMissing => "credentials_missing",
			Self::DependencyCredentialsRejected => "dependency_credentials_rejected",
			Self::DependencyInvalidResponse => "dependency_invalid_response",
			Self::DependencyOutcomeUnknown => "dependency_outcome_unknown",
			Self::DependencyUnavailable => "dependency_unavailable",
			Self::DuplicateOrder => "duplicate_order",
			Self::EndpointRetired => "endpoint_retired",
			Self::EventNotFound => "event_not_found",
			Self::FeatureNotConfigured => "feature_not_configured",
			Self::ForbiddenScope => "forbidden_scope",
			Self::GroupNotFound => "group_not_found",
			Self::IdentityTokenExpired => "identity_token_expired",
			Self::InsufficientBalance => "insufficient_balance",
			Self::InsufficientShares => "insufficient_shares",
			Self::InternalError => "internal_error",
			Self::InvalidBatch => "invalid_batch",
			Self::InvalidIdentityToken => "invalid_identity_token",
			Self::InvalidMessage => "invalid_message",
			Self::InvalidPathParameter => "invalid_path_parameter",
			Self::InvalidPersonalAccessToken => "invalid_personal_access_token",
			Self::InvalidServiceCredential => "invalid_service_credential",
			Self::InvalidSignature => "invalid_signature",
			Self::LpPayoutNotRecoverable => "lp_payout_not_recoverable",
			Self::LpPayoutRunNotFound => "lp_payout_run_not_found",
			Self::MalformedJson => "malformed_json",
			Self::MalformedWebsocketMessage => "malformed_websocket_message",
			Self::MarketClosed => "market_closed",
			Self::MarketNotFound => "market_not_found",
			Self::MethodNotAllowed => "method_not_allowed",
			Self::OrderExpirationTooSoon => "order_expiration_too_soon",
			Self::OrderFundingFailed => "order_funding_failed",
			Self::OrderNotCancellable => "order_not_cancellable",
			Self::OrderNotFillable => "order_not_fillable",
			Self::OrderNotFound => "order_not_found",
			Self::OrderPriceOutOfRange => "order_price_out_of_range",
			Self::OrderSizeAboveMaximum => "order_size_above_maximum",
			Self::OrderSizeBelowMinimum => "order_size_below_minimum",
			Self::OrderSubmissionFailed => "order_submission_failed",
			Self::OrderValueAboveMaximum => "order_value_above_maximum",
			Self::OrderValueBelowMinimum => "order_value_below_minimum",
			Self::PnlNotReady => "pnl_not_ready",
			Self::PnlUnavailable => "pnl_unavailable",
			Self::PostOnlyWouldCross => "post_only_would_cross",
			Self::PriceFeedNotFound => "price_feed_not_found",
			Self::PriceServiceNotConfigured => "price_service_not_configured",
			Self::PriceStreamCapacityExceeded => "price_stream_capacity_exceeded",
			Self::PriceStreamNotConfigured => "price_stream_not_configured",
			Self::PriceTemporarilyUnavailable => "price_temporarily_unavailable",
			Self::ProfileNotFound => "profile_not_found",
			Self::RateLimited => "rate_limited",
			Self::RequestBodyTooLarge => "request_body_too_large",
			Self::RouteNotFound => "route_not_found",
			Self::SecurityNotFound => "security_not_found",
			Self::SequenceConflict => "sequence_conflict",
			Self::ServerRestarting => "server_restarting",
			Self::SettlementDivergent => "settlement_divergent",
			Self::SettlementsPending => "settlements_pending",
			Self::SignerAuthorizationRequired => "signer_authorization_required",
			Self::SlowConsumer => "slow_consumer",
			Self::StreamSubjectNotFound => "stream_subject_not_found",
			Self::StreamSubjectUnavailable => "stream_subject_unavailable",
			Self::SubscriptionLimitExceeded => "subscription_limit_exceeded",
			Self::TokenNotFound => "token_not_found",
			Self::TradingDayNotFound => "trading_day_not_found",
			Self::UnsupportedMediaType => "unsupported_media_type",
			Self::UnsupportedWebsocketData => "unsupported_websocket_data",
			Self::ValidationFailed => "validation_failed",
			Self::ValueCapExceeded => "value_cap_exceeded",
			Self::VenueNotFound => "venue_not_found",
			Self::WalletNotRegistered => "wallet_not_registered",
			Self::WalletSetupFailed => "wallet_setup_failed",
			Self::WalletSetupIncomplete => "wallet_setup_incomplete",
			Self::WalletSetupOutcomeUnknown => "wallet_setup_outcome_unknown",
			Self::WebsocketMessageTooLarge => "websocket_message_too_large",
			Self::WebsocketProtocolError => "websocket_protocol_error",
			Self::WebsocketUpgradeRequired => "websocket_upgrade_required",
			Self::WrongCredentialType => "wrong_credential_type",
		}
	}

	/// Canonical safe title registered for this problem.
	pub fn title(self) -> &'static str {
		registry::metadata(self).title
	}

	/// Canonical safe detail; absence is part of the contract.
	pub fn public_detail(self) -> Option<&'static str> {
		registry::metadata(self).detail
	}

	/// Expected HTTP status, absent for stream-only or asynchronous classifications.
	pub fn http_status(self) -> Option<u16> {
		registry::metadata(self).status
	}
}

impl FutureCode {
	/// Validate a future wire code's bounded lower-snake-case syntax.
	pub fn new(value: Cow<'static, str>) -> Result<Self, ProblemDecodeError> {
		decode::code_name(&value, ProblemField::Code)?;

		Ok(Self(value))
	}

	/// Borrow the original future code without assigning behavior to it.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl ProblemCode {
	/// Parse a wire code into a closed known variant or bounded future variant.
	pub fn parse(value: &str) -> Result<Self, ProblemDecodeError> {
		decode::code_name(value, ProblemField::Code)?;

		Ok(KnownProblemCode::from_wire(value).map_or_else(
			|| Self::Unknown(FutureCode(Cow::Owned(value.to_owned()))),
			Self::Known,
		))
	}

	/// Borrow the server's machine-readable wire name.
	pub fn as_str(&self) -> &str {
		match self {
			Self::Known(code) => code.as_str(),
			Self::Unknown(code) => code.as_str(),
		}
	}

	/// Registered classification when this SDK recognizes the code.
	pub const fn known(&self) -> Option<KnownProblemCode> {
		match self {
			Self::Known(code) => Some(*code),
			Self::Unknown(_) => None,
		}
	}
}

impl KnownFieldErrorCode {
	/// Lower-snake-case field-error name from the platform registry.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::InvalidValue => "invalid_value",
			Self::AdditionalErrors => "additional_errors",
		}
	}

	/// Canonical public explanation fixed by the classification.
	pub const fn message(self) -> &'static str {
		match self {
			Self::InvalidValue => "Invalid request value.",
			Self::AdditionalErrors => "Additional validation errors were omitted.",
		}
	}
}

impl FieldErrorCode {
	/// Parse a registered or future field-error code using the same bounded syntax.
	pub fn parse(value: &str) -> Result<Self, ProblemDecodeError> {
		decode::code_name(value, ProblemField::FieldErrorCode)?;

		Ok(match value {
			"invalid_value" => Self::Known(KnownFieldErrorCode::InvalidValue),
			"additional_errors" => Self::Known(KnownFieldErrorCode::AdditionalErrors),
			_ => Self::Unknown(FutureCode(Cow::Owned(value.to_owned()))),
		})
	}

	/// Borrow the wire name without parsing diagnostic text.
	pub fn as_str(&self) -> &str {
		match self {
			Self::Known(code) => code.as_str(),
			Self::Unknown(code) => code.as_str(),
		}
	}
}

impl fmt::Display for ProblemCode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl fmt::Display for KnownProblemCode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl<'de> Deserialize<'de> for ProblemCode {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = String::deserialize(deserializer)?;

		Self::parse(&value).map_err(serde::de::Error::custom)
	}
}

impl Serialize for ProblemCode {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.as_str())
	}
}

impl<'de> Deserialize<'de> for FieldErrorCode {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = String::deserialize(deserializer)?;

		Self::parse(&value).map_err(serde::de::Error::custom)
	}
}

impl Serialize for FieldErrorCode {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.as_str())
	}
}
