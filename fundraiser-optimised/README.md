# Fundraiser - Optimised

SPL Token fundraising program built with [Pinocchio](https://github.com/anza-xyz/pinocchio), a minimal-overhead Solana framework using unsafe Rust for maximum performance.

## Overview

Same fundraiser logic as the other implementations, but optimized for minimal compute unit (CU) consumption. Uses raw pointer arithmetic for state access, zero-copy deserialization, and Pinocchio's lightweight CPI abstractions. No Borsh serialization overhead.

## Program ID

```
22222222222222222222222222222222222222222222
```

## Architecture

### Entrypoint

Uses Pinocchio's `entrypoint!` macro. Discriminator is the first byte of instruction data:

```rust
match FundraiserInstruction::try_from(discriminator) {
    0 => Initialize,
    1 => Contribute,
    2 => Checker,
    3 => Refund,
}
```

### State Accounts

State is accessed via raw pointer casts -- no serialization/deserialization.

**Fundraiser** (89 bytes) - zero-copy struct wrapping `*mut u8`

| Field            | Type   | Offset | Size | Description                |
|------------------|--------|--------|------|----------------------------|
| `maker`          | Pubkey | 0      | 32   | Creator of the fundraiser  |
| `mint_to_raise`  | Pubkey | 32     | 32   | SPL token mint to collect  |
| `amount_to_raise`| u64    | 64     | 8    | Target amount              |
| `amount_raised`  | u64    | 72     | 8    | Amount collected so far    |
| `time_ending`    | i64    | 80     | 8    | Unix timestamp when it ends|
| `bump`           | u8     | 88     | 1    | PDA bump seed              |

**Contributor** (8 bytes) - zero-copy struct wrapping `*const u8`

| Field    | Type | Offset | Size | Description                    |
|----------|------|--------|------|--------------------------------|
| `amount` | u64  | 0      | 8    | Total contributed by this user |

### Instructions

#### 1. Initialize (discriminant: 0)
Writes fundraiser data directly to account memory via `unsafe` pointer operations. No serialization framework involved.

**Accounts:** fundraiser

**Data layout:** maker (32) + mint_to_raise (32) + amount_to_raise (8) + time_ending (8) + bump (1) = 81 bytes (amount_raised is initialized to 0 and not passed in data)

#### 2. Contribute (discriminant: 1)
Transfers tokens using Pinocchio's `Transfer::invoke()`. Updates fundraiser and contributor state via direct pointer writes.

**Accounts:** signer, contributor, signer_ta, fundraiser, vault, token_program

**Validation:**
- Fundraiser has not ended (`current_time <= time_ending`)
- Uses `assert!` macro for checks

#### 3. Checker (discriminant: 2)
Maker claims raised tokens via `Transfer::invoke_signed()` with PDA signer seeds.

**Accounts:** maker, maker_ta, fundraiser, vault, token_program

**Validation:**
- Fundraiser must have ended (`current_time > time_ending`)
- Fundraiser account verified via `from_account_info` (checks data_len and owner)

#### 4. Refund (discriminant: 3)
Contributor reclaims tokens. Updates state via unsafe pointer writes.

**Accounts:** fundraiser, contributor, contributor_ta, vault, token_program

**Validation:**
- Contributor amount must be > 0
- Fundraiser and contributor verified via `from_account_info`
- CPI signed by fundraiser PDA using `signer!` macro

### Optimization Techniques

- **Zero-copy state access** - `Fundraiser` and `Contributor` wrap raw pointers, fields read via `unsafe` pointer casts at known offsets
- **No serialization** - data written directly to account memory via `ptr::add()` and pointer casts
- **Pinocchio CPI** - `Transfer { from, to, authority, amount }.invoke()` instead of building instruction structs
- **`borrow_mut_data_unchecked()`** - skips runtime borrow checks for account data access
- **`from_account_info_unchecked()`** - skips owner/length checks when not needed
- **`signer!` macro** - efficient PDA signer seed construction
- **Const program ID** - `five8_const::decode_32_const` for compile-time base58 decoding

## Tech Stack

- **Framework:** Pinocchio v0.6.0
- **CPI helpers:** pinocchio-system v0.2.0, pinocchio-token v0.2.0
- **Hashing:** solana-nostd-sha256
- **Encoding:** five8 v0.2.1 / five8_const v0.1.3
- **Testing:** Mollusk SVM

## Build & Test

```bash
cargo build-sbf
cargo test
```
