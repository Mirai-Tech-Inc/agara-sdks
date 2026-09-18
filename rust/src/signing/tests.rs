#![cfg(test)]

use super::*;

// Fixed vector shared with chain-client's eip712 golden and the
// Python SDK's signing test.
const ACCOUNT: Address = alloy::primitives::address!("279640887C3806d4FBd424bb0B58F0430CE661C1");
const EXCHANGE: Address = alloy::primitives::address!("1b42FF8DdB251074637d3A9872D72f51e3AbB23d");
const CHAIN_ID: u64 = 84532;
const GOLDEN_HASH: B256 =
	alloy::primitives::b256!("f5cbbd057896816a0be9a705a90bf1f094b01af05f977658791b3e65c862c961");

#[test]
fn order_hash_matches_cross_repo_golden() {
	// Arrange — the fixed order from chain-client's eip712 golden.
	let order = Order {
		salt: U256::from(1u64),
		maker: ACCOUNT,
		tokenId: U256::from(2u64),
		makerAmount: U256::from(100u64),
		takerAmount: U256::from(100u64),
		side: 0,
		timestamp: U256::ZERO,
		metadata: B256::ZERO,
		builder: B256::ZERO,
	};
	let domain = Eip712Domain {
		name: Some(DOMAIN_NAME.into()),
		version: Some(DOMAIN_VERSION.into()),
		chain_id: Some(U256::from(CHAIN_ID)),
		verifying_contract: Some(EXCHANGE),
		salt: None,
	};

	// Act
	let hash = order.eip712_signing_hash(&domain);

	// Assert — equals the cast-derived digest shared with chain-client.
	core::assert_eq!(hash, GOLDEN_HASH);
}

#[test]
fn sign_limit_order_produces_recoverable_signature() {
	// Arrange — deterministic key; recover the signer to prove the
	// 65-byte (r||s||v) signature is well-formed and eth-shaped.
	let key = "0x1111111111111111111111111111111111111111111111111111111111111111";
	let signer: PrivateKeySigner = key.parse().unwrap();

	// Act
	let signed = sign_limit_order()
		.private_key(key)
		.domain(EngineDomain::new(CHAIN_ID, EXCHANGE).unwrap())
		.deposit_wallet_address(signer.address())
		.token_id(U256::from(2u64))
		.side(Side::Buy)
		.price_micro(Micro::new(500_000))
		.shares_micro(Micro::new(2_000_000))
		.salt(U256::from(1u64))
		.call()
		.unwrap();

	// Assert
	let hash = signed.order_hash;
	let bytes = signed.signature.as_bytes();
	core::assert_eq!(bytes.len(), 65);
	core::assert!(bytes[64] == 27 || bytes[64] == 28);
	let sig = alloy::primitives::Signature::from_raw(&bytes).unwrap();
	let recovered = sig.recover_address_from_prehash(&hash).unwrap();
	core::assert_eq!(recovered, signer.address());
}

#[test]
fn signed_request_rejects_changed_economics_and_zero_salt() {
	// Arrange
	let key = "0x1111111111111111111111111111111111111111111111111111111111111111";
	let domain = EngineDomain::new(CHAIN_ID, EXCHANGE).unwrap();
	let make = || {
		sign_limit_order()
			.private_key(key)
			.domain(domain)
			.deposit_wallet_address(ACCOUNT)
			.token_id(U256::from(2))
			.side(Side::Buy)
			.price_micro(Micro::new(500_000))
			.shares_micro(Micro::new(2_000_000))
	};
	// Act
	let zero = make().salt(U256::ZERO).call();
	let signed = make()
		.salt(U256::from(1))
		.timestamp(U256::from(123))
		.metadata(B256::from([1; 32]))
		.builder(B256::from([2; 32]))
		.call()
		.unwrap();
	let mismatch = signed
		.to_request_body()
		.token_id(TokenId::new("2").unwrap())
		.side(Side::Buy)
		.price_micro(Micro::new(400_000))
		.shares_micro(Micro::new(2_000_000))
		.time_in_force(TimeInForce::Gtc)
		.post_only(false)
		.call();
	let request = signed
		.to_request_body()
		.token_id(TokenId::new("2").unwrap())
		.side(Side::Buy)
		.price_micro(Micro::new(500_000))
		.shares_micro(Micro::new(2_000_000))
		.time_in_force(TimeInForce::Gtc)
		.post_only(false)
		.call()
		.unwrap();
	// Assert
	core::assert!(zero.is_err());
	core::assert!(mismatch.is_err());
	core::assert_eq!(request.timestamp, "123");
	core::assert_eq!(request.metadata, alloy::hex::encode_prefixed([1; 32]));
	core::assert_eq!(request.builder, alloy::hex::encode_prefixed([2; 32]));
}
