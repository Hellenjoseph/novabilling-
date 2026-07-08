# NovaBilling 🌊

[![NovaBilling CI](https://github.com/Hellenjoseph/novabilling-/actions/workflows/ci.yml/badge.svg)](https://github.com/Hellenjoseph/novabilling-/actions/workflows/ci.yml)
[![Soroban](https://img.shields.io/badge/Soroban-v26.1.0-blue.svg)](https://soroban.stellar.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-orange.svg)](LICENSE)

**NovaBilling** is an on-chain, non-custodial recurring subscription and billing protocol built natively on Stellar Soroban. It solves a crucial web3 commerce challenge by enabling time-locked, recurring payment pulling from subscriber wallets, allowing decentralized recurring revenue (SaaS, memberships, newsletters) to operate seamlessly on-chain.

---

## 🚀 Key Features

*   **Secure Pull Payment Protocol**: Subscribers approve a pre-defined billing rate and frequency, enabling merchants to execute recurring pulls securely without keeping funds escrowed in the contract.
*   **Time-Locked Enforcements**: The smart contract prevents early charges, strictly enforcing billing cycle limits (e.g. 30 days) on-chain.
*   **On-Chain Delinquency Management**: Attempts to charge a wallet with insufficient balance or revoked approvals automatically transitions the subscription to `Delinquent` on the ledger without breaking execution.
*   **Subscriber Billing Controls**: Subscribers retain full custody of their keys, with controls to pause, resume, or permanently cancel billing authorizations at any time.

---

## 🛠️ Folder Architecture

```
novabilling/
├── .github/workflows/
│   └── ci.yml                          # Continuous Integration pipeline
├── Contracts/
│   └── novabilling-contract/
│       ├── src/
│       │   ├── error.rs                # Execution error codes
│       │   ├── helper.rs               # Time and cycle validations
│       │   ├── lib.rs                  # Core billing methods
│       │   ├── storage.rs              # State persistent interfaces
│       │   ├── test.rs                 # Unit test suite
│       │   └── types.rs                # State types and storage structures
│       └── Cargo.toml                  # Contract dependencies
├── Cargo.toml                          # Workspace root
└── README.md                           # Documentation
```

---

## ⚙️ Setup & Installation

### Prerequisites

Ensure you have Rust and the WebAssembly target installed:

1.  **Install Rust**:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2.  **Add target `wasm32v1-none`** (recommended for Soroban contracts compiled on Rust 1.84+):
    ```bash
    rustup target add wasm32v1-none
    ```
3.  **Install Soroban CLI**:
    ```bash
    cargo install --locked soroban-cli
    ```

---

## 💻 Developer Commands

Run these commands inside `Contracts/novabilling-contract/`:

### 🧪 Run Unit Tests

Execute the full suite of unit tests checking subscription registration, immediate charges, double-charge protection, delinquency transitions, and pause/cancel logic:

```bash
cargo test
```

### 📦 Build WebAssembly Contract

Compile the optimized, bare-metal WebAssembly contract ready for deployment:

```bash
cargo build --target wasm32v1-none --release
```

The compiled contract will be output at:
`target/wasm32v1-none/release/novabilling_contract.wasm`

---

## 📜 Smart Contract API

### Configuration & Management
*   `set_admin(admin: Address)`
*   `create_subscription(subscriber: Address, merchant: Address, token: Address, rate: i128, period: u64) -> u32`
*   `charge_subscription(subscription_id: u32)`
*   `pause_subscription(subscription_id: u32)`
*   `resume_subscription(subscription_id: u32)`
*   `cancel_subscription(caller: Address, subscription_id: u32)`

### Read Operations
*   `get_subscription(subscription_id: u32) -> Option<Subscription>`
*   `get_subscriber_subscriptions(subscriber: Address) -> Vec<u32>`
*   `get_merchant_subscriptions(merchant: Address) -> Vec<u32>`

---

## 🛡️ License

This project is licensed under the Apache License 2.0. See the `LICENSE` file for details.
