//! Adversarial request mutation tests independent of HTTP transport and optional signing support.

use agara_sdk::{
	Micro,
	batches::{
		AccountBatchSubmission, AccountBatchSupersedeSubmission, BatchField, BatchOpDto,
		BatchValidationError, WireValueError,
	},
	ids::{OrderHash, OrderType, Side, TimeInForce, TokenId},
	models::{OrderField, OrderValidationError, SignedOrderRequest},
};

const NOW: i64 = 1_800_000_000;
const ACCOUNT: &str = "0x1111111111111111111111111111111111111111";
const RECIPIENT: &str = "0x2222222222222222222222222222222222222222";
const CONDITION: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MARKET: &str = "10000000-0000-4000-8000-000000000001";
const ZERO_WORD: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const MAX_U256: &str =
	"115792089237316195423570985008687907853269984665640564039457584007913129639935";
const OVERFLOW_U256: &str =
	"115792089237316195423570985008687907853269984665640564039457584007913129639936";

fn scalar_signature(r: &str, s: &str, parity: u8) -> String {
	std::format!("0x{r:0>64}{s:0>64}{parity:02x}")
}

fn request() -> SignedOrderRequest {
	SignedOrderRequest {
		token_id: TokenId::new("2").unwrap(),
		side: Side::Buy,
		order_type: OrderType::Limit,
		time_in_force: TimeInForce::Gtc,
		price_micro: Micro::new(500_000),
		shares_micro: Micro::new(2_000_000),
		post_only: false,
		expiration_unix_seconds: None,
		order_hash: OrderHash::new(CONDITION).unwrap(),
		signature: scalar_signature("1", "1", 27),
		salt: "1".to_owned(),
		maker: ACCOUNT.to_owned(),
		chain_token_id: "2".to_owned(),
		maker_amount: "1000000".to_owned(),
		taker_amount: "2000000".to_owned(),
		side_u8: 0,
		timestamp: "0".to_owned(),
		metadata: ZERO_WORD.to_owned(),
		builder: ZERO_WORD.to_owned(),
	}
}

fn split() -> BatchOpDto {
	BatchOpDto::Split {
		market_id: MARKET.to_owned(),
		condition_id: CONDITION.to_owned(),
		shares_micro: 1_000_000,
	}
}

fn batch() -> AccountBatchSubmission {
	AccountBatchSubmission {
		ops: std::vec![split()],
		seq: 0,
		deadline_unix_seconds: (NOW + 3600) as u64,
		signature: scalar_signature("1", "1", 27),
		heals_batch_hash: None,
	}
}

#[rstest::rstest]
#[case(OrderField::Salt, "0", WireValueError::Zero)]
#[case(OrderField::Salt, "-1", WireValueError::DecimalSyntax)]
#[case(OrderField::Salt, "1.5", WireValueError::DecimalSyntax)]
#[case(OrderField::Salt, OVERFLOW_U256, WireValueError::Uint256Overflow)]
#[case(
	OrderField::Maker,
	"0x0000000000000000000000000000000000000000",
	WireValueError::Zero
)]
#[case(OrderField::Maker, "0x01", WireValueError::HexLength { bytes: 20 })]
#[case(OrderField::Timestamp, "1e6", WireValueError::DecimalSyntax)]
#[case(OrderField::Timestamp, OVERFLOW_U256, WireValueError::Uint256Overflow)]
#[case(OrderField::Metadata, "0x12", WireValueError::HexLength { bytes: 32 })]
#[case(
	OrderField::Builder,
	"0xzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
	WireValueError::HexEncoding
)]
#[case(
	OrderField::ChainTokenId,
	OVERFLOW_U256,
	WireValueError::Uint256Overflow
)]
#[case(OrderField::MakerAmount, "0x10", WireValueError::DecimalSyntax)]
#[case(OrderField::TakerAmount, "-1", WireValueError::DecimalSyntax)]
fn mutable_wire_fields_are_revalidated(
	#[case] field: OrderField,
	#[case] value: &str,
	#[case] expected: WireValueError,
) {
	// Arrange
	let mut request = request();
	match field {
		OrderField::Salt => request.salt = value.to_owned(),
		OrderField::Maker => request.maker = value.to_owned(),
		OrderField::Timestamp => request.timestamp = value.to_owned(),
		OrderField::Metadata => request.metadata = value.to_owned(),
		OrderField::Builder => request.builder = value.to_owned(),
		OrderField::ChainTokenId => request.chain_token_id = value.to_owned(),
		OrderField::MakerAmount => request.maker_amount = value.to_owned(),
		OrderField::TakerAmount => request.taker_amount = value.to_owned(),
		_ => core::unreachable!(),
	}

	// Act
	let error = request.validate_at(NOW).unwrap_err();

	// Assert
	match error {
		OrderValidationError::Field { field: actual, source } => {
			core::assert_eq!(actual, field);
			core::assert_eq!(source, expected);
		},
		other => core::panic!("unexpected error: {other:?}"),
	}
}

