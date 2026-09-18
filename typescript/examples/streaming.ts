import { marketStream, priceStream } from "@agara/sdk/streaming";

export async function watchMarket(tokenId: string, signal: AbortSignal) {
  const stream = marketStream({ channels: [{ name: "orderbook", token_id: tokenId }], signal });
  for await (const event of stream) {
    if (event.type === "gap") {
      console.log("Discard local state and reconcile:", event.recovery);
    } else if (event.type === "frame") {
      console.log(event.frame);
    }
  }
}

export async function watchPrice(signal: AbortSignal) {
  for await (const event of priceStream(["Crypto.BTC/USD"], { signal })) {
    console.log(event.frame.parsed);
  }
}
