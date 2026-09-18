//! Typed deposit and withdrawal quote contracts.

/// Address family required by a bridge-supported asset.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortfolioBridgeAddressType {
	/// EVM-compatible address.
	Evm,

	/// Bitcoin address.
	Btc,

	/// Solana address.
	Sol,
}

/// Trading wallet and provider-issued addresses for cross-chain deposits.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeDepositAddressResponse {
	/// Trading deposit-wallet address to which the quoted bridge route ultimately credits collateral.
	pub wallet_address: String,

	/// Provider-issued deposit addresses for the supported EVM, Solana and Bitcoin routes.
	pub deposit_addresses: PortfolioBridgeDepositAddresses,

	/// Informational note from the bridge provider; None when omitted.
	pub note: Option<String>,
}

/// Provider-issued deposit addresses, separated by chain address family.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeDepositAddresses {
	/// EVM deposit address.
	pub evm: String,

	/// Solana deposit address.
	pub svm: String,

	/// Bitcoin deposit address.
	pub btc: String,
}

/// Bridge assets currently supported for the caller’s trading wallet.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeSupportedAssetsResponse {
	/// Trading deposit-wallet address for which these supported routes were resolved.
	pub wallet_address: String,

	/// Currently supported provider chain/token combinations for this wallet.
	pub supported_assets: Vec<PortfolioBridgeSupportedAsset>,

	/// Informational note from the bridge provider; None when omitted.
	pub note: Option<String>,
}

/// One supported chain/token combination and its minimum checkout value.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeSupportedAsset {
	/// Provider chain identifier; preserve its text namespace rather than assuming an EVM integer.
	pub chain_id: String,

	/// Human-readable provider name for the supported chain.
	pub chain_name: String,

	/// Address type to use for this asset.
	pub address_type: PortfolioBridgeAddressType,

	/// Token identity and native decimal precision for this supported route.
	pub token: PortfolioBridgeSupportedToken,

	/// Minimum checkout value as exact decimal USD text.
	pub min_checkout_usd: String,
}

/// Provider token identity and native base-unit precision.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeSupportedToken {
	/// Human-readable name supplied by the catalogue or provider.
	pub name: String,

	/// Provider token ticker, such as USDC; distinct from a security or feed symbol.
	pub symbol: String,

	/// Provider-native token address on the supported chain.
	pub address: String,

	/// Native token decimal places used to interpret base-unit amounts.
	pub decimals: u8,
}

/// Request a deposit estimate using source-chain native token units.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeDepositQuoteRequest {
	/// Provider source-chain identifier for the deposit route.
	pub from_chain_id: String,

	/// Source token address in the bridge provider’s chain-specific format.
	pub from_token_address: String,

	/// Positive integer source-token amount encoded as text in native base units.
	pub from_amount_base_unit: String,
}

/// Estimated bridge output, timing and fees; a quote does not execute a transfer.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeDepositQuoteResponse {
	/// Estimated checkout time in milliseconds.
	pub est_checkout_time_ms: u64,

	/// Provider fee/slippage estimates; these values do not confirm an executed transfer.
	pub est_fee_breakdown: PortfolioBridgeEstimatedFeeBreakdown,

	/// Estimated source-token value as exact decimal USD text.
	pub est_input_usd: String,

	/// Estimated destination-token value as exact decimal USD text.
	pub est_output_usd: String,

	/// Estimated destination quantity as integer text in the destination token’s native base units.
	pub est_to_token_base_unit: String,

	/// Opaque provider quote identifier, distinct from an execution or transaction ID.
	pub quote_id: String,
}

/// Request a withdrawal estimate for a destination chain, token and recipient.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeWithdrawQuoteRequest {
	/// Provider destination-chain identifier for the withdrawal route.
	pub to_chain_id: String,

	/// Destination token address in the bridge provider’s chain-specific format.
	pub to_token_address: String,

	/// Recipient in the destination chain’s address format; it need not be an EVM address.
	pub recipient_address: String,

	/// Positive integer source collateral amount encoded as text in native base units.
	pub from_amount_base_unit: String,
}

/// Provider decimal-string fee estimates, retaining the provider’s units and precision.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]

pub struct PortfolioBridgeEstimatedFeeBreakdown {
	/// Provider’s human-readable label for the application fee.
	pub app_fee_label: String,

	/// Application fee percentage as provider decimal text.
	pub app_fee_percent: String,

	/// Application fee estimate as decimal USD text.
	pub app_fee_usd: String,

	/// Bridge fill-cost percentage as provider decimal text.
	pub fill_cost_percent: String,

	/// Bridge fill-cost estimate as decimal USD text.
	pub fill_cost_usd: String,

	/// Gas estimate as decimal USD text.
	pub gas_usd: String,

	/// Provider’s maximum-slippage estimate as exact decimal text.
	pub max_slippage: String,

	/// Provider’s minimum-received estimate, preserved as decimal text in its reported units.
	pub min_received: String,

	/// Provider’s swap-impact measure preserved as exact decimal text.
	pub swap_impact: String,

	/// Estimated swap impact as decimal USD text.
	pub swap_impact_usd: String,

	/// Provider’s aggregate impact measure preserved as exact decimal text.
	pub total_impact: String,

	/// Estimated aggregate impact as decimal USD text.
	pub total_impact_usd: String,
}
