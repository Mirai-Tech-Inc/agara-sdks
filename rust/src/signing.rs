//! EIP-712 order signing for the agara CTF exchange (feature `signing`).
//!
//! Mirrors `crates/chain-client/src/eip712.rs`: domain
//! `("Agara CTF Exchange", "1")` and a 10-field `Order` (no
//! `signatureType`). The digest produced here is byte-for-byte the one
//! the on-chain `CTFExchange.hashOrder` view and the maker
//! `AgaraAccount.isValidSignature` verify, so a bot's pre-signed order
//! validates on-chain. LIMIT orders only — MARKET orders continue
//! through the Privy-signed path.

use alloy::hex;
use alloy::primitives::{Address, B256, U256};
use alloy::signers::SignerSync;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use alloy::sol_types::{Eip712Domain, SolStruct};

use crate::error::{AgaraError, Result};
use crate::ids::{OrderHash, Side, TimeInForce, TokenId};
use crate::models::SignedOrderRequest;
use crate::units::{MICRO, Micro};

const DOMAIN_NAME: &str = "Agara CTF Exchange";
const DOMAIN_VERSION: &str = "1";

/// Off-chain wire sentinel: the on-chain Order dropped `signatureType`,
/// but the request still carries `signature_type = 3` so the router
/// routes the ERC-1271 smart-account path.
const SIGNATURE_KIND_ERC1271: u8 = 3;

const SIDE_BUY: u8 = 0;
const SIDE_SELL: u8 = 1;

const ZERO_BYTES32: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

sol! {
	/// EIP-712 typed shape of the CTFExchange Order — field order is
	/// load-bearing for the typehash; mirror the contract exactly.
	struct Order {
		uint256 salt;
		address maker;
		address signer;
		uint256 tokenId;
		uint256 makerAmount;
		uint256 takerAmount;
		uint8 side;
		uint256 timestamp;
		bytes32 metadata;
		bytes32 builder;
	}
}

/// EIP-712 domain binding a signature to a specific engine deployment.
#[derive(Clone, Copy, Debug)]
pub struct EngineDomain {
	/// The chain the exchange contract lives on.
	pub chain_id: u64,
	/// The exchange contract that verifies the signature via ERC-1271.
	pub exchange_contract: Address,
}

/// Output of [`sign_limit_order`]. Serialize with
/// [`SignedOrder::to_request_body`] for
/// [`crate::AgaraClient::place_signed_order`].
#[derive(Clone, Debug)]
pub struct SignedOrder {
	order_hash: B256,
	signature: String,
	salt: U256,
	maker: Address,
	signer: Address,
	token_id: U256,
	maker_amount: U256,
	taker_amount: U256,
	side: u8,
}

impl SignedOrder {
	/// The EIP-712 order hash — your correlation key before the router
	/// assigns an `order_id`.
	pub fn order_hash(&self) -> OrderHash {
		OrderHash::new(hex::encode_prefixed(self.order_hash))
	}

	/// Build the request body for `POST /trade/v1/orders/signed`.
	/// `token_id` is the human-facing outcome id the router looks up; the
	/// chain envelope carries the same value as a u256 decimal string.
	#[allow(clippy::too_many_arguments)]
	pub fn to_request_body(
		&self,
		token_id: TokenId,
		side: Side,
		price_micro: Micro,
		shares_micro: Micro,
		time_in_force: TimeInForce,
		post_only: bool,
		expiration_unix_seconds: Option<i64>,
	) -> SignedOrderRequest {
		SignedOrderRequest {
			token_id,
			side,
			order_type: crate::ids::OrderType::Limit,
			time_in_force,
			price_micro,
			shares_micro,
			post_only,
			expiration_unix_seconds,
			order_hash: self.order_hash(),
			signature: self.signature.clone(),
			salt: self.salt.to_string(),
			maker: hex::encode_prefixed(self.maker),
			signer: hex::encode_prefixed(self.signer),
			chain_token_id: self.token_id.to_string(),
			maker_amount: self.maker_amount.to_string(),
			taker_amount: self.taker_amount.to_string(),
			side_u8: self.side,
			signature_type: SIGNATURE_KIND_ERC1271,
			timestamp: "0".to_owned(),
			metadata: ZERO_BYTES32.to_owned(),
			builder: ZERO_BYTES32.to_owned(),
		}
	}
}