#[rstest::rstest]
#[case("0", "1", 27, WireValueError::SignatureR)]
#[case(
	"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
	"1",
	27,
	WireValueError::SignatureR
)]
#[case("1", "0", 27, WireValueError::SignatureS)]
#[case(
	"1",
	"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
	27,
	WireValueError::SignatureS
)]
#[case(
	"1",
	"7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a1",
	27,
	WireValueError::SignatureHighS
)]
#[case("1", "1", 35, WireValueError::SignatureParity)]
fn malformed_signature_scalars_are_typed_without_signing_feature(
	#[case] r: &str,
	#[case] s: &str,
	#[case] parity: u8,
	#[case] expected: WireValueError,
) {
	// Arrange
	let mut request = request();
	request.signature = scalar_signature(r, s, parity);

	// Act
	let error = request.validate_at(NOW).unwrap_err();

	// Assert
	core::assert!(
		core::matches!(error, OrderValidationError::Field { field: OrderField::Signature, source } if source == expected)
	);
}

#[rstest::rstest]
#[case(27)]
#[case(28)]
fn canonical_scalar_boundaries_and_supported_parities_validate(#[case] parity: u8) {
	// Arrange
	let mut request = request();
	request.salt = MAX_U256.to_owned();
	request.timestamp = MAX_U256.to_owned();
	request.signature = scalar_signature(
		"1",
		"7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0",
		parity,
	);

	// Act
	let result = request.validate_at(NOW);

	// Assert
	core::assert!(result.is_ok());
}

#[test]
fn amount_and_side_tampering_cannot_preserve_structural_validity() {
	// Arrange
	let mut amount = request();
	amount.maker_amount = "1000001".to_owned();
	let mut side = request();
	side.side_u8 = 1;
	let mut token = request();
	token.chain_token_id = "3".to_owned();

	// Act
	let amount = amount.validate_at(NOW);
	let side = side.validate_at(NOW);
	let token = token.validate_at(NOW);

	// Assert
	core::assert!(core::matches!(
		amount,
		Err(OrderValidationError::AmountMismatch { field: OrderField::MakerAmount })
	));
	core::assert!(core::matches!(
		side,
		Err(OrderValidationError::SideMismatch)
	));
	core::assert!(core::matches!(
		token,
		Err(OrderValidationError::TokenMismatch)
	));
}

#[test]
fn sell_requires_reversed_signed_amounts() {
	// Arrange
	let mut request = request();
	request.side = Side::Sell;
	request.side_u8 = 1;

	// Act
	let unchanged = request.validate_at(NOW);
	core::mem::swap(&mut request.maker_amount, &mut request.taker_amount);
	let reversed = request.validate_at(NOW);

	// Assert
	core::assert!(core::matches!(
		unchanged,
		Err(OrderValidationError::AmountMismatch { field: OrderField::MakerAmount })
	));
	core::assert!(reversed.is_ok());
}

