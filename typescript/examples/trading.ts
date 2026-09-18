import { type AgaraClient, PublicClient, parseUnits } from "@agara/sdk";
import { signOrder } from "@agara/sdk/signing";
import { privateKeyToAccount } from "viem/accounts";

const publicClient = new PublicClient();
const markets = await publicClient.listMarkets({ source: "agara", limit: 10 });
console.log(markets);

export async function placeSignedLimit(
  client: AgaraClient,
  key: `0x${string}`,
  maker: `0x${string}`,
  exchangeContract: `0x${string}`,
  chainId: number,
  tokenId: string,
) {
  const body = await signOrder(
    {
      domain: { chainId, exchangeContract },
      maker,
      tokenId,
      side: "BUY",
      priceMicro: parseUnits("0.60"),
      sharesMicro: parseUnits("2.00"),
    },
    privateKeyToAccount(key),
  );
  const accepted = await client.placeSignedOrder(body);
  return client.waitForOrder(accepted.order_id);
}
