//! Place a resting limit order, wait for it to settle, print the result.
//!
//!     export AGARA_TOKEN="agt_..."
//!     export AGARA_TOKEN_ID="21742..."
//!     cargo run --example trading

use core::time::Duration;

use agara_sdk::ids::{Side, TokenId};
use agara_sdk::{AgaraClient, DEFAULT_BASE_URL};
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> agara_sdk::Result<()> {
	let base_url = std::env::var("AGARA_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());
	let token = std::env::var("AGARA_TOKEN").expect("set AGARA_TOKEN");
	let token_id = TokenId::new(std::env::var("AGARA_TOKEN_ID").expect("set AGARA_TOKEN_ID"));

	let client = AgaraClient::builder().token(token).base_url(base_url).build()?;

	let book = client.get_orderbook(&token_id).await?;
	println!("best bid {:?} / ask {:?}", book.best_bid(), book.best_ask());

	let ack = client
		.place_limit_order()
		.token_id(token_id.clone())
		.side(Side::Buy)
		.price(dec!(0.01))
		.shares(dec!(1))
		.call()
		.await?;
	println!("placed {} — {:?}", ack.order_id, ack.status);

	let order = client
		.wait_for_terminal(
			&ack.order_id,
			Duration::from_secs(10),
			Duration::from_secs(1),
		)
		.await?;
	println!("final status: {:?}", order.status);

	client.cancel_order(&ack.order_id).await?;
	println!("cancel requested");
	Ok(())
}
