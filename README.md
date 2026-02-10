# Solana Token Fundraiser

Three implementations of the same SPL Token fundraiser program on Solana, each at a different abstraction level -- from high-level framework to bare-metal unsafe Rust.

## What It Does

A maker creates a fundraiser for an SPL token with a target amount and deadline. Contributors donate tokens into a PDA-owned vault. If the target is met, the maker claims all tokens. If the deadline passes without reaching the target, contributors can get refunds.

**Instructions:** Initialize | Contribute | Checker (claim) | Refund

## Implementations

|                        | [Anchor](./fundraiser-anchor)                  | [Native](./fundraiser-native)          | [Optimised](./fundraiser-optimised)      |
| ---------------------- | ---------------------------------------------- | -------------------------------------- | ---------------------------------------- |
| **Framework**          | Anchor v0.30                                   | Solana SDK v2.0.15                     | Pinocchio v0.6                           |
| **Language**           | Rust                                           | Rust                                   | Rust (unsafe)                            |
| **Serialization**      | Anchor (Borsh under the hood)                  | Borsh                                  | Zero-copy (raw pointers)                 |
| **Account Validation** | Declarative constraints (`#[account]`)         | Manual checks in handler code          | Manual + `assert!` macros                |
| **CPI**                | `CpiContext::new` / `new_with_signer`          | `invoke` / `invoke_signed`             | `Transfer::invoke()` / `invoke_signed()` |
| **Account Closing**    | `close = maker` constraint                     | Manual lamport transfer + data zeroing | Not implemented (out of scope)           |
| **Testing**            | TypeScript + Mocha                             | Mollusk SVM                            | Mollusk SVM                              |
| **Duration Model**     | Relative (`time_started` + `duration` in days) | Absolute (`time_ending` timestamp)     | Absolute (`time_ending` timestamp)       |
| **Error Handling**     | `#[error_code]` enum                           | `thiserror` + `ProgramError::Custom`   | `ProgramError` + `assert!` panics        |

## Comparison

### Anchor

- Highest developer ergonomics -- declarative account constraints, auto PDA derivation, built-in ATA init
- Most safety guardrails (ownership checks, signer checks, space calculation done by the framework)
- Largest binary size and highest CU consumption
- Best for: prototyping, auditable code, teams new to Solana

### Native

- Full manual control over every account check and CPI
- Borsh for serialization -- explicit but adds some overhead
- Middle ground between safety and performance
- Best for: understanding Solana internals, moderate optimization needs

### Optimised (Pinocchio)

- Zero-copy state via raw pointer arithmetic -- no serialization overhead
- `borrow_mut_data_unchecked()` skips runtime borrow checks
- Compile-time base58 decoding (`five8_const`)
- Minimal CU footprint
- Best for: production programs where compute budget matters

## Build & Test

```bash
# Anchor
cd fundraiser-anchor && anchor build && anchor test

# Native
cd fundraiser-native && cargo build-sbf && cargo test

# Optimised
cd fundraiser-optimised && cargo build-sbf && cargo test
```
