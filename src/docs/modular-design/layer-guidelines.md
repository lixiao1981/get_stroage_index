# Layer Definition Guidelines for erc-mdbx-index

## Overview

This document provides comprehensive guidelines for defining and implementing architectural layers in the erc-mdbx-index project.

## Layer Architecture Principles

### Core Principles

1. **Single Responsibility**
   - Each layer has a clear, focused purpose
   - Minimize cross-layer dependencies
   - Maximize layer independence

2. **Explicit Boundaries**
   - Define clear interfaces between layers
   - Use traits to abstract layer interactions
   - Minimize public API surface

3. **Dependency Direction**
   - Dependencies flow downward only
   - Lower layers are unaware of higher layers
   - Avoid circular dependencies

## Detailed Layer Specifications

### 1. Database Abstraction Layer (DAL)

#### Responsibilities
- Encapsulate low-level database interactions
- Manage database connections and transactions
- Provide safe, abstracted database access

#### Key Traits
```rust
pub trait DatabaseEnvironment {
    fn open(&self) -> Result<()>;
    fn close(&self) -> Result<()>;
}

pub trait ReadTransaction {
    fn get(&self, bucket: &str, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn cursor(&self, bucket: &str) -> Result<Box<dyn Cursor>>;
}

pub trait WriteTransaction: ReadTransaction {
    fn put(&mut self, bucket: &str, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&mut self, bucket: &str, key: &[u8]) -> Result<()>;
}
```

#### Implementation Guidelines
- Use `libmdbx` for MDBX database interactions
- Implement transaction lifecycle management
- Ensure thread-safe database access
- Minimize runtime overhead

### 2. Model Layer

#### Responsibilities
- Define domain-specific data structures
- Implement business logic constraints
- Provide type-safe representations

#### Key Patterns
- NewType for domain-specific types
- Immutable data structures
- Validation at construction

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Address(Vec<u8>);

impl Address {
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 20 {
            return Err(Error::InvalidAddressLength);
        }
        Ok(Self(slice.to_vec()))
    }
}

#[derive(Debug, Clone)]
pub struct PlainAccount {
    address: Address,
    nonce: u64,
    balance: U256,
    storage_root: Option<H256>,
    code_hash: Option<H256>,
}
```

#### Implementation Guidelines
- Prefer composition over inheritance
- Implement `From`/`TryFrom` traits for conversions
- Use `thiserror` for domain-specific errors

### 3. Codec Layer

#### Responsibilities
- Handle data encoding/decoding
- Implement RLP parsing
- Convert between binary and structured representations

#### Key Traits
```rust
pub trait Encoder {
    fn encode(&self) -> Vec<u8>;
}

pub trait Decoder: Sized {
    fn decode(data: &[u8]) -> Result<Self>;
}

pub trait RlpCodec: Encoder + Decoder {}
```

#### Implementation Guidelines
- Use `alloy-rlp` for RLP encoding
- Support different Erigon encoding variants
- Provide comprehensive error handling
- Minimize allocation and copying

### 4. Reader Layer

#### Responsibilities
- Implement business logic for data retrieval
- Coordinate between database and model layers
- Provide high-level, domain-specific reading operations

#### Example Implementation
```rust
pub struct StateReader<D: Database, C: Codec> {
    db: D,
    codec: C,
}

impl<D: Database, C: Codec> StateReader<D, C> {
    pub fn get_account(&self, address: Address) -> Result<Option<PlainAccount>> {
        let raw_bytes = self.db.get(address.as_bytes())?;
        raw_bytes.map(|bytes| PlainAccount::decode(&bytes)).transpose()
    }
}
```

#### Implementation Guidelines
- Use dependency injection
- Support generic database and codec backends
- Provide high-level, meaningful operations
- Minimize complex logic in this layer

### 5. Utility Layer

#### Responsibilities
- Provide cross-cutting helper functions
- Implement configuration management
- Handle signal processing
- Offer generic, reusable utilities

#### Example Utilities
```rust
pub struct Config {
    db_path: PathBuf,
    max_readers: u32,
    read_only: bool,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

pub struct SignalHandler {
    // Graceful shutdown mechanisms
}
```

## Cross-Cutting Concerns

### Dependency Injection
- Use generic type parameters
- Prefer trait bounds over concrete types
- Support mocking for testing

### Error Handling
- Define centralized error types
- Use `#[from]` for error conversion
- Provide context-rich error messages

### Performance Considerations
- Prefer static dispatch
- Use `#[inline]` judiciously
- Minimize heap allocations

## Testing Strategies

- Unit test each layer in isolation
- Use mock implementations
- Test error cases and edge conditions
- Aim for >90% test coverage

## Version Control

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05

## Contribution Guidelines

1. Respect layer boundaries
2. Minimize dependencies
3. Write comprehensive tests
4. Document architectural decisions
5. Follow Rust idioms and best practices