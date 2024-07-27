# Solana-Rust-Sandpit

<img src="solana.png" alt="Solana" style="width:200px;"/>

Version: 0.1.0

Release: 19/07/2024

[![Rust](https://github.com/janrockdev/solana-sandpit/actions/workflows/rust.yml/badge.svg)](https://github.com/janrockdev/solana-sandpit/actions/workflows/rust.yml)

### Description
Sandpit to integrates all common Solana features under one gRPC service with testing client.

### Features
- [x] wallet balance request
- [x] client network argument
- [x] create new wallet
- [x] request faucet balance for wallet
- [x] send coin between wallets
- [x] basic smart contract

### Compile
```shell
cargo build
```

### Run
```shell
#server
cargo run --bin server

#client
#wallet balance request
cargo run --bin client get-balance <network> <wallet_address>
#create wallet
cargo run --bin client create-wallet
#airdrop wallet - example one 1SOL = 1_000_000_000
cargo run --bin client request-airdrop <network> <wallet_address> 1000000000
#send SOL from one wallet to another
cargo run --bin client 
<<<<<<< HEAD
#submit transaction to contract
cargo run --bin client greet devnet <secret-key> cau
```

### Resources
#### Solana Explorer
https://explorer.solana.com/?cluster=devnet

#### Phantom Wallet
https://phantom.app/