# Architectural Rules and Guidelines

## Overview

This document defines the core architectural rules for the erc-mdbx-index project, establishing clear guidelines for code organization, dependency management, and design principles.

## Layer Dependency Rules

### 1. Strict Layer Hierarchy

```
Layers (Bottom to Top):
- Utility
- Database Abstraction Layer (DAL)
- Model Layer
- Codec Layer
- Reader Layer
```

**Rules**:
- Lower layers CANNOT depend on higher layers
- Each layer has a minimal, well-defined interface
- Dependencies flow downward only

### 2. Dependency Direction

```rust
//  Correct Dependencies
// reader/state.rs
use crate::db::Database;
use crate::model::Account;
use crate::codec::Decoder;

// L Incorrect Dependencies
// db/transaction.rs
use crate::reader::StateReader;  // Layer violation!
```

## Module Design Principles

### 1. Interface Segregation

- Define narrow, focused traits
- Prefer multiple small interfaces over large, complex ones
- Use trait objects for runtime polymorphism

```rust
// Good: Focused Traits
trait Readable {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}

trait Writable {
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
}

trait Database: Readable + Writable {}
```

### 2. Minimal Public API

- Restrict public API surface
- Use `pub(crate)` and `pub(super)` visibility modifiers
- Re-export only essential types in `lib.rs`

```rust
// db/mod.rs
mod transaction;  // Private implementation
pub struct Database;  // Public interface
pub(crate) fn internal_helper();  // Crate-level visibility
```

## Error Handling Rules

### 1. Error Type Conversion

- Define a centralized error enum
- Use `thiserror` for comprehensive error definitions
- Convert errors at module boundaries

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DatabaseError),

    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
}
```

### 2. Error Context

- Provide meaningful error messages
- Include relevant context
- Use `#[source]` and `#[from]` attributes

## Naming Conventions

- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Prefix traits with verbs: `Readable`, `Writable`
- Suffix error types with `Error`

## Import Rules

### 1. Explicit Imports

```rust
//  Explicit, Clear Imports
use crate::model::Account;
use crate::db::Database;
use crate::error::Result;

// L Avoid Glob Imports
use crate::*;
use crate::model::*;
```

### 2. Standard Import Order

1. Standard library imports
2. External crate imports
3. Local module imports
4. Local crate imports

```rust
// Recommended Import Order
use std::path::PathBuf;
use std::collections::HashMap;

use alloy_primitives::Address;
use thiserror::Error;

use crate::model::Account;
use crate::error::Result;
```

## Testing Guidelines

- Each module must have unit tests
- Use mock implementations for dependency injection
- Test error cases and edge conditions
- Aim for >90% test coverage

## Performance Considerations

- Prefer zero-cost abstractions
- Use traits with static dispatch
- Minimize heap allocations
- Use `#[inline]` judiciously

## Version Control

- Commit small, focused changes
- Write descriptive commit messages
- Use feature branches for significant changes
- Squash commits before merging

## Code Review Checklist

- [ ] Follows layer dependency rules
- [ ] Minimal public API surface
- [ ] Proper error handling
- [ ] Comprehensive unit tests
- [ ] No performance regressions
- [ ] Clear, descriptive naming
- [ ] Follows Rust idioms

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05