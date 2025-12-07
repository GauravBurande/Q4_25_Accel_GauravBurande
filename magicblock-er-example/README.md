# Ephemeral Rollup State Account — Solana Example with Magicblock

This repository implements a **stateful user account program** using **Anchor** on Solana, enhanced with **Magicblock Ephemeral Rollups** for scalable off-chain state execution. It demonstrates how to:

- Create and manage a **user-owned PDA state account**
- **Delegate** the account to an Ephemeral Rollup for fast state updates
- **Commit** ephemeral state updates back to the Solana base layer
- Use **VRF (Verifiable Random Function)** for provably random updates on both

  - Base Layer
  - Ephemeral Rollup Layer

- **Undelegate** and eventually **close** the state account

This example is useful for developers exploring hybrid architectures where high-throughput execution happens off-chain while settlement and final security remain on Solana.

---

## Features

- Program Derived Account (PDA) per user
- Offload state computation to Magicblock Ephemeral Rollup
- Two-way sync of state:

  - `update` → Base layer only
  - `update_commit` → Ephemeral → Committed to base layer

- Secure delegation and undelegation primitives
- VRF-powered randomness on both layers
- Safe closing of accounts and lamport reclaim

---

## Program Layout

```
src/
├── instructions/
│   ├── init_user.rs             # Create PDA + initialize state
│   ├── update_user.rs           # Base-layer state mutation
│   ├── update_commit.rs         # ER execution + commit to base
│   ├── delegate.rs              # Delegate PDA to ER validator
│   ├── undelegate.rs            # Commit + detach from ER
│   ├── close_user.rs            # Close PDA and reclaim lamports
│   ├── update_user_with_random.rs
│   │                             # VRF callback handler
│   └── mod.rs
├── state/
│   ├── user_account.rs          # PDA struct + bump validation
│   └── mod.rs
└── lib.rs                       # Entrypoint + instruction routing
tests/
└── er-state-account.ts          # End-to-end ER workflow tests
```

---

## PDA Seeds

### User Account

```
["user", user_pubkey]
```

The PDA stores:

```rust
pub struct UserAccount {
    pub user: Pubkey,
    pub data: u64,
    pub bump: u8,
}
```

---

## Instruction Overview

| Instruction       | Layer            | Who Signs | Result                                          |
| ----------------- | ---------------- | --------- | ----------------------------------------------- |
| `initialize`      | Base             | User      | Creates PDA and initializes state               |
| `update`          | Base             | User      | Mutate state on-chain                           |
| `delegate`        | Base             | User      | Delegates PDA to Ephemeral Rollup validator     |
| `update_commit`   | Ephemeral Rollup | User (ER) | Update ephemeral state and commit proof to Base |
| `vrf_random_er`   | Ephemeral Rollup | User (ER) | Random update using ER VRF                      |
| `vrf_random_base` | Base             | User      | Random update using Base VRF                    |
| `undelegate`      | Ephemeral Rollup | User (ER) | Commit latest ER state and return to base-only  |
| `close`           | Base             | User      | Close PDA, return lamports to user              |

---

## Magicblock VRF Integration

VRF workflows request randomness from an oracle queue and pass the response via program callback:

```rust
pub fn update_user(&mut self, rnd_u8: u8) -> Result<()> {
    self.user_account.data = rnd_u8 as u64;
    Ok(())
}
```

Guaranteed integrity of randomness ensures trust-minimized logic on both Base + ER layers.

---

## Local + ER Test Workflow

The included TypeScript tests show the full lifecycle:

- Create PDA
- Mutate base-layer state
- Delegate to ER
- Update & commit using ER execution
- Random updates via VRF
- Undelegate and return to base-only
- Close account and clean up

Example:

```sh
anchor test
```

Tests internally use:

- Base layer RPC via AnchorProvider
- Ephemeral Rollup RPC via Magicblock SDK
- `GetCommitmentSignature` for finalization proofs

---

## Build & Deploy

Build the program

```sh
anchor build
```

Deploy to Solana

```sh
solana program deploy target/deploy/er_state_account.so
or
anchor deploy
```

View local test logs (recommended)

```sh
ANCHOR_VERBOSITY=3 anchor test
```

---

## Security Notes

- PDA bump enforced for state validation
- Signer constraints ensure only account owner can act
- Delegation limited to explicitly passed validator accounts
- Closing uses `close = user` to safely return lamports
- VRF identity restricted to Magicblock VRF Program

---

## Learning Outcomes

This example helps developers understand:

- Hybrid rollup + L1 state models
- State concurrency and safety using PDAs
- Provable execution using commit + undelegate flows
- VRF-powered randomness for games, lotteries, etc.
