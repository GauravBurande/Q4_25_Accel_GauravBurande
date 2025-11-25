import { CrossbarClient } from "@switchboard-xyz/common";

const crossbar = CrossbarClient.default();

/**
 * Print out the results of a feed simulation to the console and return them
 * @param feeds - the feed public keys encoded as base58 strings
 * @returns results - the output of each job in each feed
 */
async function printFeedResults(
  feeds: string[]
): Promise<{ feed: string; results: number[]; feedHash: string }[]> {
  const results = await crossbar.simulateSolanaFeeds(
    "devnet", // network "mainnet" | "devnet"
    feeds // feed pubkeys as base58
  );

  for (let simulation of results) {
    console.log(
      `Feed Public Key ${simulation.feed} job outputs: ${
        // simulation.results // results array is empty, weird!
        simulation.result // results array is empty, weird!
      }`
    );
  }

  return results;
}

setInterval(async () => {
  const umbraFeed = "9n3z7h9FsrdN9FQgBWayeEMsZiGy4kewinoMvBACzRfh";
  const results = await printFeedResults([umbraFeed]);
}, 1000 * 10);
