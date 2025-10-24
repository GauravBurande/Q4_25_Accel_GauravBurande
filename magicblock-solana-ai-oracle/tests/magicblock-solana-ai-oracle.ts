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

  it("Is initialized!", async () => {
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
    const twitter_context = `@inspiration_gx is a Solana Turbine graduate building full-stack blockchain products with Rust, React, Node.js, and TypeScript, while serving as DevRel for SuperteamNG in Ekiti and founding Gildore on Sol. They host the "The Anchor Founder" podcast to publicly document tackling advanced Solana challenges like token extensions, LiteSVM testing, and Pinocchio implementations through live coding streams. Active in the Turbin3 Tribe working groups, their contributions emphasize open-source experimentation and community collaboration on tools like transfer hooks and escrow vaults."How far are you willing to expand your mind?" -@inspiration_gx You've swapped encouraging replies on debugging woes and token extensions progress, with them hyping your LiteSVM tests and you dubbing them chief content creator at Solana Turbine during a podcast shoutout.`;

    const prompt = `You are a DeFi Credit Agent. Analyze the following Twitter context and output a single DeFi Credit Score (0–100) as an integer, based on the user's credibility, technical expertise, and community trust.\n\n\
        Twitter context:\n\"${twitter_context}\"\n\n\
        Only return the score. Do not provide explanations, text, or any extra information.`;
    const tx = await program.methods
      .interactAgent("")
      .accounts({
        interaction: interactionAddress,
        contextAccount: agent.context,
        user: provider.wallet.publicKey,
      })
      .rpc();
    console.log("Your transaction signature ", tx);
  });
});
