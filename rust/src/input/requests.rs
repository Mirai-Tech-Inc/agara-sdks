use crate::{
	AgaraError, batches, bridge, catalogue,
	ids::{OrderType, Side, TimeInForce},
	incentives, models, pnl,
	validation::{AmountReason, Field, OrderInputReason, QueryReason, ValidationError},
};

const MAX_SIGNED_ORDERS: i64 = 32;

macro_rules! validated {
    ($($type:ty),+ $(,)?) => {
        $(impl super::Validate for $type {
            fn validate_input(&self) -> Result<(), AgaraError> {
                <$type>::validate(self).map_err(AgaraError::from)
            }
        })+
    };
}
use validated;

self::validated!(
	models::SignedOrderRequest,
	batches::AccountBatchSubmission,
	batches::AccountBatchSupersedeSubmission,
	catalogue::MarketsQuery,
	catalogue::EventsQuery,
	catalogue::PricePointQuery,
	catalogue::PriceTicksQuery,
	catalogue::TokenHistoryQuery,
	catalogue::SearchQuery,
	catalogue::CalendarRangeQuery,
	catalogue::NextSessionQuery,
	catalogue::PnlHistoryQuery,
	incentives::LpIncentivesParams,
	incentives::ClosedLpIncentivesParams,
	bridge::PortfolioBridgeDepositQuoteRequest,
	bridge::PortfolioBridgeWithdrawQuoteRequest,
);

fn bridge_amount(value: &str) -> Result<(), ValidationError> {
	let amount = crate::ids::TokenId::new(value.to_owned())
		.map_err(|_| ValidationError::amount(Field::BaseUnitAmount, AmountReason::OutOfRange))?;
	if amount.as_str() == "0" {
		return Err(ValidationError::amount(
			Field::BaseUnitAmount,
			AmountReason::NonPositive,
		));
	}

	if amount.as_str() != value {
		return Err(ValidationError::amount(
			Field::BaseUnitAmount,
			AmountReason::NonCanonicalInteger,
		));
	}

	Ok(())
}

impl super::Validate for models::CreateOrderRequest {
	fn validate_input(&self) -> Result<(), AgaraError> {
		if self.side == Side::Unspecified {
			return Err(ValidationError::order(Field::Side, OrderInputReason::Unsupported).into());
		}

		crate::client::validate_limit_options(
			self.side,
			self.time_in_force,
			self.post_only,
			self.expiration_unix_seconds,
		)?;
		match (self.order_type, self.side) {
			(OrderType::Limit, _) => {
				let price = self.price_micro.ok_or_else(|| {
					ValidationError::order(Field::PriceMicro, OrderInputReason::Required)
				})?;
				if !(1..crate::MICRO).contains(&price.raw()) {
					return Err(ValidationError::order(
						Field::PriceMicro,
						OrderInputReason::InvalidPrice,
					)
					.into());
				}

				let shares = self.shares_micro.ok_or_else(|| {
					ValidationError::order(Field::SharesMicro, OrderInputReason::Required)
				})?;
				super::positive(shares, Field::SharesMicro)?;
				if self.collateral_amount_micro.is_some() {
					return Err(ValidationError::order(
						Field::CollateralAmountMicro,
						OrderInputReason::Forbidden,
					)
					.into());
				}
			},
			(OrderType::Market, side) => {
				if !core::matches!(self.time_in_force, TimeInForce::Fak | TimeInForce::Fok) {
					return Err(ValidationError::order(
						Field::TimeInForce,
						OrderInputReason::Unsupported,
					)
					.into());
				}

				if self.price_micro.is_some() {
					return Err(ValidationError::order(
						Field::PriceMicro,
						OrderInputReason::Forbidden,
					)
					.into());
				}

				let (required, other, field, forbidden) = if side == Side::Buy {
					(
						self.collateral_amount_micro,
						self.shares_micro,
						Field::CollateralAmountMicro,
						Field::SharesMicro,
					)
				} else {
					(
						self.shares_micro,
						self.collateral_amount_micro,
						Field::SharesMicro,
						Field::CollateralAmountMicro,
					)
				};
				super::positive(
					required
						.ok_or_else(|| ValidationError::order(field, OrderInputReason::Required))?,
					field,
				)?;
				if other.is_some() {
					return Err(
						ValidationError::order(forbidden, OrderInputReason::Forbidden).into(),
					);
				}
			},
			(OrderType::Unknown, _) => {
				return Err(ValidationError::order(
					Field::OrderType,
					OrderInputReason::Unsupported,
				)
				.into());
			},
		}

		Ok(())
	}
}

