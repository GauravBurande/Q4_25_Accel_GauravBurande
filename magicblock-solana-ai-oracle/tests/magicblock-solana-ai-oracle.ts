import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MagicblockSolanaAiOracle } from "../target/types/magicblock_solana_ai_oracle";
import { PublicKey } from "@solana/web3.js";

describe("magicblock-solana-ai-oracle", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const llmProgramAddress = new PublicKey(
    "LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab"
  );

  const getLLMProgram = async () => {
    const llmProgramIDL = await Program.fetchIdl(llmProgramAddress, provider);
    const llmProgram: any = new Program(llmProgramIDL);
    return llmProgram;
  };

  const GetAgentAndInteraction = async () => {
    const [agentAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent")],
      program.programId
    );

    const agent = await program.account.agent.fetch(agentAddress);

    const [interactionAddress] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("interaction"),
        provider.wallet.publicKey.toBuffer(),
        agent.context.toBuffer(),
      ],
      llmProgramAddress
    );

    return { agent, interactionAddress };
  };

  const provider = anchor.getProvider();
  const connection = provider.connection;

  const program = anchor.workspace
    .magicblockSolanaAiOracle as Program<MagicblockSolanaAiOracle>;

  xit("Is initialized!", async () => {
    const llmProgram: any = await getLLMProgram();

    const [counterAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("counter")],
      llmProgramAddress
    );

    const counter = await llmProgram.account.counter.fetch(counterAddress);

    const [llmContext] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("test-context"),
        new anchor.BN(counter.count).toArrayLike(Buffer, "le", 4),
      ],
      llmProgramAddress
    );

    const tx = await program.methods
      .initialize()
      .accounts({
        counter: counterAddress,
        llmContext,
        signer: provider.wallet.publicKey,
      })
      .rpc();
    console.log("Your transaction signature", tx);
    console.log("Your count", counter.count);
  });

  it("get ur DeFi cred score", async () => {
    // it's more like ur web3 aura score
    const { agent, interactionAddress } = await GetAgentAndInteraction();
    const twitter_context = `@inspiration_gx is a Solana Turbine graduate building full-stack blockchain products with Rust, React, Node.js, and TypeScript. They serve as DevRel for SuperteamNG in Ekiti and founded Gildore on Sol. Host of "The Anchor Founder" podcast, documenting advanced Solana work on token extensions, LiteSVM testing, and Pinocchio integrations. Active in Turbin3 working groups, they focus on open-source tools like transfer hooks and escrow vaults. Known for supportive replies, collaboration, and strong community trust.`;

    const prompt = `You are a DeFi Credit Agent. Analyze the following Twitter context and output a single DeFi Credit Score (0–100) as an integer based on credibility, technical expertise, and community trust.\n\nTwitter context:\n"${twitter_context}"\n\nOnly return the score. Do not include any explanation or extra text.`;

    const tx = await program.methods
      .interactAgent(prompt)
      .accounts({
        interaction: interactionAddress,
        contextAccount: agent.context,
        user: provider.wallet.publicKey,
      })
      .rpc();
    console.log("Your transaction signature ", tx);
  });
});
