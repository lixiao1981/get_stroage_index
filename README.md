# get_stroage_index

## Erigon PlainState Indexer

A high-performance Rust tool for reading the latest Ethereum world state (PlainState) from Erigon's MDBX database.

### Key Features

- Direct MDBX database access
- RLP decoding for Ethereum state
- High-throughput state reads
- Modular Rust architecture

### Prerequisites

- Rust Nightly
- libmdbx
- Erigon database

### Quick Start

```bash
# Build
cargo build --release

# Run tests
cargo test
```

### Documentation

Detailed architectural documentation is available in `src/docs/RUST_MODULAR_DESIGN.md`

### License

[To be determined]