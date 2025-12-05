# Design Patterns

## Overview

This document provides guidance on key design patterns for the erc-mdbx-index project, focusing on Rust-specific idioms and best practices.

## Trait-First Design

### Purpose
- Provide flexible, abstract interfaces
- Enable dependency injection
- Support easy testing and mocking

### Example

```rust
pub trait Database {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn cursor(&self, bucket: &str) -> Result<Box<dyn Cursor>>;
}

pub trait Cursor {
    fn next(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>>;
}

// Concrete implementation
struct MdbxDatabase { /* ... */ }
impl Database for MdbxDatabase { /* ... */ }

// Easy testing
struct MockDatabase { /* ... */ }
impl Database for MockDatabase { /* ... */ }
```

## NewType Pattern

### Purpose
- Add type safety
- Prevent incorrect type usage
- Provide domain-specific semantics

### Example

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

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

// Prevents incorrect usage
let addr = Address::from_slice(&[0; 20])?;
```

## Builder Pattern

### Purpose
- Configure complex objects
- Provide flexible object construction
- Validate configuration before object creation

### Example

```rust
pub struct DatabaseConfig {
    path: PathBuf,
    max_dbs: u32,
    read_only: bool,
}

pub struct DatabaseConfigBuilder {
    path: Option<PathBuf>,
    max_dbs: u32,
    read_only: bool,
}

impl DatabaseConfigBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            max_dbs: 10,
            read_only: true,
        }
    }

    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn max_dbs(mut self, max: u32) -> Self {
        self.max_dbs = max;
        self
    }

    pub fn writable(mut self) -> Self {
        self.read_only = false;
        self
    }

    pub fn build(self) -> Result<DatabaseConfig> {
        Ok(DatabaseConfig {
            path: self.path.ok_or(Error::MissingPath)?,
            max_dbs: self.max_dbs,
            read_only: self.read_only,
        })
    }
}

// Usage
let config = DatabaseConfigBuilder::new()
    .path("/path/to/db".into())
    .max_dbs(20)
    .writable()
    .build()?;
```

## Strategy Pattern

### Purpose
- Define a family of algorithms
- Make algorithms interchangeable
- Avoid multiple conditionals

### Example

```rust
trait DecodeStrategy {
    fn decode(&self, input: &[u8]) -> Result<PlainAccount>;
}

struct RlpDecodeStrategy;
impl DecodeStrategy for RlpDecodeStrategy {
    fn decode(&self, input: &[u8]) -> Result<PlainAccount> {
        // RLP-specific decoding
    }
}

struct JsonDecodeStrategy;
impl DecodeStrategy for JsonDecodeStrategy {
    fn decode(&self, input: &[u8]) -> Result<PlainAccount> {
        // JSON-specific decoding
    }
}

struct AccountDecoder<S: DecodeStrategy> {
    strategy: S,
}

impl<S: DecodeStrategy> AccountDecoder<S> {
    fn decode(&self, input: &[u8]) -> Result<PlainAccount> {
        self.strategy.decode(input)
    }
}
```

## Error Handling Pattern

### Purpose
- Provide comprehensive error information
- Support error conversion
- Maintain error context across layers

### Example

```rust
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DatabaseError),

    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
}

// Automatic error conversion
fn get_account(db: &Database) -> Result<PlainAccount> {
    let raw_bytes = db.get(key)?;  // Converts DatabaseError automatically
    PlainAccount::decode(&raw_bytes)?  // Converts CodecError automatically
}
```

## When to Use Each Pattern

1. **Trait-First**: Dependency injection, testing
2. **NewType**: Type safety, domain semantics
3. **Builder**: Complex configuration
4. **Strategy**: Algorithm selection
5. **Error Handling**: Comprehensive error management

## Best Practices

- Prefer composition over inheritance
- Keep interfaces minimal
- Use traits for abstraction
- Validate configurations early
- Provide meaningful error context

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05