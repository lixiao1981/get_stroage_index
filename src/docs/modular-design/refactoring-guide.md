# Refactoring Guide

## Overview

This document provides strategies for systematically improving code architecture and addressing common modular design violations.

## Common Architectural Violations

### 1. Circular Dependencies

**Symptoms**:
- Modules directly import each other
- Tight coupling between layers
- Difficult to understand system structure

**Refactoring Strategy**:

1. **Identify Circular Dependency**
```rust
// L Circular Dependency
// reader/state.rs
use crate::codec::AccountDecoder;

// codec/decoder.rs
use crate::reader::StateReader;  // Circular reference!
```

2. **Apply Trait Abstraction**
```rust
//  Trait Abstraction
// reader/state.rs
pub trait AccountDecodable {
    fn decode_account(&self, data: &[u8]) -> Result<PlainAccount>;
}

struct StateReader<D: AccountDecodable> {
    decoder: D,
}
```

### 2. Glob Imports

**Symptoms**:
- `use crate::*` or `use module::*`
- Unclear import sources
- Potential naming conflicts

**Refactoring Strategy**:

1. **Replace Glob Imports**
```rust
// L Unclear imports
use crate::*;

//  Explicit Imports
use crate::model::PlainAccount;
use crate::db::Database;
use crate::error::Result;
```

### 3. Layer Boundary Violations

**Symptoms**:
- Lower layers depend on higher layers
- Direct database access in service layer
- Inconsistent error handling

**Refactoring Strategy**:

1. **Define Layer Boundaries**
```rust
// L Layer Violation
// db/environment.rs
use crate::reader::StateReader;  // Lower layer depends on higher layer

//  Correct Dependency
// db/environment.rs
pub trait StateReadable {
    fn read_state(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}
```

### 4. Error Handling Across Layers

**Symptoms**:
- Raw error types crossing layer boundaries
- No error context
- Inconsistent error translation

**Refactoring Strategy**:

1. **Centralize Error Handling**
```rust
//  Centralized Error Handling
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DatabaseError),

    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),

    #[error("State read error: {0}")]
    StateRead(String),
}

impl From<CodecError> for Error {
    fn from(err: CodecError) -> Self {
        Error::Codec(err)
    }
}
```

### 5. Tight Coupling

**Symptoms**:
- Complex constructor methods
- Many dependencies in a single module
- Hard to test or modify

**Refactoring Strategy**:

1. **Use Dependency Injection**
```rust
// L Tight Coupling
struct StateReader {
    db: MdbxDatabase,
    codec: RlpCodec,
    logger: FileLogger,
}

//  Dependency Injection
struct StateReader<D: Database, C: Codec, L: Logger> {
    db: D,
    codec: C,
    logger: L,
}
```

## Refactoring Process

1. **Analyze Current Architecture**
   - Run `cargo clippy`
   - Review dependency graph
   - Identify violation patterns

2. **Plan Refactoring**
   - Create feature flag for new implementation
   - Define incremental migration steps
   - Write comprehensive tests

3. **Implement Changes**
   - Introduce new traits/interfaces
   - Modify existing code incrementally
   - Use feature flags to toggle between old/new implementations

4. **Validate Refactoring**
   - Run full test suite
   - Verify performance impact
   - Conduct code review

## Best Practices

- Prefer composition over inheritance
- Use traits for abstraction
- Minimize public API surface
- Write tests before refactoring
- Document architectural decisions

## Tools

- `cargo clippy`: Static analysis
- `cargo machete`: Dependency analysis
- `cargo diet`: Dependency optimization

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05