/// Sign a LIMIT order. `deposit_wallet_address` is both maker and signer
/// on the envelope (the AgaraAccount address); the holder EOA behind
/// `private_key` signs the order hash flat, and `isValidSignature`
/// recovers it on-chain.
#[bon::builder]
pub fn sign_limit_order(
	private_key: &str,
	domain: EngineDomain,
	deposit_wallet_address: Address,
	token_id: U256,
	side: Side,
	price_micro: Micro,
	shares_micro: Micro,
	salt: Option<U256>,
) -> Result<SignedOrder> {
	let price = price_micro.raw();
	let shares = shares_micro.raw();
	if price <= 0 || price >= MICRO {
		return Err(AgaraError::Validation(
			"price_micro must be in (0, 1_000_000)".to_owned(),
		));
	}

	if shares <= 0 {
		return Err(AgaraError::Validation(
			"shares_micro must be > 0".to_owned(),
		));
	}

	let collateral = (shares as i128 * price as i128) / MICRO as i128;
	if collateral <= 0 {
		return Err(AgaraError::Validation(
			"collateral rounds to zero — order too small".to_owned(),
		));
	}

	let (side_u8, maker_amount, taker_amount) = match side {
		Side::Buy => (SIDE_BUY, collateral as u64, shares as u64),
		Side::Sell => (SIDE_SELL, shares as u64, collateral as u64),
		Side::Unspecified => {
			return Err(AgaraError::Validation(
				"side must be BUY or SELL".to_owned(),
			));
		},
	};

	let salt = salt.unwrap_or_else(|| U256::from(rand::random::<u128>()));
	let maker_amount = U256::from(maker_amount);
	let taker_amount = U256::from(taker_amount);

	let order = Order {
		salt,
		maker: deposit_wallet_address,
		signer: deposit_wallet_address,
		tokenId: token_id,
		makerAmount: maker_amount,
		takerAmount: taker_amount,
		side: side_u8,
		timestamp: U256::ZERO,
		metadata: B256::ZERO,
		builder: B256::ZERO,
	};
	let eip712_domain = Eip712Domain {
		name: Some(DOMAIN_NAME.into()),
		version: Some(DOMAIN_VERSION.into()),
		chain_id: Some(U256::from(domain.chain_id)),
		verifying_contract: Some(domain.exchange_contract),
		salt: None,
	};
	let order_hash = order.eip712_signing_hash(&eip712_domain);

	let signer: PrivateKeySigner = private_key
		.parse()
		.map_err(|e| AgaraError::Validation(format!("invalid private key: {e}")))?;
	let signature = signer
		.sign_hash_sync(&order_hash)
		.map_err(|e| AgaraError::Validation(format!("signing failed: {e}")))?;

	let mut bytes = signature.as_bytes();
	if bytes[64] < 27 {
		bytes[64] += 27;
	}

	Ok(SignedOrder {
		order_hash,
		signature: hex::encode_prefixed(bytes),
		salt,
		maker: deposit_wallet_address,
		signer: deposit_wallet_address,
		token_id,
		maker_amount,
		taker_amount,
		side: side_u8,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	use alloy::primitives::{address, b256};

	// Fixed vector shared with chain-client's eip712 golden and the
	// Python SDK's signing test.
	const ACCOUNT: Address = address!("279640887C3806d4FBd424bb0B58F0430CE661C1");
	const EXCHANGE: Address = address!("1b42FF8DdB251074637d3A9872D72f51e3AbB23d");
	const CHAIN_ID: u64 = 84532;
	const GOLDEN_HASH: B256 =
		b256!("ac2e7042ba6818b2d031497def0160b752d3e8c08954df173681685a777891b3");

	#[test]
	fn order_hash_matches_cross_repo_golden() {
		// Arrange — the fixed order from chain-client's eip712 golden.
		let order = Order {
			salt: U256::from(1u64),
			maker: ACCOUNT,
			signer: ACCOUNT,
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
		assert_eq!(hash, GOLDEN_HASH);
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
			.domain(EngineDomain { chain_id: CHAIN_ID, exchange_contract: EXCHANGE })
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
		let bytes = hex::decode(signed.signature.trim_start_matches("0x")).unwrap();
		assert_eq!(bytes.len(), 65);
		assert!(bytes[64] == 27 || bytes[64] == 28);
		let sig = alloy::primitives::Signature::from_raw(&bytes).unwrap();
		let recovered = sig.recover_address_from_prehash(&hash).unwrap();
		assert_eq!(recovered, signer.address());
	}
}