#[rstest::rstest]
#[case(TimeInForce::Fak)]
#[case(TimeInForce::Fok)]
fn post_only_cannot_cross_immediate_time_in_force(#[case] tif: TimeInForce) {
	// Arrange
	let mut request = request();
	request.time_in_force = tif;
	request.post_only = true;

	// Act
	let error = request.validate_at(NOW).unwrap_err();

	// Assert
	core::assert!(core::matches!(
		error,
		OrderValidationError::PostOnlyTimeInForce
	));
}

#[test]
fn expiry_checks_all_shapes_and_exact_thirty_second_boundary() {
	// Arrange
	let mut request = request();
	request.expiration_unix_seconds = Some(NOW + 30);

	// Act
	let unexpected = request.validate_at(NOW);
	request.time_in_force = TimeInForce::Gtd;
	let boundary = request.validate_at(NOW);
	request.expiration_unix_seconds = Some(NOW + 29);
	let too_soon = request.validate_at(NOW);
	request.expiration_unix_seconds = None;
	let absent = request.validate_at(NOW);

	// Assert
	core::assert!(core::matches!(
		unexpected,
		Err(OrderValidationError::ExpirationUnexpected)
	));
	core::assert!(boundary.is_ok());
	core::assert!(core::matches!(
		too_soon,
		Err(OrderValidationError::ExpirationTooSoon)
	));
	core::assert!(core::matches!(
		absent,
		Err(OrderValidationError::ExpirationRequired)
	));
}

#[test]
fn signed_limit_shape_and_notional_limits_are_enforced() {
	// Arrange
	let mut request = request();
	request.order_type = OrderType::Market;

	// Act
	let market = request.validate_at(NOW);
	request.order_type = OrderType::Limit;
	request.shares_micro = Micro::new(199_999);
	let below = request.validate_at(NOW);
	request.shares_micro = Micro::new(200_000_000_002);
	let above = request.validate_at(NOW);
	request.shares_micro = Micro::new(2_000_000);
	request.price_micro = Micro::new(1_000_000);
	let price = request.validate_at(NOW);

	// Assert
	core::assert!(core::matches!(market, Err(OrderValidationError::OrderType)));
	core::assert!(core::matches!(
		below,
		Err(OrderValidationError::NotionalBelowMinimum)
	));
	core::assert!(core::matches!(
		above,
		Err(OrderValidationError::NotionalAboveMaximum)
	));
	core::assert!(core::matches!(price, Err(OrderValidationError::Price)));
}

#[test]
fn deserialize_does_not_bypass_batch_binding_validation() {
	// Arrange
	let value = serde_json::json!({"ops":[{"kind":"SPLIT","market_id":MARKET,"condition_id":CONDITION,"shares_micro":1000000}],"seq":u64::MAX,"deadline_unix_seconds":NOW+60,"signature":scalar_signature("1","1",27)});
	let batch: AccountBatchSubmission = serde_json::from_value(value).unwrap();

	// Act
	let error = batch.validate_at(NOW).unwrap_err();

	// Assert
	core::assert!(core::matches!(
		error,
		BatchValidationError::SequenceStorageRange
	));
}

#[rstest::rstest]
#[case(0, false)]
#[case(1, true)]
#[case(20, true)]
#[case(21, false)]
fn batch_operation_count_has_inclusive_public_bounds(#[case] count: usize, #[case] accepted: bool) {
	// Arrange
	let mut batch = batch();
	batch.ops = std::vec![split(); count];

	// Act
	let result = batch.validate_at(NOW);

	// Assert
	core::assert_eq!(result.is_ok(), accepted);
	if count == 0 {
		core::assert!(core::matches!(
			result,
			Err(BatchValidationError::EmptyOperations)
		));
	}
	if count == 21 {
		core::assert!(core::matches!(
			result,
			Err(BatchValidationError::TooManyOperations { actual: 21 })
		));
	}
}

