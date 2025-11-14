import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FeedOnchain } from "../target/types/feed_onchain";
import { PublicKey } from "@solana/web3.js";

describe("feed_onchain", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.feedOnchain as Program<FeedOnchain>;

  const provider = anchor.getProvider();
  it("logs the price of Umbra!", async () => {
    const slot = await provider.connection.getSlot();
    const feed = new PublicKey("9n3z7h9FsrdN9FQgBWayeEMsZiGy4kewinoMvBACzRfh");
    const tx = await program.methods
      .test(new anchor.BN(slot))
      .accounts({
        feed,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