impl super::Validate for models::SignedOrderBatchRequest {
	fn validate_input(&self) -> Result<(), AgaraError> {
		super::number(
			self.orders.len() as i64,
			Field::Orders,
			1,
			MAX_SIGNED_ORDERS,
		)?;
		for (index, order) in self.orders.iter().enumerate() {
			order.validate().map_err(|source| models::OrderValidationError::BatchEntry {
				index,
				source: Box::new(source),
			})?;
		}

		Ok(())
	}
}

impl super::Validate for models::OrdersListRequest {
	fn validate_input(&self) -> Result<(), AgaraError> {
		super::number(
			i64::from(self.limit),
			Field::Limit,
			1,
			super::MAX_LIST_LIMIT,
		)?;
		super::cursor(self.cursor.as_deref())?;

		Ok(())
	}
}

impl super::Validate for models::PositionsListRequest {
	fn validate_input(&self) -> Result<(), AgaraError> {
		super::exchanges(&self.exchanges)?;

		Ok(())
	}
}

impl super::Validate for models::OpenOrdersListRequest {
	fn validate_input(&self) -> Result<(), AgaraError> {
		super::number(
			i64::from(self.limit),
			Field::Limit,
			1,
			super::MAX_LIST_LIMIT,
		)?;
		super::cursor(self.cursor.as_deref())?;
		super::exchanges(&self.exchanges)?;

		Ok(())
	}
}

impl super::Validate for models::SplitRequest {
	fn validate_input(&self) -> Result<(), AgaraError> {
		super::positive(self.collateral_amount_micro, Field::CollateralAmountMicro)?;

		Ok(())
	}
}

impl super::Validate for models::MergeRequest {
	fn validate_input(&self) -> Result<(), AgaraError> {
		super::positive(self.shares_micro, Field::SharesMicro)?;

		Ok(())
	}
}

impl super::Validate for pnl::RealizedPnlReportQueryDto {
	fn validate_input(&self) -> Result<(), AgaraError> {
		self.validate().map_err(AgaraError::from)
	}
}

impl pnl::RealizedPnlReportQueryDto {
	/// Require hour/day buckets for 1d, 7d or 30d, or week buckets for the all-history window.
	pub fn validate(&self) -> Result<(), ValidationError> {
		let weekly = core::matches!(self.granularity, pnl::RealizedPnlBucketGranularityDto::Week);
		let all = core::matches!(self.window, pnl::RealizedPnlWindowDto::All);
		if weekly != all {
			return Err(ValidationError::query(
				Field::Granularity,
				QueryReason::IncompatibleFields { other: Field::Window },
			));
		}

		Ok(())
	}
}

impl bridge::PortfolioBridgeDepositQuoteRequest {
	/// Validate a positive exact base-unit amount and provider identifiers before requesting a quote.
	/// Chain and token availability are checked by the server against the provider's supported assets.
	pub fn validate(&self) -> Result<(), ValidationError> {
		super::text(
			&self.from_chain_id,
			Field::FromChainId,
			super::MAX_TEXT_BYTES,
		)?;
		super::text(
			&self.from_token_address,
			Field::FromTokenAddress,
			super::MAX_TEXT_BYTES,
		)?;
		bridge_amount(&self.from_amount_base_unit)
	}
}

impl bridge::PortfolioBridgeWithdrawQuoteRequest {
	/// Validate exact quote inputs while preserving chain-specific address formats.
	/// Recipient and asset availability remain provider-owned; addresses need not be EVM addresses.
	pub fn validate(&self) -> Result<(), ValidationError> {
		super::text(&self.to_chain_id, Field::ToChainId, super::MAX_TEXT_BYTES)?;
		super::text(
			&self.to_token_address,
			Field::ToTokenAddress,
			super::MAX_TEXT_BYTES,
		)?;
		super::text(
			&self.recipient_address,
			Field::RecipientAddress,
			super::MAX_TEXT_BYTES,
		)?;
		bridge_amount(&self.from_amount_base_unit)
	}
}
