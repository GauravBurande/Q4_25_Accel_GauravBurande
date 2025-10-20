import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MagicblockSolanaAiOracle } from "../target/types/magicblock_solana_ai_oracle";

describe("magicblock-solana-ai-oracle", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.getProvider();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .magicblockSolanaAiOracle as Program<MagicblockSolanaAiOracle>;

  const PROGRAM_ID = program.programId;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
