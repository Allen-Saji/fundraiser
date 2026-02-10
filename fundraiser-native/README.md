# Fundraiser - Native

SPL Token fundraising program built with the native Solana program SDK (no framework).

## Overview

Same fundraiser logic as the Anchor version, implemented using raw Solana program primitives. All account validation, serialization (Borsh), and CPI calls are handled manually. This gives full control over the program at the cost of more boilerplate.

## Program ID

```
CsPs2Z5QLAuqWCw2enAmCfCNmdGahBYysL3cvxv9mmxH
```

## Architecture

### Entrypoint

The program uses a single-byte discriminator at the start of instruction data to route to the correct handler:

```rust
match FundraiserInstruction::try_from(discriminant) {
    0 => InitializeInstruction,
    1 => CheckerInstruction,
    2 => ContributeInstruction,
    3 => RefundInstruction,
}
```

### State Accounts

**Fundraiser** (89 bytes) - PDA seeded with `["fundraiser", maker_pubkey]`

| Field            | Type   | Offset | Size | Description                |
|------------------|--------|--------|------|----------------------------|
| `maker`          | Pubkey | 0      | 32   | Creator of the fundraiser  |
| `mint_to_raise`  | Pubkey | 32     | 32   | SPL token mint to collect  |
| `amount_to_raise`| u64    | 64     | 8    | Target amount              |
| `current_amount` | u64    | 72     | 8    | Amount collected so far    |
| `time_ending`    | i64    | 80     | 8    | Unix timestamp when it ends|
| `bump`           | u8     | 88     | 1    | PDA bump seed              |

**Contributor** (8 bytes) - PDA seeded with `["contributor", fundraiser_pubkey, contributor_pubkey]`

| Field    | Type | Offset | Size | Description                    |
|----------|------|--------|------|--------------------------------|
| `amount` | u64  | 0      | 8    | Total contributed by this user |

### Instructions

#### 1. Initialize (discriminant: 0)
Creates a fundraiser. Instruction data: `amount_to_raise (u64)` + `time_ending (i64)`.

**Accounts:** maker (signer), fundraiser, mint_to_raise, system_program

**Validation:**
- Maker must be signer
- Mint must be owned by SPL Token or Token-2022 program
- PDA derived and bump stored

#### 2. Contribute (discriminant: 2)
Transfers tokens from contributor to vault via CPI.

**Accounts:** signer, contributor_account, signer_ta, fundraiser, vault, token_program

**Validation:**
- Fundraiser and contributor accounts owned by this program
- Amount > 0
- Fundraiser has not ended (`current_time <= time_ending`)
- Contributor token account mint matches fundraiser mint
- Contributor has sufficient balance

#### 3. Checker (discriminant: 1)
Maker claims all raised tokens. Closes the fundraiser account (lamports returned to maker, data zeroed).

**Accounts:** maker (signer), mint_to_raise, fundraiser, vault, maker_ta, token_program

**Validation:**
- Maker must be signer
- Fundraiser PDA verified
- Vault owned by fundraiser PDA
- Maker ATA mint matches
- Vault has non-zero balance
- CPI transfer signed by PDA, then account closed manually

#### 4. Refund (discriminant: 3)
Contributor reclaims their tokens. Resets contributor amount to 0.

**Accounts:** maker, fundraiser, contributor_account, contributor_ta, vault, token_program

**Validation:**
- Fundraiser and contributor accounts owned by this program
- Contributor has positive balance
- Token mint matches
- CPI transfer signed by fundraiser PDA

### Key Differences from Anchor

- **Manual account validation** - signer checks, ownership checks, PDA verification done explicitly
- **Borsh serialization** - state structs derive `BorshSerialize`/`BorshDeserialize`
- **Manual account closing** - lamports transferred and data zeroed by hand (no `close` constraint)
- **Explicit CPI construction** - `invoke` / `invoke_signed` with manually built instructions
- **Duration stored as `time_ending`** (absolute timestamp) instead of relative `duration` + `time_started`

## Tech Stack

- **SDK:** solana-program v2.0.15
- **Serialization:** Borsh v1.5.1
- **Tokens:** SPL Token v6.0.0, SPL Token-2022 v5.0.2
- **Errors:** thiserror v2.0.0
- **Testing:** Mollusk SVM (Solana Virtual Machine)

## Build & Test

```bash
cargo build-sbf
cargo test
```
