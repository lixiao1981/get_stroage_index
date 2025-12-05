# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Erigon PlainState Indexer** - A high-performance Rust tool for reading the latest Ethereum world state (PlainState) from Erigon's MDBX database. The indexer must handle Erigon's specific serialization: DupSort storage layout for contract storage, and RLP-encoded account state.

**Key Constraints:**
- Rust Nightly required (for performance-critical features)
- Disk-direct I/O optimized for NVMe sequential reads
- Must decode Erigon's specific RLP account encoding and DupSort storage structure
- High throughput on large state reads

## Development Commands

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_name -- --exact

# Check code (faster than full build)
cargo check

# Format code
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --open

# Run with backtrace for debugging
RUST_BACKTRACE=1 cargo run --release
```

## Architecture Overview

### Key Components to Implement

1. **MDBX Reader** - Direct interface to the MDBX database using libmdbx-rs
   - Open/close transactions on the PlainState bucket
   - Handle DupSort key-value iteration (critical for storage reads)
   - Efficient sequential scanning for large state snapshots

2. **RLP Decoder** - Decode Erigon's account encoding using alloy-rlp
   - Account state: nonce, balance, codehash (and optional fields for contract storage roots)
   - Must handle both legacy and new account formats Erigon may use

3. **Storage Iterator** - Handle the DupSort layout for contract storage
   - DupSort creates a nested key-value structure where each account key can have multiple values
   - Must iterate storage efficiently without loading all values into memory

4. **Index Builder** - Output the indexed state
   - Accumulate state changes/snapshots
   - Export in desired format (likely another MDBX, flat files, or streaming)

### Data Flow

```
MDBX Database (PlainState bucket)
    ↓ (Direct I/O, sequential scan)
RLP Decoder (alloy-rlp)
    ↓ (Account + Storage)
Index/State Accumulator
    ↓
Output (Format TBD)
```

### Performance Considerations

- Use `libmdbx-rs` cursor operations for sequential I/O efficiency
- Leverage DupSort cursors for storage iteration to avoid duplicate key parsing
- Consider memory-mapping strategies for large state reads
- Benchmark against sequential I/O patterns to validate NVMe optimization

## Dependencies (Current Stack)

- **libmdbx-rs** - Direct MDBX database access
- **alloy-rlp** - RLP encoding/decoding for Ethereum types
- **Rust Nightly** - For potential SIMD or other performance features

## Important Notes on Erigon Specifics

- **PlainState Bucket**: Contains account state and storage. Keys are account addresses (20 bytes).
- **DupSort Layout**: Storage keys are encoded as `account_address + storage_key`. This creates duplicates in the key space that must be iterated with DupSort-aware cursors.
- **Account RLP Format**: Erigon encodes accounts as RLP tuples. Verify against the current Erigon implementation for field order and optional fields.
- **Storage Iteration**: Use DupSort cursors (`DUPSORT` flag in libmdbx) to iterate storage efficiently without loading all keys into memory.

## Testing Strategy

- Unit tests for RLP decoding (various account formats)
- Integration tests against a small Erigon MDBX snapshot
- Benchmark tests for throughput on large state scans
- Verify correct handling of DupSort storage layout

## Code Style & Standards

- Follow Rust conventions (snake_case for functions/variables, PascalCase for types)
- Use `cargo fmt` before committing
- Run `cargo clippy` to catch common mistakes
- Document public APIs, especially around MDBX access and RLP decoding
