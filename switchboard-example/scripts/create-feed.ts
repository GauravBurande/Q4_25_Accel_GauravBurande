import {
  AnchorUtils,
  PullFeed,
  getDefaultQueue, // gets the mainnet queue
  getDefaultDevnetQueue, // better use devnet for development
  asV0Tx,
} from "@switchboard-xyz/on-demand";
import { CrossbarClient, OracleJob } from "@switchboard-xyz/common";

const jobs: OracleJob[] = [
  new OracleJob({
    tasks: [
      {
        jupiterSwapTask: {
          baseAmountString: "1",
          inTokenAddress: "PRVT6TB7uss3FrUd2D9xs2zqDBsa3GbMJMwCQsgmeta", // Umbra MINT Address
          outTokenAddress: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC MINT Address
          slippage: 20,
        },
      },
      //   {
      //     jsonParseTask: {
      //       path: "$.result",
      //     },
      //   }, // no need of the json parser here, jupiter SwapTask does it all
    ],
  }),
];
console.log("Running simulation...\n");

// Print the jobs that are being run.
const jobJson = JSON.stringify({ jobs: jobs.map((job) => job.toJSON()) });
console.log(jobJson);
console.log();

// Serialize the jobs to base64 strings.
const serializedJobs = jobs.map((oracleJob) => {
  const encoded = OracleJob.encodeDelimited(oracleJob).finish();
  const base64 = Buffer.from(encoded).toString("base64");
  return base64;
});

// Call the simulation server.
const response = await fetch("https://api.switchboard.xyz/api/simulate", {
  method: "POST",
  headers: [["Content-Type", "application/json"]],
  body: JSON.stringify({ cluster: "Mainnet", jobs: serializedJobs }), // also worked on devnet, surprising! // maybe both the mints are on devnet too!
});

// Check response.
if (response.ok) {
  const data = await response.json();
  console.log(`Response is good (${response.status})`);
  console.log(JSON.stringify(data, null, 2));
} else {
  console.log(`Response is bad (${response.status})`);
  console.log(await response.text());
}

let devnetRPC = "https://api.devnet.solana.com"; // rate limited
let queue = await getDefaultDevnetQueue(devnetRPC);

const crossbarClient = CrossbarClient.default();
const { feedHash } = await crossbarClient.store(queue.pubkey.toBase58(), jobs);

const keypairRoute = "/Users/gauravburande/.config/solana/id.json";
const payer = AnchorUtils.initKeypairFromFile(keypairRoute);
console.log("Using Payer:", payer.publicKey.toBase58(), "\n");

const [pullFeed, feedKeypair] = PullFeed.generate(queue.program);

// Get the initialization for the pull feeds
const ix = await pullFeed.initIx({
  name: "Umbra price feed", // the feed name (max 32 bytes)
  jobs,
  queue: queue.pubkey, // the queue of oracles to bind to
  maxVariance: 1.0, // the maximum variance allowed for the feed results
  minResponses: 1, // minimum number of responses of jobs to allow
  feedHash: Buffer.from(feedHash.slice(2), "hex"), // the feed hash
  minSampleSize: 1, // The minimum number of samples required for setting feed value
  maxStaleness: 60, // The maximum number of slots that can pass before a feed value is considered stale.
  payer: payer.publicKey, // the payer of the feed
});

// Generate VersionedTransaction
const tx = await asV0Tx({
  connection: queue.program.provider.connection,
  ixs: [ix],
  payer: payer.publicKey,
  signers: [payer, feedKeypair],
  computeUnitPrice: 75_000,
  computeUnitLimitMultiple: 1.3,
});

// simulate the txn
const simulateResult =
  await queue.program.provider.connection.simulateTransaction(tx, {
    commitment: "processed",
  });
console.log(simulateResult);

// Send transaction to validator
const sig = await queue.program.provider.connection.sendTransaction(tx, {
  preflightCommitment: "processed",
  skipPreflight: true,
});

// Finished!
console.log(`Feed ${feedKeypair.publicKey} initialized: ${sig}`);

export const feedAddress = "9n3z7h9FsrdN9FQgBWayeEMsZiGy4kewinoMvBACzRfh"; /// on devnet cluster

// txn sig:
// 4YSc7xM7BH6eCzSX2CH9y7mxDx5pdJ6QNegvDCHHqyw81zobbEarQGmEYGt59X15AGf8TzJ7owioB4NPipmi6veS