#[test]
fn batch_deadline_and_signature_policies_are_identical_for_successors() {
	// Arrange
	let mut batch = batch();
	batch.deadline_unix_seconds = (NOW + 86_340) as u64;
	let mut successor = AccountBatchSupersedeSubmission {
		ops: batch.ops.clone(),
		deadline_unix_seconds: batch.deadline_unix_seconds,
		signature: batch.signature.clone(),
	};

	// Act
	let accepted = batch.validate_at(NOW);
	let successor_accepted = successor.validate_at(NOW);
	batch.deadline_unix_seconds += 1;
	successor.deadline_unix_seconds = NOW as u64;
	let far = batch.validate_at(NOW);
	let expired = successor.validate_at(NOW);
	successor.deadline_unix_seconds = (NOW + 60) as u64;
	successor.signature = "0x11".to_owned();
	let signature = successor.validate_at(NOW);

	// Assert
	core::assert!(accepted.is_ok() && successor_accepted.is_ok());
	core::assert!(core::matches!(
		far,
		Err(BatchValidationError::DeadlineTooFar)
	));
	core::assert!(core::matches!(
		expired,
		Err(BatchValidationError::DeadlineExpired)
	));
	core::assert!(core::matches!(
		signature,
		Err(BatchValidationError::Field {
			field: BatchField::Signature,
			source: WireValueError::HexLength { bytes: 65 }
		})
	));
}

#[rstest::rstest]
#[case(-1)]
#[case(0)]
#[case(9_999)]
fn withdrawals_cannot_bypass_the_minimum(#[case] amount_micro: i64) {
	// Arrange
	let op = BatchOpDto::Withdraw { destination: RECIPIENT.to_owned(), amount_micro };

	// Act
	let error = op.validate().unwrap_err();

	// Assert
	core::assert!(core::matches!(
		error,
		BatchValidationError::WithdrawalBelowMinimum
	));
}

#[test]
fn zero_targets_self_withdrawal_nil_market_and_bad_conditions_are_distinct() {
	// Arrange
	let zero = BatchOpDto::Withdraw {
		destination: "0x0000000000000000000000000000000000000000".to_owned(),
		amount_micro: 10_000,
	};
	let self_send = BatchOpDto::Withdraw { destination: ACCOUNT.to_owned(), amount_micro: 10_000 };
	let nil = BatchOpDto::Split {
		market_id: "00000000-0000-0000-0000-000000000000".to_owned(),
		condition_id: CONDITION.to_owned(),
		shares_micro: 1,
	};
	let bad_condition = BatchOpDto::Merge {
		market_id: MARKET.to_owned(),
		condition_id: "0x12".to_owned(),
		shares_micro: 1,
	};

	// Act
	let zero = zero.validate();
	let self_send = self_send.validate_for_account(ACCOUNT);
	let nil = nil.validate();
	let bad_condition = bad_condition.validate();

	// Assert
	core::assert!(core::matches!(
		zero,
		Err(BatchValidationError::Field {
			field: BatchField::Destination,
			source: WireValueError::Zero
		})
	));
	core::assert!(core::matches!(
		self_send,
		Err(BatchValidationError::SelfWithdrawal)
	));
	core::assert!(core::matches!(
		nil,
		Err(BatchValidationError::Field {
			field: BatchField::MarketId,
			source: WireValueError::NilUuid
		})
	));
	core::assert!(core::matches!(
		bad_condition,
		Err(BatchValidationError::Field {
			field: BatchField::ConditionId,
			source: WireValueError::HexLength { bytes: 32 }
		})
	));
}

