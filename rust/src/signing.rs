#![cfg(feature = "signing")]

//! Domain-bound EIP-712 order signing and verification.

mod hashable;
mod tests;

use core::num::NonZeroU64;

use self::hashable::Order;

use alloy::{
	hex,
	primitives::{Address, B256, Signature, SignatureError, U256},
	signers::{
		SignerSync,
		local::{LocalSignerError, PrivateKeySigner},
	},
	sol_types::{Eip712Domain, SolStruct},
};

use crate::{
	batch_signing::compose::ComposeError,
	batches::{self, BatchValidationError, WireValueError},
	ids::{OrderHash, OrderType, Side, TimeInForce, TokenId},
	models::{OrderField, OrderValidationError, SignedOrderRequest},
	units::Micro,
};

const DOMAIN_NAME: &str = "Agara CTF Exchange";
const DOMAIN_VERSION: &str = "1";
const SIDE_BUY: u8 = 0;
const SIDE_SELL: u8 = 1;

/// Domain component that cannot be zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainField {
	/// EVM chain identifier.
	ChainId,

	/// Exchange contract verifying an order signature.
	ExchangeContract,

	/// Account owning the order.
	Maker,

	/// Account executing a batch.
	Account,

	/// Installed account implementation version.
	ImplementationVersion,

	/// Expected holder externally owned account.
	Holder,
}

/// Structural reason that an account-batch call cannot be signed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallFailure {
	/// An EVM call must name a nonzero target.
	ZeroTarget,

	/// The account contract rejects calls targeting itself.
	SelfTarget,
}

