import { OracleJob } from "@switchboard-xyz/common";

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
