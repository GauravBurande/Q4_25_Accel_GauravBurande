# Pinocchio Escrow Program — Solana Token Escrow Example

This repository demonstrates a non-Anchor Solana escrow program built with the Pinocchio framework.
The program enables two parties to trustlessly exchange SPL tokens using a Program Derived Address (PDA) escrow vault.

It includes:

- A custom `Escrow` state struct stored inside an onchain PDA account
- Secure checks for account owners, mint correctness, and signer validations
- PDA-based authority using signed instructions and seeds
- Full integration tests using `LiteSVM` for local execution

---

## Features

- Creates an escrow vault owned by a PDA
- Moves tokens from the maker into escrow
- Allows cancel by maker before it is taken
- Allows taker to exchange their tokens for the escrowed tokens
- Closes vault and escrow account upon success or cancellation
- Lightweight runtime using `pinocchio` instead of Anchor

---

## Project Layout

```
src/
 ├── instructions/
 │   ├── make.rs       # Initialize escrow + deposit maker tokens
 │   ├── take.rs       # Complete trade, taker deposits & receives escrow tokens
 │   ├── cancel.rs     # Maker cancels escrow & retrieves tokens
 │   └── mod.rs
 ├── state/
 │   ├── escrow.rs     # Account layout and accessors
 │   └── mod.rs
 ├── tests/
 │   └── mod.rs        # LiteSVM e2e tests for make, cancel, take
 └── lib.rs            # Entrypoint + instruction dispatch
```

---

## Onchain Program

The program routes incoming instructions to handlers based on a `u8` discriminator:

```rust
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match EscrowInstructions::try_from(discriminator)? {
        EscrowInstructions::Make => process_make_instruction(accounts, data)?,
        EscrowInstructions::Take => process_take_instruction(accounts)?,
        EscrowInstructions::Cancel => process_cancel_instruction(accounts)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    }
    Ok(())
}
```

### Instruction Overview

| Instruction | Who Signs | Result                                                |
| ----------- | --------- | ----------------------------------------------------- |
| `Make`      | Maker     | Creates PDA escrow account and vault, deposits tokens |
| `Take`      | Taker     | Executes token swap, closes PDA + vault               |
| `Cancel`    | Maker     | Returns escrowed tokens to maker, closes PDA + vault  |

### Escrow PDA Seeds

```
["escrow", maker_pubkey, bump]
```

A bump byte is stored inside the account to enable signed operations later.

---

## State: Escrow Account

```rust
#[repr(C)]
pub struct Escrow {
    maker: [u8; 32],
    mint_a: [u8; 32],
    mint_b: [u8; 32],
    amount_to_receive: [u8; 8],
    amount_to_give: [u8; 8],
    pub bump: u8,
}
```

This layout is validated to match exactly the byte length and alignment expected onchain.

---

## Local Testing with LiteSVM

Unit tests spin up a local Solana virtual environment, deploy the compiled `.so`, mint tokens, and execute full escrow flows.

Example from `tests/mod.rs`:

```rust
#[test]
pub fn test_make_instruction() {
    let (mut svm, payer, mint_a, mint_b, maker_ata_a, escrow, vault, associated_token_program, token_program, system_program) = setup();

    let (escrow_pda, bump) = Pubkey::find_program_address(
        &[b"escrow".as_ref(), payer.pubkey().as_ref()],
        &PROGRAM_ID.parse().unwrap(),
    );

    let tx = svm.send_transaction(
        build_make_instruction(
            &svm,
            &payer,
            bump,
            mint_a,
            mint_b,
            escrow_pda,
            maker_ata_a,
            vault,
            system_program,
            token_program,
            associated_token_program,
        )
    ).unwrap();
}
```

Tests confirm:

- PDA derivation is correct
- Account owners and mint IDs are validated
- Transfers and vault lifecycle operate correctly

---

## Building and Deploying

1. Build the program

```sh
cargo build-bpf
//or
cargo-build-sbf
```

2. Deploy the program to Devnet or local validator

```sh
solana program deploy target/deploy/escrow.so
```

3. Test

```sh
cargo test -- --nocapture
```

---

## Instruction Encoding

Instruction data layout:

```
Make:
[ discriminator (u8) | bump (u8) | amount_to_receive (u64) | amount_to_give (u64) ]

Take:
[ discriminator (u8) ]

Cancel:
[ discriminator (u8) ]
```

---

## Security Considerations

- Only the maker can cancel
- Taker must sign to execute a swap
- ATA owner and mint types are validated
- PDA is asserted before read/write
- All lamports in PDA are returned upon closing