#[cfg(feature = "signing")]
mod crypto {
	use super::*;
	use agara_sdk::{
		batch_signing::{
			self, BatchCall, BatchCallDto,
			compose::{self, ComposeContext, ComposeError, ContractTarget, MarketRoute, RoutedOp},
		},
		signing::{self, CallFailure, DomainField, EngineDomain, SigningError},
	};
	use alloy::{
		primitives::{Address, Bytes, U256},
		signers::local::PrivateKeySigner,
	};

	const KEY: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
	const CHAIN: u64 = 8453;
	const EXCHANGE: Address =
		alloy::primitives::address!("3333333333333333333333333333333333333333");

	fn context() -> ComposeContext {
		ComposeContext::builder()
			.ctf(EXCHANGE)
			.neg_risk_adapter("0x4444444444444444444444444444444444444444".parse().unwrap())
			.collateral("0x5555555555555555555555555555555555555555".parse().unwrap())
			.build()
			.unwrap()
	}

	fn signed() -> SignedOrderRequest {
		signing::sign_limit_order()
			.private_key(KEY)
			.domain(EngineDomain::new(CHAIN, EXCHANGE).unwrap())
			.deposit_wallet_address(ACCOUNT.parse().unwrap())
			.token_id(U256::from(2))
			.side(Side::Buy)
			.price_micro(Micro::new(500_000))
			.shares_micro(Micro::new(2_000_000))
			.salt(U256::from(1))
			.call()
			.unwrap()
			.to_request_body()
			.token_id(TokenId::new("2").unwrap())
			.side(Side::Buy)
			.price_micro(Micro::new(500_000))
			.shares_micro(Micro::new(2_000_000))
			.time_in_force(TimeInForce::Gtc)
			.post_only(false)
			.call()
			.unwrap()
	}

	#[test]
	fn context_aware_order_verification_detects_hash_domain_and_holder_tampering() {
		// Arrange
		let request = signed();
		let signer: PrivateKeySigner = KEY.parse().unwrap();
		let domain = EngineDomain::new(CHAIN, EXCHANGE).unwrap();
		let mut tampered = request.clone();
		tampered.timestamp = "1".to_owned();

		// Act
		let good = request.verify(domain, signer.address());
		let hash = tampered.verify(domain, signer.address());
		let wrong_domain = request.verify(
			EngineDomain::new(CHAIN + 1, EXCHANGE).unwrap(),
			signer.address(),
		);
		let wrong_holder = request.verify(domain, ACCOUNT.parse().unwrap());

		// Assert
		core::assert!(good.is_ok());
		core::assert!(core::matches!(hash, Err(SigningError::OrderHashMismatch)));
		core::assert!(core::matches!(
			wrong_domain,
			Err(SigningError::OrderHashMismatch)
		));
		core::assert!(core::matches!(
			wrong_holder,
			Err(SigningError::HolderMismatch)
		));
	}

	#[test]
	fn private_key_failures_preserve_typed_sources_without_echoing_the_key() {
		// Arrange
		let secret = "0xTHIS_IS_A_PRIVATE_KEY_INPUT_NOT_AN_ERROR_MESSAGE";

		// Act
		let error = signing::sign_limit_order()
			.private_key(secret)
			.domain(EngineDomain::new(CHAIN, EXCHANGE).unwrap())
			.deposit_wallet_address(ACCOUNT.parse().unwrap())
			.token_id(U256::from(2))
			.side(Side::Buy)
			.price_micro(Micro::new(500_000))
			.shares_micro(Micro::new(2_000_000))
			.call()
			.unwrap_err();

		// Assert
		core::assert!(core::matches!(error, SigningError::PrivateKey(_)));
		core::assert!(
			std::error::Error::source(&error)
				.unwrap()
				.downcast_ref::<alloy::signers::local::LocalSignerError>()
				.is_some()
		);
		core::assert!(!error.to_string().contains(secret));
		core::assert!(!std::format!("{error:?}").contains(secret));
	}

