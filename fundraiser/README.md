# Pinocchio Fundraiser Program — Solana Token Crowdfunding Example

This repository implements a **token-based crowdfunding (fundraiser)** program using the **Pinocchio framework** on Solana.
The program enables a project creator (maker) to raise SPL tokens from contributors within a fixed time window. Funds are held inside a PDA-owned vault until either:

- The target goal is reached → Maker **collects** all funds
- The campaign expires before success → Contributors **refund** their share

---

## Features

- Program Derived Address (PDA) for **fundraiser state**
- PDA **Contributor state** for tracking individual pledges
- Secure SPL token transfers using CPI
- Time-based fundraising using Clock sysvar
- Safe constraints:

  - Minimum raise amount
  - Max contribution percentage per contributor
  - Duration enforced on collect/refund

- Full integration tests using **LiteSVM**

---

## Project Layout

```
src/
├── instructions/
│   ├── initialize.rs   # Create fundraiser state + vault ATA
│   ├── contribute.rs   # Transfer tokens into fundraiser vault
│   ├── refund.rs       # Contributor withdraws before success
│   ├── collect.rs      # Maker collects after success + duration
│   └── mod.rs
├── state/
│   ├── fundraiser.rs   # Fundraiser account layout + PDA checks
│   ├── contributor.rs  # Contributor account layout + PDA checks
│   └── mod.rs
├── tests/
│   ├── mod.rs          # e2e testing with LiteSVM
│   └── constant.rs
└── lib.rs              # Entrypoint + instruction dispatch
```

---

## Onchain Instruction Routing

```rust
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &crate::ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundInstructions::try_from(discriminator)? {
        FundInstructions::Initialize => process_initialize(accounts, data)?,
        FundInstructions::Contribute => process_contribute(accounts, data)?,
        FundInstructions::Refund => process_refund(accounts, data)?,
        FundInstructions::Collect => process_collect(accounts, data)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}
```

### Instruction Overview

| Instruction | Who Signs   | Result                                                                |
| ----------- | ----------- | --------------------------------------------------------------------- |
| Initialize  | Maker       | Creates fundraiser state PDA and vault ATA                            |
| Contribute  | Contributor | Transfers SPL tokens into vault, tracking individual amount           |
| Refund      | Contributor | Withdraws contribution before goal success OR before campaign expires |
| Collect     | Maker       | Receives all funds after goal success and duration end                |

---

## PDA Seeds

### Fundraiser PDA

```
["fundraiser", maker_pubkey, bump]
```

### Contributor PDA

```
["contributor", contributor_pubkey, bump]
```

---

## State Accounts

### Fundraiser State

```rust
#[repr(C)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 1],  // days
    pub bump: u8,
}
```

### Contributor State

```rust
#[repr(C)]
pub struct Contributor {
    pub amount: [u8; 8],
}
```

---

## Local Testing with LiteSVM

Tests simulate:

- PDA derivation and initialization
- Multiple contributors
- Time travel using sysvar modification
- Token lifecycle: contribute → refund / collect

Example:

```rust
#[test]
pub fn test_contribute_instruction() {
    let (mut svm, payer, mint, ata, fundraiser, vault, _, token_program, system_program) = setup();

    let tx_init = build_init_transaction(...);
    svm.send_transaction(tx_init).unwrap();

    let tx = build_contribute_transaction(
        &mut svm,
        &payer,
        1_000_000,
        mint,
        ata,
        fundraiser,
        vault,
        program_id(),
        token_program,
        system_program,
        associated_token_program,
    );

    let result = svm.send_transaction(tx).unwrap();
    println!("{}", result.pretty_logs());
}
```

---

## Build & Deploy

1. Build program

```sh
cargo build-bpf
```

2. Deploy program

```sh
solana program deploy target/deploy/fundraiser.so
```

3. Test

```sh
cargo test -- --nocapture
```

---

## Instruction Encoding

| Instruction | Layout                                               |
| ----------- | ---------------------------------------------------- |
| Initialize  | `[0, bump, amount_to_raise (u64 LE), duration (u8)]` |
| Contribute  | `[1, bump, amount (u64 LE)]`                         |
| Refund      | `[2, bump]`                                          |
| Collect     | `[3, bump]`                                          |

---

## Security Considerations

- PDA validation on every access
- All Signers required where necessary
- Duration enforced before collect
- Max per-contributor limits to avoid maker self-funding
- Safe handling of PDA lamports on close (refund)
