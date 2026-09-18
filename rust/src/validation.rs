//! Machine-matchable validation failures shared by request boundaries.

mod reasons;

pub use reasons::{AmountReason, IdentifierReason, OrderInputReason, QueryReason};

/// The precise request or configuration field rejected before I/O.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Field {
	/// Bridge deposit source-chain identifier.
	FromChainId,

	/// Bridge withdrawal destination-chain identifier.
	ToChainId,

	/// Provider-specific source token address.
	FromTokenAddress,

	/// Provider-specific destination token address.
	ToTokenAddress,

	/// Provider-specific bridge recipient address.
	RecipientAddress,

	/// Positive integer amount in a token’s native base units.
	BaseUnitAmount,

	/// An identifier whose role is chosen by a generic boundary.
	Identifier,

	/// Account-batch group UUID.
	BatchGroupId,

	/// HTTP request path.
	Path,

	/// Personal access token credential.
	ApiToken,

	/// Order collateral budget.
	Collateral,

	/// A single exchange selection.
	Exchange,

	/// Market lifecycle-state filter.
	State,

	/// Parent event slug filter.
	EventSlug,

	/// Compound query configuration.
	Query,

	/// HTTP service origin and optional reverse-proxy path.
	BaseUrl,

	/// Bearer credential used for private endpoints.
	Token,

	/// Overall request timeout.
	Timeout,

	/// Connection establishment timeout.
	ConnectTimeout,

	/// Response read timeout.
	ReadTimeout,

	/// Maximum duration of a polling operation.
	PollTimeout,

	/// Delay between status reads.
	PollInterval,

	/// First retry delay.
	InitialBackoff,

	/// Upper bound on retry delay.
	MaxBackoff,

	/// Fraction of randomized retry delay.
	Jitter,

	/// Maximum number of retries.
	MaxRetries,

	/// Outcome token identifier.
	TokenId,

	/// On-chain market condition identifier.
	ConditionId,

	/// Internal order UUID.
	OrderId,

	/// Signed order digest.
	OrderHash,

	/// Internal wallet UUID.
	WalletId,

	/// Signed account-batch digest.
	BatchHash,

	/// Account-batch group UUID.
	GroupId,

	/// Catalogue market UUID.
	MarketId,

	/// Catalogue event UUID.
	EventId,

	/// EVM account or contract address.
	Address,

	/// Encoded cryptographic signature.
	Signature,

	/// Order uniqueness nonce.
	Salt,

	/// Order metadata digest.
	Metadata,

	/// Order builder digest.
	Builder,

	/// Signing domain chain identifier.
	ChainId,

	/// Account sequence bound by a batch signature.
	Sequence,

	/// Batch signature deadline.
	Deadline,

	/// Price expressed in whole collateral units.
	Price,

	/// Quantity expressed in whole shares.
	Shares,

	/// Budget expressed in whole collateral units.
	CollateralAmount,

	/// Price in integer micro units.
	PriceMicro,

	/// Quantity in integer micro shares.
	SharesMicro,

	/// Budget in integer micro collateral.
	CollateralAmountMicro,

	/// Whole-unit amount being converted.
	Amount,

	/// Integer micro-unit amount.
	AmountMicro,

	/// Order buy or sell direction.
	Side,

	/// Limit or market order kind.
	OrderType,

	/// Order persistence policy.
	TimeInForce,

	/// Maker-only execution flag.
	PostOnly,

	/// Good-til-date expiration timestamp.
	Expiration,

	/// Collection of independently signed orders.
	Orders,

	/// Account-batch operations.
	Operations,

	/// Requested exchange filters.
	Exchanges,

	/// Requested market condition filters.
	ConditionIds,

	/// Requested outcome filters.
	TokenIds,

	/// Maximum entries in a result page.
	Limit,

	/// Opaque pagination continuation token.
	Cursor,

	/// One-based page number.
	Page,

	/// Catalogue or incentive category filter.
	Category,

	/// Literal incentive-market search text.
	Search,

	/// Field selected for incentive sorting.
	SortBy,

	/// Ascending or descending sort direction.
	SortOrder,

	/// Inclusive range start.
	From,

	/// Inclusive range end.
	To,

	/// Timestamp of a historical price lookup.
	At,

	/// Calendar trading date.
	Date,

	/// Trading venue market identifier code.
	Mic,

	/// Canonical security or provider symbol.
	Symbol,

	/// Price data provider identifier.
	Provider,

	/// Token-price history window.
	Range,

	/// Number of sampled price points.
	Points,

	/// Realized-PnL bucket resolution.
	Granularity,

	/// Realized-PnL reporting window.
	Window,

	/// Catalogue exchange source.
	Source,

	/// Catalogue resource slug.
	Slug,

	/// Root category hierarchy filter.
	Root,

	/// Games or propositions event filter.
	EventTypeBucket,

	/// Named catalogue event filter.
	Filter,

	/// Whether ended events are excluded.
	ExcludeEnded,

	/// Event resolution filter.
	Resolution,

	/// Event ordering policy.
	Sort,

	/// Whether catalogue market enrichment is requested.
	IncludeMarkets,

	/// A single resource identifier in an HTTP path.
	PathSegment,
}

/// A rejected domain identifier and the violated invariant.
#[derive(Debug, thiserror::Error)]
#[error("invalid {field:?}: {reason}")]
pub struct IdentifierError {
	/// Typed identifier field that failed validation.
	pub field: Field,

