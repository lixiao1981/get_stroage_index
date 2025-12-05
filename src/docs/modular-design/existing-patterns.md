# Existing Design Patterns in erc-mdbx-index

## Overview

This document captures the current design patterns used in the erc-mdbx-index project, serving as a reference for our modular design approach.

## Identified Patterns

### 1. Trait-Based Abstraction

Traits are used extensively to define abstract interfaces, enabling dependency injection and decoupling.

**Examples**:

#### Database Abstraction
```rust
pub trait ReadTransaction {
    fn get(&self, bucket: &str, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn cursor(&self, bucket: &str) -> Result<impl Cursor>;
}

pub trait Cursor {
    fn seek(&mut self, key: &[u8]) -> Result<Option<(&[u8], &[u8])>>;
    fn next(&mut self) -> Result<Option<(&[u8], &[u8])>>;
}
```

#### Codec Abstractions
```rust
pub trait AccountDecoder {
    fn decode(&self, data: &[u8]) -> Result<PlainAccount>;
}

pub trait AccountEncoder {
    fn encode(&self) -> Vec<u8>;
}
```

### 2. NewType Pattern

Provides type safety for domain-specific concepts.

```rust
pub struct Address(Vec<u8>);

impl Address {
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 20 {
            return Err(Error::InvalidAddressLength);
        }
        Ok(Self(slice.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
```

### 3. Builder Pattern

Used for configuring complex objects with multiple optional parameters.

```rust
pub struct ErigonDbBuilder {
    path: Option<PathBuf>,
    max_dbs: u32,
    max_readers: u32,
    read_only: bool,
}

impl ErigonDbBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            max_dbs: 256,
            max_readers: 126,
            read_only: true,
        }
    }

    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn max_dbs(mut self, n: u32) -> Self {
        self.max_dbs = n;
        self
    }

    pub fn build(self) -> Result<ErigonDb> {
        // Construction logic
    }
}
```

### 4. Error Handling Pattern

Centralized error management with context preservation.

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DatabaseError),

    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
}
```

### 5. Dependency Injection Pattern

Modules designed to accept generic type parameters for flexibility.

```rust
pub struct StateReader<D: Database = ErigonDb> {
    db: D,
}

impl<D: Database> StateReader<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }
}
```

## Pattern Analysis

- **Most Used**: Trait-Based Abstraction
- **Primary Goal**: Decoupling and Testability
- **Performance Approach**: Mostly static dispatch
- **Flexibility**: High, with minimal runtime overhead

## Recommendations

1. Continue using trait-based abstractions
2. Expand NewType usage for more type safety
3. Use Builder pattern for complex configurations
4. Maintain centralized error handling

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05