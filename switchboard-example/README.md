Here’s a **clean, production-ready README.md** for your repo based on the full context you shared — including onchain Anchor code, Switchboard On-Demand feed creation, job simulation, and feed consumption.

---

# 📡 Switchboard On-Demand Price Feed — Solana + Anchor Example

This repository demonstrates **how to create and consume a Switchboard On-Demand Pull Feed** for real-time pricing on Solana.
It includes:

- A **Switchboard On-Demand job** (Jupiter SwapTask) that fetches the **Umbra → USDC** price
- A **TypeScript client script** to simulate, register, and initialize the feed on Devnet
- An **Anchor onchain program** showing how to parse and read the Pull Feed account
- Example usage + common debugging notes (e.g., _NotEnoughSamples_)

---

## ✨ Features

- ✔️ Creates On-Demand feeds using `@switchboard-xyz/on-demand`
- ✔️ Uses **JupiterSwapTask** to fetch token swap quotes
- ✔️ Simulates feed results before publishing them
- ✔️ Pushes jobs to Switchboard Crossbar and initializes the feed on Solana
- ✔️ Anchor program parses `PullFeedAccountData` directly onchain
- ✔️ Example showing how to call `get_value` inside a Solana instruction

---

## 📁 Project Structure

```
.
├── program/                   # Anchor program (onchain)
│   └── src/lib.rs             # Reads Switchboard PullFeed onchain
├── scripts/
│   └── create-feed.ts         # Creates + simulates + initializes PullFeed
├── Cargo.toml
├── Anchor.toml
└── README.md
```

---

# 🧩 onchain Program (Anchor)

This Anchor instruction shows how to:

- Read a Switchboard Pull Feed account
- Parse it using `PullFeedAccountData::parse`
- Query feed values with `get_value()`

```rust
use anchor_lang::prelude::*;
use switchboard_on_demand::on_demand::accounts::pull_feed::PullFeedAccountData;

declare_id!("3Co44vnKtvUd1RshCrEnFUferHi1x5sWwjYKuYco2yjU");

#[derive(Accounts)]
pub struct Test<'info> {
    /// CHECK: feed account passed by client
    pub feed: AccountInfo<'info>,
}

#[program]
pub mod feed_onchain {
    use super::*;

    pub fn test(ctx: Context<Test>, slot: u64) -> Result<()> {
        let feed_data = ctx.accounts.feed.data.borrow();
        let feed = PullFeedAccountData::parse(feed_data).unwrap();

        msg!(
            "Umbra Price: {:?}",
            feed.get_value(slot, 100, 1, true).unwrap()
        );

        Ok(())
    }
}
```

---

# 🛰️ Off-Chain Script: Creating a Switchboard On-Demand Feed

Your feed uses **Jupiter SwapTask**:

- Base amount: `1 UMBRA`
- Output token: `USDC`
- Cluster: Devnet

The script:

1. Builds the `OracleJob`
2. Simulates the feed job
3. Uploads the job to Switchboard Crossbar
4. Initializes a PullFeed account onchain

## 🔧 Example Job

```ts
const jobs: OracleJob[] = [
  new OracleJob({
    tasks: [
      {
        jupiterSwapTask: {
          baseAmountString: "1",
          inTokenAddress: "PRVT6TB7uss3FrUd2D9xs2zqDBsa3GbMJMwCQsgmeta", // Umbra mint
          outTokenAddress: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
          slippage: 20,
        },
      },
    ],
  }),
];
```

## 🧪 Run Simulation

```ts
const response = await fetch("https://api.switchboard.xyz/api/simulate", {
  method: "POST",
  headers: [["Content-Type", "application/json"]],
  body: JSON.stringify({
    cluster: "Mainnet",
    jobs: serializedJobs,
  }),
});
```

## 🏗️ Initialize the Feed

```ts
const [pullFeed, feedKeypair] = PullFeed.generate(queue.program);

const ix = await pullFeed.initIx({
  name: "Umbra price feed",
  jobs,
  queue: queue.pubkey,
  maxVariance: 1.0,
  minResponses: 1,
  feedHash: Buffer.from(feedHash.slice(2), "hex"),
  minSampleSize: 1,
  maxStaleness: 100,
  payer: payer.publicKey,
});
```

---

# 🚀 Usage

### 1️⃣ Install Dependencies

```
bun install
anchor build
anchor deploy (on Devnet)
```

### 2️⃣ Create the feed (Devnet)

```
bun feed
```

This will:

- Simulate the job
- Upload to Crossbar
- Initialize the feed
- Output the feed public key

### 3️⃣ Call the Anchor test instruction

```
anchor test --skip-build --skip-local-validator --skip-deploy
```

or manually:

```
solana program invoke --program <PROGRAM_ID> --accounts "feed=<FEED_PUBKEY>" ...
```

---

# ⚠️ Common Issues

### ❗ `NotEnoughSamples`

Switchboard tasks require **≥ minSampleSize** responses.

Fix:

- Set `minSampleSize: 1`
- Ensure the queue is active (Devnet queue sometimes slow)
- Use `minResponses: 1`
- Ensure your job doesn’t error (e.g., invalid Jupiter tokens on Devnet)

### ❗ Feed returns `None`

Check that:

- You use the **correct slot** in `get_value()`
- Feed is fresh (`maxStaleness`)
- Feed is initialized and updated at least once
