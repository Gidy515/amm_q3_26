````md
# AMM Program on Solana

A production-oriented Constant Product Automated Market Maker (AMM) built on Solana using Anchor and tested with LiteSVM.

This project implements the foundational mechanics behind decentralized exchanges (DEXs) such as :contentReference[oaicite:0]{index=0} and :contentReference[oaicite:1]{index=1} using a constant product invariant model (`x * y = k`).

---

# What is an AMM?

An Automated Market Maker (AMM) is a decentralized exchange mechanism that allows users to trade assets directly against liquidity pools instead of relying on traditional order books.

Instead of matching buyers and sellers manually:

- Liquidity providers deposit token pairs into pools
- Traders swap against those pools
- Prices are determined algorithmically

The most common AMM model is the **Constant Product Formula**:

```math
x * y = k
```
````

Where:

- `x` = reserve of token X
- `y` = reserve of token Y
- `k` = invariant constant

As one asset is bought, the other becomes scarcer, causing the price to adjust automatically.

---

# Problem AMMs Solve

Traditional exchanges rely on:

- Centralized matching engines
- Market makers
- Custodial infrastructure
- High barriers to liquidity provisioning

AMMs solve this by enabling:

- Permissionless liquidity provisioning
- Fully on-chain trading
- Decentralized price discovery
- Continuous liquidity availability
- Trust-minimized token swaps

This project demonstrates how those primitives are implemented directly on Solana using Anchor.

---

# Project Architecture

This AMM consists of:

## Core Accounts

### Config PDA

Stores:

- Pool configuration
- Token pair
- Fee configuration
- Authority
- Pool state
- PDA bumps

Derived using:

```rust
[b"config", seed.to_le_bytes()]
```

---

### LP Mint PDA

Represents ownership shares of liquidity providers.

Liquidity providers receive LP tokens proportional to their contribution to the pool.

Derived using:

```rust
[b"lp", config.key().as_ref()]
```

---

### Vault Token Accounts

Vaults hold the pool reserves for:

- Token X
- Token Y

Vaults are Associated Token Accounts owned by the Config PDA.

---

# Features Implemented

## 1. Initialize Pool

Creates and initializes:

- Config PDA
- LP mint PDA
- Vault token accounts
- Pool configuration

### Responsibilities

- Sets token pair
- Stores fee configuration
- Creates LP mint
- Creates vault accounts
- Establishes pool authority

---

## 2. Deposit / Add Liquidity

Allows liquidity providers to deposit token pairs into the pool.

### Logic

For initial liquidity:

```text
User provides:
max_x and max_y
```

For subsequent deposits:

The protocol calculates optimal token ratios using:

```text
ConstantProduct::xy_deposit_amounts_from_l()
```

### Process

- Transfers token X into vault
- Transfers token Y into vault
- Mints LP tokens to provider

### Slippage Protection

```rust
require!(x <= max_x, AmmError::SlippageExceeded);
require!(y <= max_y, AmmError::SlippageExceeded);
```

Protects users from unexpected pricing changes.

---

## 3. Withdraw Liquidity

Allows liquidity providers to burn LP tokens and redeem their proportional share of reserves.

### Process

- Burn LP tokens
- Calculate proportional reserve share
- Transfer token X back to user
- Transfer token Y back to user

---

## 4. Token Swaps

Allows users to swap:

- X → Y
- Y → X

using the constant product invariant.

### Swap Engine

Powered by:

```rust
constant_product_curve
```

### Flow

1. User deposits input token
2. AMM calculates output amount
3. Protocol applies fee
4. Output token transferred to user
5. Pool invariant maintained

---

# Security Features

## PDA-based Authorities

Critical accounts are program-derived:

- Config PDA
- LP Mint PDA
- Vault ATAs

This prevents unauthorized signing.

---

## Slippage Protection

Protects users against:

- MEV
- Front-running
- Unfavorable execution

---

## Ownership Constraints

Anchor constraints enforce:

```rust
has_one = mint_x
has_one = mint_y
```

Ensuring account integrity.