	/// Specific identifier invariant that was violated.
	#[source]
	pub reason: IdentifierReason,
}

/// An amount that cannot be represented or accepted without rounding.
#[derive(Debug, thiserror::Error)]
#[error("invalid {field:?}: {reason}")]
pub struct AmountError {
	/// Amount field whose conversion or range check failed.
	pub field: Field,

	/// Exact conversion or amount-range failure.
	#[source]
	pub reason: AmountReason,
}

/// A query constraint rejected before a request is built.
#[derive(Debug, thiserror::Error)]
#[error("invalid {field:?}: {reason}")]
pub struct QueryError {
	/// Query parameter that requires correction.
	pub field: Field,

	/// Machine-matchable constraint, including conflicting fields when relevant.
	#[source]
	pub reason: QueryReason,
}

/// Invalid relationships among unsigned order inputs.
#[derive(Debug, thiserror::Error)]
#[error("invalid {field:?}: {reason}")]
pub struct OrderInputError {
	/// Order field that requires correction.
	pub field: Field,

	/// Violated order-shape or lifetime constraint.
	#[source]
	pub reason: OrderInputReason,
}

/// Invalid HTTP client configuration; parser errors retain their original source.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigurationError {
	/// The configured service URL could not be parsed.
	#[error("invalid base URL: {0}")]
	InvalidBaseUrl(#[source] url::ParseError),

	/// The bearer token cannot be represented as an HTTP header value.
	#[error("invalid token header: {0}")]
	InvalidTokenHeader(#[source] reqwest::header::InvalidHeaderValue),

	/// A private client was configured with an empty bearer credential.
	#[error("token must be nonempty")]
	EmptyToken,

	/// Only HTTP and HTTPS service origins are supported.
	#[error("base URL must use HTTP or HTTPS")]
	UnsupportedScheme,

	/// User information in a service URL would introduce a second authentication mechanism.
	#[error("base URL must not contain credentials")]
	UrlCredentials,

	/// Query parameters belong on individual requests.
	#[error("base URL must not contain a query")]
	UrlQuery,

	/// URL fragments are not sent to an HTTP server.
	#[error("base URL must not contain a fragment")]
	UrlFragment,

	/// The service URL has no network host.
	#[error("base URL must contain a host")]
	MissingHost,

	/// A configured timeout cannot provide a finite, positive deadline.
	#[error("{field:?} must be finite and positive")]
	InvalidTimeout {
		/// Timeout setting that must be corrected.
		field: Field,
	},
}

/// An invalid retry policy; no retry is scheduled when validation fails.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RetryPolicyError {
	/// Retry delays must not create a tight loop.
	#[error("initial retry backoff must be positive")]
	ZeroInitialBackoff,

	/// A backoff ceiling cannot be smaller than the initial delay.
	#[error("maximum backoff must be at least the initial backoff")]
	MaxBelowInitial,

	/// Jitter must be finite and between zero and one.
	#[error("retry jitter must be finite and in [0, 1]")]
	InvalidJitter,

	/// The configured attempt count exceeds the supported policy limit.
	#[error("retry count exceeds the supported maximum")]
	ExcessiveAttempts,
}

/// All shared local input-validation families.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidationError {
	/// A resource identifier violates its format or size constraints.
	#[error(transparent)]
	Identifier(#[from] IdentifierError),

	/// An amount is invalid, inexact, or outside the supported range.
	#[error(transparent)]
	Amount(#[from] AmountError),

	/// HTTP transport configuration cannot be used safely.
	#[error(transparent)]
	Configuration(#[from] ConfigurationError),

	/// Query parameters violate a documented endpoint constraint.
	#[error(transparent)]
	Query(#[from] QueryError),

	/// Unsigned order fields form an unsupported combination.
	#[error(transparent)]
	Order(#[from] OrderInputError),

	/// A retry policy cannot provide a bounded backoff.
	#[error(transparent)]
	RetryPolicy(#[from] RetryPolicyError),
}

impl ValidationError {
	/// Retarget a field-bearing failure without changing its category or original parser source.
	/// Configuration and retry errors without a field retain their intrinsic identity.
	pub fn with_field(mut self, field: Field) -> Self {
		match &mut self {
			Self::Identifier(error) => error.field = field,
			Self::Amount(error) => error.field = field,
			Self::Query(error) => error.field = field,
			Self::Order(error) => error.field = field,
			Self::Configuration(ConfigurationError::InvalidTimeout { field: original }) => {
				*original = field
			},
			Self::Configuration(_) | Self::RetryPolicy(_) => {},
		}

		self
	}

	/// Reject an identifier while preserving its field and exact invariant.
	pub const fn identifier(field: Field, reason: IdentifierReason) -> Self {
		Self::Identifier(IdentifierError { field, reason })
	}

	/// Reject an amount without substituting a rounded or saturated value.
	pub const fn amount(field: Field, reason: AmountReason) -> Self {
		Self::Amount(AmountError { field, reason })
	}

	/// Reject a query parameter before issuing network I/O.
	pub const fn query(field: Field, reason: QueryReason) -> Self {
		Self::Query(QueryError { field, reason })
	}

	/// Reject an invalid unsigned order field combination.
	pub const fn order(field: Field, reason: OrderInputReason) -> Self {
		Self::Order(OrderInputError { field, reason })
	}
}
