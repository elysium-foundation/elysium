# ![Elysium](https://cdn.elysiumchain.tech/elysium-chain-meta-image.png)

# Information
**An Ethereum compatible Chain built with the [Polkadot-SDK](https://github.com/paritytech/polkadot-sdk).**

👉 _Discover the Elysium project at [elysium](https://docs.elysiumchain.tech/docs/network-endpoints)._<br>
👉 _Learn to [use the Elysium network](https://docs.elysiumchain.tech/docs/intro) with our technical docs._<br>

## Run Elysium Local Development Node with Docker

Docker images are published for every tagged release.

```bash
# Join the public testnet
docker run vaival/elysium-testnet:latest --dev --rpc-external --rpc-cors all
```

You can find more detailed instructions to [Run a Node on Elysium
](https://docs.elysiumchain.tech/docs/node-operator/dev-node)


### Sealing Options

The above command will start the node in instant seal mode. It creates a block when a transaction arrives, similar to Ganache's auto-mine. You can also choose to author blocks at a regular interval, or control authoring manually through the RPC.

```bash
# Author a block every 6 seconds.
docker run vaival/elysium-testnet:latest --dev --rpc-external --rpc-cors all --sealing 6000

# Manually control the block authorship and finality
docker run vaival/elysium-testnet:latest --dev --rpc-external --rpc-cors all --sealing manual
```

### Prefunded Development Addresses

Running Elysium in development mode will pre-fund several well-known addresses that (mostly) contain the letters "th" in their names to remind you that they are for ethereum-compatible usage. These addresses are derived from
Substrate's canonical mnemonic: `bottom drive obey lake curtain smoke basket hold race lonely fit walk`

```
# Alice:
- Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
- PrivKey: 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
```

Mainnet tokens are only available for those people who are bridging their assets from Polygon to [Elysium using Bridge](https://bridge.elysiumchain.tech).

| Network  | URL	                                                                                                            | 
|----------|-----------------------------------------------------------------------------------------------------------------|
| Atlantis | The [Faucet Atlantis](https://faucet.atlantischain.network/) website. The faucet dispenses 1 ELY every 24 hours | 
| Elysium  | The [Faucet Elysium](https://faucet.elysiumchain.tech/) website. The faucet dispenses 3 ELY for one time.       |

## Build the Elysium Node

To build Elysium, a proper Substrate development environment is required. If you're new to working with Substrate-based blockchains, consider starting with the [Getting Started with Elysium Development Node](https://docs.elysiumchain.tech/docs/category/node-operator) documentation.

If you need a refresher setting up your Substrate environment, see [Substrate's Getting Started Guide](https://substrate.dev/docs/en/knowledgebase/getting-started/).

```bash
# Fetch the code
git clone https://github.com/elysium-foundation/elysium
cd elysium

# Build the node (The first build will be long (~30min))
cargo build --release --locked

# To execute the chain, run:
./target/release/elysium --dev

```

## Install required packages and Rust
To run the Elysium you need to have ubuntu 20.04 installed. Follow these steps to set up rust for building the repo.

```bash
sudo apt update
sudo apt install --assume-yes build-essential git clang curl libssl-dev protobuf-compiler llvm libudev-dev make
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

#Follow the prompts displayed to proceed with a default installation.
source $HOME/.cargo/env
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

#Verify the configuration of your development environment by running the following command:
rustup show
rustup +nightly show
```

## Run Tests

Elysium incorporates Rust unit tests and TypeScript integration tests, which are executed in CI and can also be run locally.

```bash
# Run the Rust unit tests
cargo test

# Run the tests one by one
cargo test -- --test-threads=1

### Running testcases for EVM support are
cd ts-test 
npm run build
npm run test
```
## Chain ID

Elysium Chain Mainnet chain ID is: 1339 & Testnet chain ID is: 1338 for more information follow [Elysium Network](/docs/intro#elysium-networks)

### Public Endpoints

Elysium has two endpoints available for users to connect to: one for RPC and one for Websocket connection.

The endpoints in this section are for development purposes only and are not meant to be used in production applications.

If you are looking for an API provider suitable for production use, you can check out the Node Operator section of
this guide to setup your own node.

#### Atlantis (Testnet)

| Variable  | Value                             |
|-----------|-----------------------------------|
| Chain ID	 | 1338                              |
| RPC URL   | https://rpc.atlantischain.network | 
| WSS URL	  | wss://ws.atlantischain.network    | 

#### Elysium Mainnet

| Variable  | Value                         |
|-----------|-------------------------------|
| Chain ID	 | 1339                          |
| RPC URL   | https://rpc.elysiumchain.tech | 
| WSS URL	  | wss://ws.elysiumchain.tech    | 


## Runtime Architecture

The Elysium Runtime, built using FRAME, comprises pallets from Polkadot-SDK, Frontier, and the `pallets/` directory.

From Polkadot-SDK:

- **Utility**: Allows users to use derivative accounts, and batch calls
- **Balances**: Tracks ELY token balances
- **Sudo**: Allows a privileged account to make arbitrary runtime changes. This will be removed by Governance.
- **Timestamp**: On-Chain notion of time
- **Transaction Payment**: Transaction payment (fee) management
- **Randomness Collective Flip**: A (mock) onchain randomness beacon, which will be replaced by chain randomness by mainnet.

From Frontier:

- **EVM Chain Id**: A place to store the chain id
- **EVM**: Encapsulates execution logic for an Ethereum Virtual Machine
- **Ethereum**: Ethereum-style data encoding and access for the EVM.

The following pallets are stored in `pallets/`. They are designed for Elysium's specific requirements:
When modifying the git repository for these dependencies, a tool called [diener](https://github.com/bkchr/diener) can be used to replace the git URL and branch for each reference in all `Cargo.toml` files with a single command. This alleviates a lot of the repetitive modifications necessary when changing dependency versions.

## Changelog

You can find the changelog [here](./CHANGELOG.md).
