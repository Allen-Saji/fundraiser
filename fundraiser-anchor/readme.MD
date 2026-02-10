# Fundraiser - Anchor

SPL Token fundraising program built with the [Anchor](https://www.anchor-lang.com/) framework on Solana.

## Overview

A maker creates a fundraiser specifying a token mint, target amount, and duration. Contributors donate SPL tokens into a PDA-owned vault. If the target is met, the maker claims all tokens. If the deadline passes without reaching the target, contributors can reclaim their tokens.

## Program ID

```
Eoiuq1dXvHxh6dLx3wh9gj8kSAUpga11krTrbfF5XYsC
```

## Architecture

### State Accounts

**Fundraiser** - PDA seeded with `["fundraiser", maker_pubkey]`

| Field            | Type    | Description                        |
|------------------|---------|------------------------------------|
| `maker`          | Pubkey  | Creator of the fundraiser          |
| `mint_to_raise`  | Pubkey  | SPL token mint to collect          |
| `amount_to_raise`| u64     | Target amount                      |
| `current_amount` | u64     | Amount collected so far            |
| `time_started`   | i64     | Unix timestamp of creation         |
| `duration`       | u8      | Campaign duration in days          |
| `bump`           | u8      | PDA bump seed                      |

**Contributor** - PDA seeded with `["contributor", fundraiser_pubkey, contributor_pubkey]`

| Field    | Type | Description                          |
|----------|------|--------------------------------------|
| `amount` | u64  | Total amount contributed by this user|

### Instructions

#### 1. `initialize(amount: u64, duration: u8)`
Creates a new fundraiser and its associated token vault (ATA owned by the fundraiser PDA).

**Accounts:** maker (signer), mint_to_raise, fundraiser (PDA, init), vault (ATA, init), system_program, token_program, associated_token_program

**Validation:**
- `amount` must be greater than `3 * 10^decimals` (MIN_AMOUNT_TO_RAISE)

#### 2. `contribute(amount: u64)`
Transfers SPL tokens from a contributor's ATA to the vault.

**Accounts:** contributor (signer), mint_to_raise, fundraiser, contributor_account (PDA, init_if_needed), contributor_ata, vault, token_program, system_program

**Validation:**
- Amount must be at least 1 token
- Amount must not exceed 10% of target (`MAX_CONTRIBUTION_PERCENTAGE`)
- Contributor's cumulative total must not exceed 10% of target
- Fundraiser must still be active (duration not elapsed)

#### 3. `check_contributions()`
Maker claims all raised tokens if the target has been met. Closes the fundraiser account and returns rent to the maker.

**Accounts:** maker (signer), mint_to_raise, fundraiser (PDA, close), vault, maker_ata (init_if_needed), token_program, system_program, associated_token_program

**Validation:**
- Vault balance must be >= `amount_to_raise`
- CPI transfer signed by the fundraiser PDA

#### 4. `refund()`
Contributors reclaim their tokens if the deadline has passed and the target was not met. Closes the contributor account and returns rent.

**Accounts:** contributor (signer), maker, mint_to_raise, fundraiser, contributor_account (PDA, close), contributor_ata, vault, token_program, system_program

**Validation:**
- Duration must have elapsed
- Vault balance must be less than target

### Constants

| Constant                     | Value  | Description                           |
|------------------------------|--------|---------------------------------------|
| `ANCHOR_DISCRIMINATOR`       | 8      | Anchor account discriminator size     |
| `MIN_AMOUNT_TO_RAISE`        | 3      | Minimum raise amount (before decimals)|
| `SECONDS_TO_DAYS`            | 86400  | Seconds per day                       |
| `MAX_CONTRIBUTION_PERCENTAGE`| 10     | Max contribution as % of target       |
| `PERCENTAGE_SCALER`          | 100    | Percentage denominator                |

## Tech Stack

- **Framework:** Anchor v0.30.0
- **Language:** Rust
- **Testing:** TypeScript + Mocha (`anchor test`)
- **Tokens:** SPL Token v0.4.6

## Build & Test

```bash
anchor build
anchor test
```