/// A cryptographic or domain validation failure with its original typed source retained.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SigningError {
	/// A required signing-domain component is zero.
	#[error("signing domain requires nonzero {0:?}")]
	Domain(DomainField),

	/// The public and signed order envelopes do not satisfy the order contract.
	#[error(transparent)]
	Order(#[from] OrderValidationError),

	/// The batch violates operation, binding, or deadline policy.
	#[error(transparent)]
	Batch(#[from] BatchValidationError),

	/// Canonical call composition failed.
	#[error(transparent)]
	Compose(#[from] ComposeError),

	/// A call target violates account execution rules.
	#[error("invalid batch call {index}: {reason:?}")]
	Call {
		/// Zero-based call index.
		index: usize,

		/// Exact target constraint that failed.
		reason: CallFailure,
	},

	/// Operating-system entropy was unavailable while creating a random salt.
	#[error("operating-system entropy is unavailable")]
	Entropy(#[source] rand::rand_core::OsError),

	/// The private key could not be decoded; its input is never included in the error message.
	#[error("invalid private key")]
	PrivateKey(#[source] LocalSignerError),

	/// The configured local signer could not produce a signature.
	#[error("cryptographic signing failed")]
	Signer(#[source] alloy::signers::Error),

	/// Signature parsing or public-key recovery failed.
	#[error("signature recovery failed")]
	Signature(#[source] SignatureError),

	/// Uint256 decoding failed after structural checks.
	#[error("Uint256 decoding failed")]
	Uint256(#[source] alloy::primitives::ruint::ParseError),

	/// ABI calldata must retain its unambiguous hexadecimal prefix.
	#[error("batch calldata must be 0x-prefixed")]
	CallDataPrefix,

	/// ABI calldata contains malformed hexadecimal.
	#[error("batch calldata hexadecimal decoding failed")]
	CallData(#[source] hex::FromHexError),

	/// The supplied order hash differs from the deployment-bound EIP-712 hash.
	#[error("order hash differs from the deployment-bound digest")]
	OrderHashMismatch,

	/// The recovered signer differs from the expected account holder.
	#[error("signature does not recover to the expected holder")]
	HolderMismatch,
}

/// A validated chain and exchange address for the `Agara CTF Exchange` signing domain.
///
/// Domain construction cannot be bypassed by deserializing raw fields:
/// ```compile_fail
/// let _: agara_sdk::signing::EngineDomain = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Copy, Debug, dissolve_derive::Dissolve)]
#[dissolve(visibility = "pub(crate)")]
pub struct EngineDomain {
	chain_id: NonZeroU64,
	exchange_contract: Address,
}

/// A validated, immutable signed order. Build a checked wire envelope with `to_request_body`.
#[derive(Clone, Debug, dissolve_derive::Dissolve)]
#[dissolve(visibility = "pub(crate)")]
pub struct SignedOrder {
	order_hash: B256,
	signature: Signature,
	salt: U256,
	maker: Address,
	token_id: U256,
	maker_amount: U256,
	taker_amount: U256,
	side: SignedSide,
	timestamp: U256,
	metadata: B256,
	builder: B256,
	price_micro: Micro,
	shares_micro: Micro,
}

/// Sign a LIMIT order using exact integer micro-units and the nine-field exchange order type.
///
/// Rejects invalid domain/maker/salt, nonpositive shares, price outside `(0, 1)`, and notionals
/// outside the platform's $0.10–$100,000 bounds. Market-specific tick rules remain server-authoritative.
/// The private key is parsed locally and is never included in an error message.
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
	#[builder(default)] timestamp: U256,
	#[builder(default)] metadata: B256,
	#[builder(default)] builder: B256,
) -> Result<SignedOrder, SigningError> {
	let collateral = crate::models::limit_collateral(price_micro, shares_micro)?;
	let side = SignedSide::new(side)?;
	if deposit_wallet_address.is_zero() {
		return Err(SigningError::Domain(DomainField::Maker));
	}

	let salt = match salt {
		Some(salt) => salt,
		None => random_salt()?,
	};
	if salt.is_zero() {
		return Err(OrderValidationError::Field {
			field: OrderField::Salt,
			source: WireValueError::Zero,
		}
		.into());
	}

	let shares = U256::from(shares_micro.raw().unsigned_abs());
	let collateral = U256::from(collateral.unsigned_abs());
	let (maker_amount, taker_amount) = match side {
		SignedSide::Buy => (collateral, shares),
		SignedSide::Sell => (shares, collateral),
	};
	let order = Order {
		salt,
		maker: deposit_wallet_address,
		tokenId: token_id,
		makerAmount: maker_amount,
		takerAmount: taker_amount,
		side: side.raw(),
		timestamp,
		metadata,
		builder,
	};
	let order_hash = order.eip712_signing_hash(&domain.as_eip712());
	let signature = sign_digest(private_key, &order_hash)?;

	Ok(SignedOrder {
		order_hash,
		signature,
		salt,
		maker: deposit_wallet_address,
		token_id,
		maker_amount,
		taker_amount,
		side,
		timestamp,
		metadata,
		builder,
		price_micro,
		shares_micro,
	})
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SignedSide {
	Buy,
	Sell,
}

fn random_salt() -> Result<U256, SigningError> {
	loop {
		let mut bytes = [0; 32];
		rand::TryRngCore::try_fill_bytes(&mut rand::rngs::OsRng, &mut bytes)
			.map_err(SigningError::Entropy)?;
		let salt = U256::from_be_bytes(bytes);
		if !salt.is_zero() {
			return Ok(salt);
		}
	}
}

pub(crate) fn sign_digest(private_key: &str, digest: &B256) -> Result<Signature, SigningError> {
	let signer: PrivateKeySigner = private_key.parse().map_err(SigningError::PrivateKey)?;

	signer.sign_hash_sync(digest).map_err(SigningError::Signer)
}

pub(crate) fn decode_uint256(value: &str) -> Result<U256, SigningError> {
	U256::from_str_radix(value, 10).map_err(SigningError::Uint256)
}

fn request_uint256(value: &str, field: OrderField) -> Result<U256, SigningError> {
	let value = batches::uint256_decimal(value)
		.map_err(|source| OrderValidationError::Field { field, source })?;

	decode_uint256(value)
}

impl EngineDomain {
	/// Require a nonzero chain ID and a nonzero exchange contract; no unchecked deserializer exists.
	pub fn new(chain_id: u64, exchange_contract: Address) -> Result<Self, SigningError> {
		let chain_id =
			NonZeroU64::new(chain_id).ok_or(SigningError::Domain(DomainField::ChainId))?;
		if exchange_contract.is_zero() {
			return Err(SigningError::Domain(DomainField::ExchangeContract));
		}

		Ok(Self { chain_id, exchange_contract })
	}

	/// Chain ID cryptographically bound into every order digest.
	pub fn chain_id(&self) -> u64 {
		self.chain_id.get()
	}

	/// Nonzero exchange address cryptographically bound into every order digest.
	pub fn exchange_contract(&self) -> Address {
		self.exchange_contract
	}

	fn as_eip712(&self) -> Eip712Domain {
		let domain = (*self).dissolve();

		Eip712Domain {
			name: Some(DOMAIN_NAME.into()),
			version: Some(DOMAIN_VERSION.into()),
			chain_id: Some(U256::from(domain.chain_id.get())),
			verifying_contract: Some(domain.exchange_contract),
			salt: None,
		}
	}
}

#[bon::bon]
impl SignedOrder {
	/// The immutable EIP-712 order identity used for deduplication and reconciliation.
	pub fn order_hash(&self) -> OrderHash {
		OrderHash::from_bytes(self.order_hash.0)
	}

	/// Construct a validated submission without allowing its economics to differ from the signature.
	///
	/// GTD expiry is checked against the local clock. The signed timestamp, metadata and builder
	/// bytes are copied exactly, and canonical 27/28 recovery parity is emitted.
	#[builder]
	pub fn to_request_body(
		&self,
		token_id: TokenId,
		side: Side,
		price_micro: Micro,
		shares_micro: Micro,
		time_in_force: TimeInForce,
		post_only: bool,
		expiration_unix_seconds: Option<i64>,
	) -> Result<SignedOrderRequest, SigningError> {
		let signed = self.clone().dissolve();
		let requested_token = request_uint256(token_id.as_str(), OrderField::TokenId)?;
		if requested_token != signed.token_id
			|| price_micro != signed.price_micro
			|| shares_micro != signed.shares_micro
			|| SignedSide::new(side)?.raw() != signed.side.raw()
		{
			return Err(OrderValidationError::SignedEconomicsMismatch.into());
		}

		let request = SignedOrderRequest {
			token_id,
			side,
			order_type: OrderType::Limit,
			time_in_force,
			price_micro: signed.price_micro,
			shares_micro: signed.shares_micro,
			post_only,
			expiration_unix_seconds,
			order_hash: OrderHash::from_bytes(signed.order_hash.0),
			signature: hex::encode_prefixed(signed.signature.as_bytes()),
			salt: signed.salt.to_string(),
			maker: hex::encode_prefixed(signed.maker),
			chain_token_id: signed.token_id.to_string(),
			maker_amount: signed.maker_amount.to_string(),
			taker_amount: signed.taker_amount.to_string(),
			side_u8: signed.side.raw(),
			timestamp: signed.timestamp.to_string(),
			metadata: hex::encode_prefixed(signed.metadata),
			builder: hex::encode_prefixed(signed.builder),
		};
		request.validate()?;

		Ok(request)
	}
}

impl SignedOrderRequest {
	/// Validate the envelope, recompute its deployment-bound hash, and recover the expected holder.
	///
	/// `holder` is the EOA authorized by the maker account, not the maker account address itself.
	pub fn verify(&self, domain: EngineDomain, holder: Address) -> Result<(), SigningError> {
		self.validate()?;
		if holder.is_zero() {
			return Err(SigningError::Domain(DomainField::Holder));
		}

		let order = Order {
			salt: request_uint256(&self.salt, OrderField::Salt)?,
			maker: Address::from(batches::address_bytes(&self.maker).map_err(|source| {
				OrderValidationError::Field { field: OrderField::Maker, source }
			})?),
			tokenId: request_uint256(&self.chain_token_id, OrderField::ChainTokenId)?,
			makerAmount: request_uint256(&self.maker_amount, OrderField::MakerAmount)?,
			takerAmount: request_uint256(&self.taker_amount, OrderField::TakerAmount)?,
			side: self.side_u8,
			timestamp: request_uint256(&self.timestamp, OrderField::Timestamp)?,
			metadata: B256::from(batches::decode_hex(&self.metadata).map_err(|source| {
				OrderValidationError::Field { field: OrderField::Metadata, source }
			})?),
			builder: B256::from(batches::decode_hex(&self.builder).map_err(|source| {
				OrderValidationError::Field { field: OrderField::Builder, source }
			})?),
		};
		let digest = order.eip712_signing_hash(&domain.as_eip712());
		let supplied = batches::decode_hex::<32>(self.order_hash.as_str()).map_err(|source| {
			OrderValidationError::Field { field: OrderField::OrderHash, source }
		})?;
		if digest.0 != supplied {
			return Err(SigningError::OrderHashMismatch);
		}

		let signature = batches::signature_bytes(&self.signature).map_err(|source| {
			OrderValidationError::Field { field: OrderField::Signature, source }
		})?;
		let recovered = Signature::from_raw(&signature)
			.map_err(SigningError::Signature)?
			.recover_address_from_prehash(&digest)
			.map_err(SigningError::Signature)?;
		if recovered != holder {
			return Err(SigningError::HolderMismatch);
		}

		Ok(())
	}
}

impl SignedSide {
	fn new(side: Side) -> Result<Self, OrderValidationError> {
		match side {
			Side::Buy => Ok(Self::Buy),
			Side::Sell => Ok(Self::Sell),
			Side::Unspecified => Err(OrderValidationError::Side),
		}
	}

	fn raw(self) -> u8 {
		match self {
			Self::Buy => SIDE_BUY,
			Self::Sell => SIDE_SELL,
		}
	}
}