---

## Pool Locking

Pool operations can be disabled using:

```rust
config.locked
```

Useful for:

- Emergency shutdowns
- Governance upgrades
- Maintenance

---

# Tech Stack

## Blockchain

- Solana

## Framework

- Anchor Framework

## Language

- Rust

## Testing

- LiteSVM
- Cargo tests

## SPL Standards

- SPL Token Program
- Associated Token Program

## Math Engine

- constant_product_curve

---

# Project Structure

```text
amm/
├── programs/
│   └── amm/
│       ├── src/
│       │   ├── instructions/
│       │   ├── state/
│       │   ├── errors/
│       │   ├── constants/
│       │   └── lib.rs
│
├── tests/
│   ├── test_initialize.rs
│   └── ix_handlers/
│
├── Anchor.toml
├── Cargo.toml
└── README.md
```

---

# Instructions Overview

| Instruction  | Description                           |
| ------------ | ------------------------------------- |
| `initialize` | Creates pool and vault infrastructure |
| `deposit`    | Adds liquidity and mints LP tokens    |
| `withdraw`   | Removes liquidity and burns LP tokens |
| `swap`       | Swaps between token pairs             |

---

# PDA Derivations

## Config PDA

```rust
[b"config", seed.to_le_bytes()]
```

---

## LP Mint PDA

```rust
[b"lp", config.key().as_ref()]
```

---

## Vault ATAs

Derived using Associated Token Accounts:

```rust
associated_token::authority = config
```

---

# Testing

This project uses **LiteSVM** for fast and deterministic Solana program testing.

Test coverage includes:

- Pool initialization
- Liquidity deposits
- Liquidity withdrawals
- Token swaps
- PDA derivations
- Vault accounting
- LP minting
- Reserve management

---

# Build the Program

## Compile Anchor Program

```bash
anchor build
```

---

# Run Tests

## Rust Tests

```bash
cargo test
```

---

## Anchor Tests

```bash
anchor test
```

---

# Example Workflow

## 1. Initialize Pool

```text
Create pool for:
TOKEN_X / TOKEN_Y
```

---

## 2. Add Liquidity

```text
Deposit:
100 TOKEN_X
100 TOKEN_Y
```

Receive:

```text
LP Tokens
```

---

## 3. Swap

```text
Swap:
TOKEN_X → TOKEN_Y
```

AMM recalculates reserves automatically.

---

## 4. Withdraw Liquidity

```text
Burn LP tokens
```

Receive proportional share of reserves.

---

# Mathematical Model

This AMM uses the constant product invariant:

```math
x * y = k
```

Where:

- Swaps maintain the invariant
- Liquidity changes adjust pool depth
- Prices emerge from reserve ratios

---

# Why LiteSVM?

LiteSVM provides:

- Fast execution
- Deterministic testing
- Lightweight Solana simulation
- Efficient local development

Compared to spinning up full validators, LiteSVM dramatically improves development speed.

---

# Future Improvements

Potential upgrades include:

- Protocol fee collection
- TWAP oracles
- Concentrated liquidity
- Dynamic fees
- Multi-hop routing
- Permissionless pool creation
- Governance integration
- Fee sharing
- Token-2022 support

---

# Learning Goals of This Project

This project demonstrates deep understanding of:

- Solana account model
- PDA derivation
- SPL token mechanics
- CPI calls
- Liquidity mathematics
- Anchor constraints
- DeFi protocol architecture
- AMM mechanics
- On-chain testing infrastructure

---

# References

## Solana

[Solana Documentation](https://solana.com/docs?utm_source=chatgpt.com)

## Anchor

[Anchor Framework](https://www.anchor-lang.com?utm_source=chatgpt.com)

## SPL Tokens

[SPL Token Program](https://spl.solana.com/token?utm_source=chatgpt.com)

---

# Disclaimer

This project is intended for educational and development purposes.

Before deploying to mainnet:

- Perform comprehensive audits
- Add invariant testing
- Conduct fuzz testing
- Validate economic assumptions
- Review overflow and precision handling
