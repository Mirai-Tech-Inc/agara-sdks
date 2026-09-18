//! Conformance vectors published in the consumer signing guide. These are a
//! CONTRACT with external integrators: four independent implementations
//! (this composer, viem, and two raw-keccak assemblies) have reproduced
//! them. Never re-bless a mismatch here: a diff means composition or the
//! digest changed, which silently breaks every consumer.

use account_batch::{
	compose::{self, ComposeContext, MarketRoute, RoutedOp},
	digest,
	ops::BatchOpDto,
};
use alloy::primitives::Address;
use uuid::Uuid;

const ACCOUNT: &str = "0x1111111111111111111111111111111111111111";

const CONDITION_1: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";

const CONDITION_2: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2";

const DESTINATION: &str = "0x5555555555555555555555555555555555555555";

const CHAIN_ID: u64 = 8453;

const SEQ: u64 = 4;

const DEADLINE: u64 = 1_784_022_000;

const SPLIT_CALLDATA: &str = "0x72ce427500000000000000000000000044444444444444444444444444444444444444440000000000000000000000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000004c4b40000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";

const MERGE_CALLDATA: &str = "0x9e7212ad00000000000000000000000044444444444444444444444444444444444444440000000000000000000000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000001e8480000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002";

const TRANSFER_CALLDATA: &str = "0xa9059cbb000000000000000000000000555555555555555555555555555555555555555500000000000000000000000000000000000000000000000000000000000f4240";

const SINGLE_CALL_BATCH_HASH: &str =
	"0x51cda675bcc49a94f2c4ace692aa542d182dcb43ffe000da5d343bb49a441574";

const MULTI_CALL_BATCH_HASH: &str =
	"0xd4fc3f9b4997d1c32b4407480b2ebc17ef397cd4cdfe5fa0c6e482d3fdd2eb23";

fn ctx() -> ComposeContext {
	ComposeContext::builder()
		.ctf("0x2222222222222222222222222222222222222222".parse::<Address>().unwrap())
		.neg_risk_adapter("0x3333333333333333333333333333333333333333".parse::<Address>().unwrap())
		.collateral("0x4444444444444444444444444444444444444444".parse::<Address>().unwrap())
		.build()
}

fn split_op() -> RoutedOp {
	RoutedOp {
		op: BatchOpDto::Split {
			market_id: Uuid::from_u128(1),
			condition_id: CONDITION_1.to_owned(),
			shares_micro: 5_000_000,
		},
		route: MarketRoute::Ctf,
	}
}

#[test]
fn vector_1_single_call_reproduces_published_values() {
	// Act
	let calls = compose::compose(&[split_op()], &ctx()).expect("composes");
	let digest = digest::batch_digest(
		ACCOUNT.parse().unwrap(),
		digest::INITIAL_ACCOUNT_IMPLEMENTATION_VERSION,
		SEQ,
		DEADLINE,
		&calls,
		CHAIN_ID,
	)
	.expect("digests");

	// Assert
	assert_eq!(calls.len(), 1);
	assert_eq!(calls[0].data, SPLIT_CALLDATA);
	assert_eq!(format!("{digest}"), SINGLE_CALL_BATCH_HASH);
}

#[test]
fn vector_2_multi_call_reproduces_published_values() {
	// Arrange
	let ops = vec![
		split_op(),
		RoutedOp {
			op: BatchOpDto::Merge {
				market_id: Uuid::from_u128(2),
				condition_id: CONDITION_2.to_owned(),
				shares_micro: 2_000_000,
			},
			route: MarketRoute::Ctf,
		},
		RoutedOp {
			op: BatchOpDto::Withdraw {
				destination: DESTINATION.to_owned(),
				amount_micro: 1_000_000,
			},
			route: MarketRoute::Ctf,
		},
	];

	// Act
	let calls = compose::compose(&ops, &ctx()).expect("composes");
	let digest = digest::batch_digest(
		ACCOUNT.parse().unwrap(),
		digest::INITIAL_ACCOUNT_IMPLEMENTATION_VERSION,
		SEQ,
		DEADLINE,
		&calls,
		CHAIN_ID,
	)
	.expect("digests");

	// Assert
	assert_eq!(calls.len(), 3);
	assert_eq!(calls[0].data, SPLIT_CALLDATA);
	assert_eq!(calls[1].data, MERGE_CALLDATA);
	assert_eq!(calls[2].data, TRANSFER_CALLDATA);
	assert_eq!(format!("{digest}"), MULTI_CALL_BATCH_HASH);
}
