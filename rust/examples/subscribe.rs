//! Stream a market's public channels plus your private account events.
//!
//!     export AGARA_TOKEN="agt_..."
//!     cargo run --example subscribe -- <token_id> <condition_id>

use agara_sdk::ids::{ConditionId, TokenId};
use agara_sdk::{AgaraStreamClient, Channel, Frame};

#[tokio::main]
async fn main() -> agara_sdk::Result<()> {
	let mut args = std::env::args().skip(1);
	let token_id = TokenId::new(args.next().expect("usage: subscribe <token_id> <condition_id>"));
	let condition_id = ConditionId::new(args.next().expect("missing <condition_id>"));
	let token = std::env::var("AGARA_TOKEN").ok();

	let mut client = AgaraStreamClient::builder().maybe_token(token).build();
	client.subscribe([
		Channel::Orderbook(token_id.clone()),
		Channel::BestQuote(token_id),
		Channel::Trades(condition_id),
		Channel::AccountEvents,
	])?;

	let mut stream = client.connect().await?;
	while let Some(frame) = stream.next().await {
		match frame {
			Frame::OrderbookSnapshot(s) => {
				println!(
					"[snapshot] seq={} bids={} asks={}",
					s.sequence,
					s.bids.len(),
					s.asks.len()
				);
			},
			Frame::BestQuote(q) => println!("[best_quote] bid={:?} ask={:?}", q.bid, q.ask),
			Frame::Trade(t) => println!(
				"[trade] {:?} {}@{} mode={:?}",
				t.side, t.size, t.price, t.settlement_mode
			),
			Frame::Fill(f) => println!("[fill {:?}] {} order={}", f.role, f.fill_id, f.order_id),
			Frame::SequenceReset(r) => eprintln!("[reset] {} — discard local state", r.channel),
			Frame::Error(e) => eprintln!("[error] {}: {}", e.code, e.message),
			_ => {},
		}
	}
	Ok(())
}
