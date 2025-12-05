# Architectural Refactoring Exercise

## Overview

This document presents a practical refactoring exercise designed to identify and resolve common architectural violations in the erc-mdbx-index project.

## Exercise Objective

Refactor a deliberately problematic codebase to adhere to our modular design principles. You'll address three key architectural violations.

## Scenario: Legacy State Reader Implementation

### Original (Problematic) Code

```rust
// legacy_state_reader.rs
use std::collections::HashMap;
use libmdbx::{Transaction, Environment};
use serde_json;

pub struct LegacyStateReader {
    env: Environment,
    cache: HashMap<Vec<u8>, Vec<u8>>,
    json_transformer: JsonTransformer,
}

impl LegacyStateReader {
    pub fn new(path: &str) -> Result<Self, String> {
        let env = Environment::new()
            .open(path)
            .map_err(|e| format!("Failed to open: {}", e))?;

        Ok(Self {
            env,
            cache: HashMap::new(),
            json_transformer: JsonTransformer::new(),
        })
    }

    // ❌ Violation 1: Circular Dependency & Layer Mixing
    pub fn get_account_raw(&self, address: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let txn = self.env.begin_ro_txn()
            .map_err(|e| format!("Transaction error: {}", e))?;

        // Direct database access in reader layer
        let result = txn.get(Some("plain_state"), address)
            .map_err(|e| format!("Read error: {}", e))?;

        // Caching logic mixed into retrieval
        if let Some(data) = result {
            self.cache.insert(address.to_vec(), data.clone());
        }

        // Transforming to JSON within state reader (multiple responsibilities)
        if let Some(data) = result {
            let json = self.json_transformer.transform(&data)
                .map_err(|e| format!("JSON transform error: {}", e))?;
            println!("Account JSON: {}", json);
        }

        Ok(result)
    }

    // ❌ Violation 2: Glob Imports & Unclear Dependencies
    use crate::*;  // Problematic glob import
    pub fn list_accounts(&self) -> Result<Vec<Vec<u8>>, String> {
        let mut accounts = Vec::new();
        // Unclear dependencies, potential namespace conflicts
        for (addr, _) in self.cache.iter() {
            accounts.push(addr.clone());
        }
        Ok(accounts)
    }
}

// ❌ Violation 3: Inconsistent Error Handling
struct JsonTransformer;

impl JsonTransformer {
    fn new() -> Self { Self {} }

    fn transform(&self, data: &[u8]) -> Result<String, String> {
        // Inconsistent, unstructured error handling
        serde_json::from_slice(data)
            .map_err(|e| format!("JSON error: {}", e))
    }
}
```

## Refactoring Objectives

1. **Violation 1**: Break Circular Dependencies & Separate Concerns
   - Remove direct database access from reader layer
   - Separate caching, transformation, and reading responsibilities
   - Use trait-based abstractions

2. **Violation 2**: Replace Glob Imports
   - Remove glob imports
   - Use explicit, targeted imports
   - Clarify module dependencies

3. **Violation 3**: Implement Consistent Error Handling
   - Create a centralized error enum
   - Use `thiserror` for comprehensive error definitions
   - Preserve error sources and context

## Refactored Solution

```rust
// Centralized Error Handling
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateReaderError {
    #[error("Database transaction error: {0}")]
    TransactionError(#[source] libmdbx::Error),

    #[error("Account retrieval failed: {0}")]
    RetrievalError(String),

    #[error("JSON transformation error: {0}")]
    TransformError(#[from] serde_json::Error),
}

// Trait-Based Abstractions
pub trait Database {
    fn get(&self, bucket: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StateReaderError>;
}

pub trait Transformer<T> {
    fn transform(&self, data: &T) -> Result<String, StateReaderError>;
}

pub trait Cache {
    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>);
    fn get(&self, key: &[u8]) -> Option<&Vec<u8>>;
}

// Explicit Imports
use std::collections::HashMap;
use serde_json;

// Separated Responsibilities
pub struct StateCache(HashMap<Vec<u8>, Vec<u8>>);

impl Cache for StateCache {
    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.0.insert(key, value);
    }

    fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.0.get(key)
    }
}

pub struct JsonAccountTransformer;

impl Transformer<[u8]> for JsonAccountTransformer {
    fn transform(&self, data: &[u8]) -> Result<String, StateReaderError> {
        let json = serde_json::from_slice(data)?;
        Ok(json)
    }
}

pub struct StateReader<D: Database, C: Cache, T: Transformer<[u8]>> {
    db: D,
    cache: C,
    transformer: T,
}

impl<D: Database, C: Cache, T: Transformer<[u8]>> StateReader<D, C, T> {
    pub fn new(db: D, cache: C, transformer: T) -> Self {
        Self { db, cache, transformer }
    }

    pub fn get_account(&mut self, address: &[u8]) -> Result<Option<String>, StateReaderError> {
        // Separation of concerns
        let raw_data = self.db.get("plain_state", address)?;

        if let Some(data) = raw_data {
            self.cache.insert(address.to_vec(), data.clone());
            let json = self.transformer.transform(&data)?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    pub fn list_accounts(&self) -> Vec<Vec<u8>> {
        // Explicit, clear implementation
        vec![]  // Placeholder for actual implementation
    }
}
```

## Refactoring Checklist

### Layer Separation
- [ ] Removed database access from reader layer
- [ ] Created trait-based abstractions
- [ ] Separated caching, transformation, and reading

### Error Handling
- [ ] Centralized error enum
- [ ] Used `thiserror` for comprehensive errors
- [ ] Preserved error sources
- [ ] Provided meaningful error messages

### Imports and Dependencies
- [ ] Removed glob imports
- [ ] Used explicit, targeted imports
- [ ] Clarified module dependencies

## Performance and Testing Considerations

- Zero-cost abstractions
- Easy to mock for testing
- Dependency injection supported
- Compile-time type safety

## Learning Outcomes

1. Importance of layer separation
2. Benefits of trait-based design
3. Consistent error handling strategies
4. Avoiding tight coupling

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05