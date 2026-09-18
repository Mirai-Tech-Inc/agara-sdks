#![cfg(test)]

use super::*;

#[test]
fn call_and_batch_typehashes_match_agara_batch_lib() {
	// Assert — alloy's derived typehashes equal the `cast`-derived goldens
	// of the literal type strings in AgaraBatchLib.
	core::assert_eq!(
		hashable::Call::eip712_type_hash(&hashable::Call {
			target: Address::ZERO,
			value: U256::ZERO,
			data: Bytes::new(),
		}),
		alloy::primitives::b256!(
			"84fa2cf05cd88e992eae77e851af68a4ee278dcff6ef504e487a55b3baadfbe5"
		),
	);
	core::assert_eq!(
		hashable::Batch::eip712_type_hash(&hashable::Batch {
			wallet: Address::ZERO,
			seq: U256::ZERO,
			deadline: U256::ZERO,
			calls: Vec::new(),
		}),
		alloy::primitives::b256!(
			"cfa53c4f79229249e0a2e24c310e53b868ba122f3f3970b2c419ccca75c133c7"
		),
	);
}

#[test]
fn batch_digest_matches_independently_computed_golden() {
	// Arrange — fixed single-call batch; golden computed with `cast`
	// against the AgaraAccount domain (verifyingContract == wallet).
	let account = alloy::primitives::address!("279640887C3806d4FBd424bb0B58F0430CE661C1");
	let target = alloy::primitives::address!("1b42FF8DdB251074637d3A9872D72f51e3AbB23d");
	let calls =
		[BatchCall { target, value: U256::ZERO, data: Bytes::from_static(&[0xab, 0xcd, 0xef]) }];

	// Act
	let digest = batch_digest_local(
		account,
		INITIAL_ACCOUNT_IMPLEMENTATION_VERSION,
		U256::ZERO,
		U256::from(1_000_000u64),
		&calls,
		84532,
	)
	.unwrap();

	// Assert — independent `cast` golden.
	core::assert_eq!(
		digest,
		alloy::primitives::b256!(
			"38f3e40326f93eb1bac6808e16be993221b90a0b9691e2d71a8df485337c9faf"
		),
	);
}

#[test]
fn implementation_version_changes_batch_digest() {
	// Arrange
	let account = alloy::primitives::address!("279640887C3806d4FBd424bb0B58F0430CE661C1");
	let calls = [BatchCall {
		target: alloy::primitives::address!("1b42FF8DdB251074637d3A9872D72f51e3AbB23d"),
		value: U256::ZERO,
		data: Bytes::from_static(&[0xab, 0xcd, 0xef]),
	}];

	// Act
	let version_one = batch_digest_local(
		account,
		INITIAL_ACCOUNT_IMPLEMENTATION_VERSION,
		U256::ZERO,
		U256::from(1_000_000u64),
		&calls,
		84532,
	)
	.unwrap();
	let version_two = batch_digest_local(
		account,
		INITIAL_ACCOUNT_IMPLEMENTATION_VERSION + 1,
		U256::ZERO,
		U256::from(1_000_000u64),
		&calls,
		84532,
	)
	.unwrap();

	// Assert
	core::assert_eq!(
		version_two,
		alloy::primitives::b256!(
			"ef59acebcf91cc2bdfc563df2cd7cae236a6e7bca829f773ae9359295df8257b"
		),
	);
	core::assert_ne!(version_one, version_two);
}
