# Magicblock AI Oracle — On-Chain DeFi Credit Agent Example

This repository demonstrates how to integrate with the **solana-gpt-oracle** to enable **AI-powered reputation scoring** directly from a Solana program using **Anchor**.
It showcases how an on-chain smart contract can request and receive AI inference results through a secure callback mechanism.

The example implements a **DeFi Credit Agent**, which:

- Analyzes a user’s off-chain signals (ex: Twitter bio) using an AI model
- Produces a **DeFi Credit Score** from 0–100
- Stores the score inside a PDA tied to the user
- Supports future scoring updates via secure interactions

---

## Features

- Secure CPI calls to the **Solana GPT Oracle**
- PDA-based **Agent** account that stores oracle context
- PDA-based **CredScore** state derived from user wallet
- AI inference triggered by on-chain instruction
- **Callback flow** verifies signer identity before storing results
- Score validation and clamping (0–100)

---

## Program Architecture

```
src/
├── lib.rs                     # Entrypoint, CPI logic, callback handler
tests/
└── magicblock-solana-ai-oracle.ts   # Full request→callback flow
```

---

## PDA Seeds

### Agent (Oracle context binding)

```
["agent"]
```

### Cred Score (per user)

```
["cred", user_pubkey]
```

Stored state:

```rust
pub struct CredScore {
    pub score: u8, // 0–100
    pub bump: u8,
}
```

---

## Instruction Overview

| Instruction           | Who Signs       | Description                                                 |
| --------------------- | --------------- | ----------------------------------------------------------- |
| `initialize`          | Signer          | Creates and registers an Agent PDA with GPT Oracle          |
| `interact_agent`      | User            | Sends input text to AI model and requests on-chain callback |
| `callback_from_agent` | Oracle identity | Validates response + stores normalized score                |

---

## AI Pipeline

1. **Base Layer** triggers AI:

   ```rust
   interact_with_llm(...)
   ```

   - Sends `text` to oracle
   - Includes callback discriminator
   - Passes PDAs as writable metadata for update

2. **Oracle Off-Chain Execution**

   - AI computes DeFi Credit Score
   - Returns result to callback instruction

3. **Callback Handler (on-chain)**

   ```rust
   let parsed_score: u8 = response.trim().parse::<u8>()?;
   cred_score_account.score = parsed_score.min(100);
   ```

---

## Security Notes

- AI callback requires **oracle identity signer**
- PDA bumps validated on all state accounts
- Writable privilege limited only to `cred_score`
- Agent PDA ties directly to Oracle context for safety

---

## Local Testing Workflow

End-to-end test included:

- Locates Oracle Counter + Context PDA
- Runs `initialize`
- Triggers `interactAgent` with real prompt text
- (Callback requires oracle execution on actual network)

Run tests:

```sh
anchor test
```

---

## Example RPC Calls

Build the program:

```sh
anchor build
```

Deploy:

```sh
solana program deploy target/deploy/magicblock_solana_ai_oracle.so
or
anchor deploy
```

Get test output logs:

```sh
ANCHOR_VERBOSITY=3 anchor test
```

---

## Learning Outcomes

This example demonstrates:

- How Solana programs integrate with AI inference oracles
- Secure CPI pattern for asynchronous callback workflows
- Flexible PDA state design for user-owned AI data
- Scaling off-chain intelligence while keeping on-chain verification