	#[test]
	fn zero_domains_and_context_targets_are_unconstructible() {
		// Arrange
		// Act
		let chain = EngineDomain::new(0, EXCHANGE);
		let exchange = EngineDomain::new(CHAIN, Address::ZERO);
		let context = ComposeContext::builder()
			.ctf(Address::ZERO)
			.collateral(EXCHANGE)
			.neg_risk_adapter(EXCHANGE)
			.build();

		// Assert
		core::assert!(core::matches!(
			chain,
			Err(SigningError::Domain(DomainField::ChainId))
		));
		core::assert!(core::matches!(
			exchange,
			Err(SigningError::Domain(DomainField::ExchangeContract))
		));
		core::assert!(core::matches!(
			context,
			Err(ComposeError::ZeroContract(ContractTarget::Ctf))
		));
	}

	#[test]
	fn raw_hashing_rejects_zero_self_and_oversized_call_domains() {
		// Arrange
		let account: Address = ACCOUNT.parse().unwrap();
		let call = BatchCall { target: EXCHANGE, value: U256::ZERO, data: Bytes::new() };
		let hash = |calls: &[BatchCall]| {
			batch_signing::batch_digest_local(account, 1, U256::ZERO, U256::from(1), calls, CHAIN)
		};

		// Act
		let empty = hash(&[]);
		let too_many = hash(&std::vec![call.clone(); 21]);
		let zero = hash(&[BatchCall { target: Address::ZERO, ..call.clone() }]);
		let own = hash(&[BatchCall { target: account, ..call.clone() }]);
		let historical = hash(&[call]);

		// Assert
		core::assert!(core::matches!(
			empty,
			Err(SigningError::Batch(BatchValidationError::EmptyOperations))
		));
		core::assert!(core::matches!(
			too_many,
			Err(SigningError::Batch(
				BatchValidationError::TooManyOperations { actual: 21 }
			))
		));
		core::assert!(core::matches!(
			zero,
			Err(SigningError::Call { index: 0, reason: CallFailure::ZeroTarget })
		));
		core::assert!(core::matches!(
			own,
			Err(SigningError::Call { index: 0, reason: CallFailure::SelfTarget })
		));
		core::assert!(historical.is_ok());
	}

	#[rstest::rstest]
	#[case(U256::MAX, U256::from(1), false)]
	#[case(U256::ZERO, U256::ZERO, true)]
	#[case(U256::ZERO, U256::MAX, true)]
	fn raw_hashing_never_truncates_persistent_binding_values(
		#[case] seq: U256,
		#[case] deadline: U256,
		#[case] is_deadline: bool,
	) {
		// Arrange
		let call = BatchCall { target: EXCHANGE, value: U256::ZERO, data: Bytes::new() };

		// Act
		let error = batch_signing::batch_digest_local(
			ACCOUNT.parse().unwrap(),
			1,
			seq,
			deadline,
			&[call],
			CHAIN,
		)
		.unwrap_err();

		// Assert
		if is_deadline {
			core::assert!(core::matches!(
				error,
				SigningError::Batch(BatchValidationError::DeadlineStorageRange)
			));
		} else {
			core::assert!(core::matches!(
				error,
				SigningError::Batch(BatchValidationError::SequenceStorageRange)
			));
		}
	}

