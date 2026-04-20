# etop

A terminal-based dashboard for observing Ethereum execution nodes.

## Overview

etop provides high-performance, real-time visualization of local or remote EVM execution clients. Built in Rust, it uses Ratatui for the interface and Alloy for the RPC backend to ensure low-latency monitoring directly in your terminal.

## Core Features

- Real-time block propagation metrics
- Native mempool tracking
- Gas fee history visualization
- Local peer and node resource observation

## Tech Stack

- **Interface**: Ratatui and Crossterm
- **Backend**: Alloy Framework
- **Asynchronous Runtime**: Tokio
- **Target Nodes**: Any standard Ethereum JSON-RPC endpoint

## Installation

### Prerequisites

- Rust 1.78 or higher
- Cargo

### Build from source

```sh
git clone https://github.com/leo9iota/etop.git && cd etop
cargo build --release
```

## Usage

Start the application by pointing it to a local or remote Ethereum node.

```sh
./target/release/etop --rpc http://localhost:8545
```

## Contributing

PRs are welcome. Please ensure that all code compiles without warnings and is formatted using standard cargo tooling.

## License

[MIT](./LICENSE)
