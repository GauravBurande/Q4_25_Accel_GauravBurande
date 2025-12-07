# Anchor Escrow Program — SPL Token Escrow With Freeze Logic

This repository implements a secure **SPL token escrow system** on Solana using the **Anchor framework**. It enables a *maker* to deposit tokens into a PDA vault and a *taker* to redeem them after a configured freeze period, completing a trustless token swap between two parties.

The program includes:

* PDA-owned escrow account with strict owner and mint validation
* Freeze period (slot-based) enforcement before taker can execute swap
* Secure token transfers using SPL Token Interface
* Automatic closing of escrow + vault accounts on success or refund
* Full **local e2e tests** powered by **LiteSVM**

---

## Features

* Initialize escrow and **deposit tokens** into a PDA-authorized vault
* Restrict redemption until **freeze period** has elapsed
* Taker deposits their side of the trade upon execution
* Token swap completes atomically
* Vault and escrow accounts **closed** to reclaim lamports
* Maker can **refund** as long as taker has not executed swap

---

## Project Structure

```text
programs/anchor-escrow/
└── src
    ├── instructions/
    │   ├── make.rs     # Initialize escrow + deposit maker tokens
    │   ├── refund.rs   # Maker cancels escrow + withdraws tokens
    │   ├── take.rs     # Taker deposits and redeems escrow tokens
    │   └── mod.rs
    ├── state/
    │   ├── escrow.rs   # Escrow struct + persistent data
    │   └── mod.rs
    ├── tests/
    │   ├── mod.rs      # LiteSVM setup + make/refund/take tests
    │   ├── error.rs
    │   └── lib.rs
    ├── lib.rs          # Program entrypoints
Cargo.toml
Anchor.toml
```

---

## PDA Seeds

Escrow account address:

```rust
[
    b"escrow",
    maker.key().as_ref(),
    seed.to_le_bytes().as_ref()
]
```

The bump is stored inside the escrow account to enable PDA-signed operations during withdrawal and closing.

Vault ATA is automatically created with:

```
mint_a + escrow PDA as authority
```

---

## On-Chain Instruction Logic

### `make`

| Who Signs | Result                                           |
| --------- | ------------------------------------------------ |
| Maker     | Creates escrow + vault and deposits maker tokens |

```rust
pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64, freeze: u32)
```

Actions:

* Initialize escrow account
* Save mint addresses, seed, freeze window
* Transfer `deposit` tokens to PDA vault

---

### `refund`

| Who Signs | Result                                    |
| --------- | ----------------------------------------- |
| Maker     | Cancels escrow and retrieves vault tokens |

```rust
pub fn refund(ctx: Context<Refund>)
```

* Only maker can call
* PDA signs for vault withdrawal
* Vault + escrow closed and lamports reclaimed

---

### `take`

| Who Signs | Result                           |
| --------- | -------------------------------- |
| Taker     | Completes swap and closes escrow |

```rust
pub fn take(ctx: Context<Take>)
```

Checks and execution:

* Freeze period must have passed
* Taker deposits tokens to maker
* PDA transfers all vault tokens to taker
* Vault + escrow closed

---

## Freeze Period Enforcement

```rust
let current_slot = Clock::get()?.slot;
let ends_at = escrow.created_at + escrow.freeze_period as u64;
require!(current_slot >= ends_at, EscrowError::FreezePeriodNotOver);
```

This prevents taker from executing trade before the time-lock expires.

---

## State Layout

```rust
#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub created_at: u64,
    pub freeze_period: u32,
    pub bump: u8,
}
```

---

## Local Testing with LiteSVM

All escrow flows are tested:

| Test          | Validates                          |
| ------------- | ---------------------------------- |
| `test_make`   | PDA creation, vault funding        |
| `test_refund` | Maker reclaim + account closure    |
| `test_take`   | Freeze period unlock + atomic swap |

Example from `test_make`:

```rust
let tx = program.send_transaction(transaction).unwrap();
assert_eq!(vault_data.amount, 10);
assert_eq!(escrow_data.receive, 10);
```

Tests emulate sysvars like Clock to validate freeze logic.

---

## Build & Deploy

Install Anchor and Solana tooling, then:

```sh
anchor build
anchor test
```

To deploy:

```sh
solana program deploy target/deploy/anchor_escrow.so
```

---

## Security Controls

* Maker authorization enforced for cancel
* Token mint correctness validated via `has_one` constraints
* PDA authority required for vault token movement
* All lamports from closed accounts returned to rightful owners
* Freeze period binding ensures fair execution timing