	#[test]
	fn direct_composition_cannot_bypass_operation_validation() {
		// Arrange
		let context = context();
		let wrong = RoutedOp {
			op: BatchOpDto::Withdraw { destination: RECIPIENT.to_owned(), amount_micro: 1 },
			route: MarketRoute::Ctf,
		};
		let own = RoutedOp {
			op: BatchOpDto::Withdraw { destination: ACCOUNT.to_owned(), amount_micro: 10_000 },
			route: MarketRoute::Ctf,
		};
		let unsupported = RoutedOp { op: split(), route: MarketRoute::NegRisk };

		// Act
		let empty = compose::compose(&[], &context);
		let amount = compose::compose(&[wrong], &context);
		let own = compose::compose_for_account(&[own], &context, ACCOUNT.parse().unwrap());
		let route = compose::compose(&[unsupported], &context);

		// Assert
		core::assert!(core::matches!(
			empty,
			Err(ComposeError::Batch(BatchValidationError::EmptyOperations))
		));
		core::assert!(
			core::matches!(amount, Err(ComposeError::Batch(BatchValidationError::Operation { index: 0, source })) if core::matches!(*source, BatchValidationError::WithdrawalBelowMinimum))
		);
		core::assert!(
			core::matches!(own, Err(ComposeError::Batch(BatchValidationError::Operation { index: 0, source })) if core::matches!(*source, BatchValidationError::SelfWithdrawal))
		);
		core::assert!(core::matches!(
			route,
			Err(ComposeError::NegRiskSplit { index: 0 })
		));
	}

	#[test]
	fn complete_batch_signing_preserves_binding_and_recovers_holder() {
		// Arrange
		let ops = [RoutedOp { op: split(), route: MarketRoute::Ctf }];
		let context = context();
		let holder: PrivateKeySigner = KEY.parse().unwrap();

		// Act
		let (digest, submission) = batch_signing::sign_batch()
			.private_key(KEY)
			.account(ACCOUNT.parse().unwrap())
			.implementation_version(1)
			.seq(4)
			.deadline_unix_seconds((NOW + 3600) as u64)
			.chain_id(CHAIN)
			.ops(&ops)
			.context(&context)
			.now_unix_seconds(NOW)
			.call()
			.unwrap();
		let signature = alloy::primitives::Signature::from_raw(
			&alloy::hex::decode(&submission.signature).unwrap(),
		)
		.unwrap();
		let recovered = signature.recover_address_from_prehash(&digest).unwrap();

		// Assert
		core::assert!(submission.validate_at(NOW).is_ok());
		core::assert_eq!(submission.seq, 4);
		core::assert_eq!(submission.deadline_unix_seconds, (NOW + 3600) as u64);
		core::assert_eq!(recovered, holder.address());
	}

	#[test]
	fn raw_call_dto_conversion_retains_field_and_hex_error_types() {
		// Arrange
		let mut call = BatchCallDto {
			target: ACCOUNT.to_owned(),
			value: OVERFLOW_U256.to_owned(),
			data: "0x".to_owned(),
		};

		// Act
		let amount = call.to_call();
		call.value = "0".to_owned();
		call.data = "abcd".to_owned();
		let prefix = call.to_call();
		call.data = "0xzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".to_owned();
		let hex = call.to_call();

		// Assert
		core::assert!(core::matches!(
			amount,
			Err(SigningError::Batch(BatchValidationError::Field {
				field: BatchField::CallValue,
				source: WireValueError::Uint256Overflow
			}))
		));
		core::assert!(core::matches!(prefix, Err(SigningError::CallDataPrefix)));
		core::assert!(core::matches!(hex, Err(SigningError::CallData(_))));
	}
}

#[test]
fn batch_response_domains_reject_malformed_ids_hashes_and_timestamps() {
	// Arrange
	let valid = serde_json::json!({"batch_hash":CONDITION,"status":"SETTLED","seq":4,"deadline_unix_seconds":NOW+60,"origin":"PRESIGNED","tx_hash":CONDITION,"executed_at":"2026-09-17T00:00:00Z","created_at":"2026-09-17T00:00:00Z"});
	let mutations = [
		("batch_hash", serde_json::json!("0x1")),
		("tx_hash", serde_json::json!("0x1")),
		("created_at", serde_json::json!("now")),
		("seq", serde_json::json!(-1)),
		("seq", serde_json::json!(u64::MAX)),
		("deadline_unix_seconds", serde_json::json!(0)),
		("status", serde_json::json!("settled")),
		("origin", serde_json::json!("")),
	];

	// Act
	let decoded: agara_sdk::batches::AccountBatchStatusDto =
		serde_json::from_value(valid.clone()).unwrap();

	// Assert
	core::assert!(decoded.is_terminal());
	core::assert_eq!(decoded.seq.raw(), 4);
	core::assert_eq!(decoded.batch_hash.as_str(), CONDITION);
	for (field, value) in mutations {
		let mut invalid = valid.clone();
		invalid[field] = value;
		core::assert!(
			serde_json::from_value::<agara_sdk::batches::AccountBatchStatusDto>(invalid).is_err(),
			"{field}"
		);
	}
}

