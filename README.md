# Solana Dev Tools Examples Library

This repository is an **example collection** of practical Solana programs demonstrating a range of smart contract patterns and techniques:

- Anchor & Pinocchio frameworks
- SPL Token & Token-2022 features
- PDA security and authority patterns
- CPI token transfers and multi-program architecture
- Off-chain execution using Magicblock Ephemeral Rollups
- AI inference callbacks via solana-gpt-oracle
- On-chain price feeds via Switchboard On-Demand
- Local testing with **LiteSVM** for fast iteration

This repository is intended as a **reference library** to accelerate learning and onboarding for Solana smart contract developers.

---

## Directory Overview

| Project                                                        | Framework           | Concept Focus                                  | Related Examples                                             |
| -------------------------------------------------------------- | ------------------- | ---------------------------------------------- | ------------------------------------------------------------ |
| [`accel-pinocchio-escrow`](./accel-pinocchio-escrow)           | Pinocchio           | PDA escrow vault, trustless token swap         | `escrow-litesvm` (Anchor version)                            |
| [`escrow-litesvm`](./escrow-litesvm)                           | Anchor              | Freeze logic + secure swap enforcement         | `accel-pinocchio-escrow`                                     |
| [`fundraiser`](./fundraiser)                                   | Pinocchio           | Crowdfunding PDA state per contributor         | PDA tracking patterns in `vault-with-transfer-hook`          |
| [`magicblock-er-example`](./magicblock-er-example)             | Anchor + Magicblock | Ephemeral Rollup delegation + VRF random state | `magicblock-solana-ai-oracle` (hybrid off-chain workflows)   |
| [`magicblock-solana-ai-oracle`](./magicblock-solana-ai-oracle) | Anchor              | AI credit score via Oracle callback CPI        | `magicblock-er-example`                                      |
| [`switchboard-example`](./switchboard-example)                 | Anchor              | On-Demand price feed creation + parsing        | Oracle interaction patterns in `magicblock-solana-ai-oracle` |
| [`vault-with-transfer-hook`](./vault-with-transfer-hook)       | Anchor              | Token-2022 whitelist enforcement vault         | `whitelist-transfer-hook`                                    |
| [`whitelist-transfer-hook`](./whitelist-transfer-hook)         | Anchor              | Minimal Transfer Hook enforcement logic        | `vault-with-transfer-hook`                                   |

---

## Example Use Cases and Learning Paths

To help navigate:

### Escrow and Token Swaps

- Beginner: `accel-pinocchio-escrow` → learn PDA basics without Anchor
- Intermediate: `escrow-litesvm` → add freeze periods, Anchor security constraints

### DAO / Crowdfunding / Treasury

- `fundraiser` → multi-user state tracking, timed unlocks
- `vault-with-transfer-hook` → shared asset custody + whitelist governance

### Oracles and Hybrid Compute

- `switchboard-example` → real-time pricing with On-Demand feeds
- `magicblock-solana-ai-oracle` → asynchronous AI callbacks
- `magicblock-er-example` → scalable off-chain execution with commit proofs

### Token-2022 Advanced Features

- `whitelist-transfer-hook` → focused on validation logic
- `vault-with-transfer-hook` → integrates whitelist enforcement into vault flows

---

## Key Technologies Demonstrated

| Category                  | Technologies Used                                  |
| ------------------------- | -------------------------------------------------- |
| Smart Contract Frameworks | Anchor, Pinocchio                                  |
| Token Standards           | SPL Token, Token-2022 Transfer Hooks               |
| Off-chain Compute         | Magicblock Ephemeral Rollups, VRF                  |
| Oracle Systems            | Switchboard, solana-gpt-oracle                     |
| PDA Design                | Vault authority, Contributor PDAs, Delegation PDAs |
| Testing                   | LiteSVM local execution, cross-program tests       |

Developers can learn secure:

- PDA seed + bump enforcement
- CPI token transfers with signer seeds
- Time-locked and condition-based account access
- Multi-program dependency patterns

---

## Tooling Requirements

Install recommended dependencies globally:

```sh
solana --version
anchor --version
rustup --version
```

Depending on the example:

```sh
cargo build-bpf
anchor build
anchor test
cargo test -- --nocapture
```

---

## Why LiteSVM?

Several examples leverage **LiteSVM** to:

- Run Solana programs locally without spinning up a validator
- Simulate sysvars like Clock for time-based logic
- Debug with readable logs and deterministic state snapshots

This supports rapid development and iteration on secure PDA logic.

---

## Contributions Welcome

I encourage:

- Additional Anchor / Pinocchio examples
- More solana devtools Examples
- More Token-2022 extension use cases
- New oracle integrations
- More testing patterns

Guidelines:

- Clear documentation inside each project
- Proper PDA validation and secure CPI checks
- Full end-to-end test coverage where applicable
