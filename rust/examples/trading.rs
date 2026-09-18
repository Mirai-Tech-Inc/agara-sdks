//! Place a post-only limit order, cancel it, and reconcile the final state.
//!
//!     export AGARA_TOKEN="agt_..."
//!     export AGARA_TOKEN_ID="21742..."
//!     cargo run --example trading

use core::time::Duration;

use agara_sdk::{
	AgaraClient, DEFAULT_BASE_URL,
	ids::{Side, TokenId},
};

#[tokio::main]
async fn main() -> agara_sdk::Result<()> {
	let base_url = std::env::var("AGARA_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());
	let token = std::env::var("AGARA_TOKEN").expect("set AGARA_TOKEN");
	let token_id = TokenId::new(std::env::var("AGARA_TOKEN_ID").expect("set AGARA_TOKEN_ID"))?;

	let client = AgaraClient::builder().token(token.into()).base_url(base_url.into()).build()?;

	let book = client.get_orderbook(&token_id).await?;
	std::println!("best bid {:?} / ask {:?}", book.best_bid(), book.best_ask());

	let ack = client
		.place_limit_order()
		.token_id(token_id.clone())
		.side(Side::Buy)
		.price(rust_decimal_macros::dec!(0.10))
		.shares(rust_decimal_macros::dec!(1))
		.post_only(true)
		.call()
		.await?;
	std::println!("placed {} — {:?}", ack.order_id, ack.status);

	client.cancel_order(&ack.order_id).await?;
	std::println!("cancel requested");

	let order = client
		.wait_for_terminal(
			&ack.order_id,
			Duration::from_secs(10),
			Duration::from_secs(1),
		)
		.await?;
	std::println!("final status: {:?}", order.status);

	Ok(())
}