#[test]
fn unknown_batch_state_and_origin_are_preserved_without_completion_semantics() {
	// Arrange
	let value = serde_json::json!({"batch_hash":CONDITION,"status":"FUTURE_COMPLETE","seq":4,"deadline_unix_seconds":NOW+60,"origin":"FUTURE_AUTHORITY","created_at":"2026-09-17T00:00:00Z"});

	// Act
	let future: agara_sdk::batches::AccountBatchStatusDto = serde_json::from_value(value).unwrap();

	// Assert
	core::assert!(!future.is_terminal());
	core::assert_eq!(future.status.as_str(), "FUTURE_COMPLETE");
	core::assert_eq!(future.origin.as_str(), "FUTURE_AUTHORITY");
	core::assert!(core::matches!(
		future.status,
		agara_sdk::batches::BatchStatus::Unknown(_)
	));
}

#[test]
fn failed_batch_requires_valid_unwind_timestamp_before_it_is_terminal() {
	// Arrange
	let mut value = serde_json::json!({"batch_hash":CONDITION,"status":"FAILED","failure":{"code":"internal_error","title":"Something went wrong","detail":"We hit an unexpected problem. Please try again in a moment.","recovery":{"strategy":"none"}},"seq":4,"deadline_unix_seconds":NOW+60,"origin":"PRESIGNED","created_at":"2026-09-17T00:00:00Z","unwound_at":null});

	// Act
	let unwinding: agara_sdk::batches::AccountBatchStatusDto =
		serde_json::from_value(value.clone()).unwrap();
	value["unwound_at"] = "2026-09-17T00:00:01Z".into();
	let unwound: agara_sdk::batches::AccountBatchStatusDto = serde_json::from_value(value).unwrap();

	// Assert
	core::assert!(!unwinding.is_terminal());
	core::assert!(unwound.is_terminal());
}

#[test]
fn invalid_batch_market_uuid_retains_the_original_parser_source() {
	// Arrange
	let op = BatchOpDto::Split {
		market_id: "not-a-uuid".to_owned(),
		condition_id: CONDITION.to_owned(),
		shares_micro: 1_000_000,
	};

	// Act
	let error = op.validate().unwrap_err();

	// Assert
	match error {
		BatchValidationError::Field { field: BatchField::MarketId, source } => {
			core::assert!(
				std::error::Error::source(&source).unwrap().downcast_ref::<uuid::Error>().is_some()
			);
		},
		other => core::panic!("unexpected error: {other:?}"),
	}
}

#[rstest::rstest]
#[case(0)]
#[case(1)]
fn order_parity_is_chain_canonical_while_batch_allows_router_normalization(#[case] parity: u8) {
	// Arrange
	let mut order = request();
	order.signature = scalar_signature("1", "1", parity);
	let mut account_batch = batch();
	account_batch.signature = order.signature.clone();

	// Act
	let order = order.validate_at(NOW);
	let account_batch = account_batch.validate_at(NOW);

	// Assert
	core::assert!(core::matches!(
		order,
		Err(OrderValidationError::Field {
			field: OrderField::Signature,
			source: WireValueError::SignatureNonCanonicalParity
		})
	));
	core::assert!(account_batch.is_ok());
}
