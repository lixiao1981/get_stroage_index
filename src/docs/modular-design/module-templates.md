# Module Template Guide

## Overview

This guide provides standard templates for creating new modules in the erc-mdbx-index project, ensuring consistency and adherence to architectural principles.

## Standard mod.rs Structure

### Template 1: Database Layer Module

**Use Case**: Creating a new database abstraction component

```rust
//! Database bucket operations for [BUCKET_NAME]
//!
//! This module provides low-level access to the [BUCKET_NAME] bucket,
//! encapsulating MDBX-specific operations and providing a safe, typed interface.

// Re-exports for public API
pub use self::cursor::BucketCursor;
pub use self::transaction::BucketTransaction;

// Internal module declarations
mod cursor;
mod transaction;
mod error;

use crate::db::{Database, Transaction};
use std::sync::Arc;

/// Bucket-specific operations for [BUCKET_NAME]
pub struct BucketOps {
    db: Arc<dyn Database>,
}

impl BucketOps {
    /// Creates a new bucket operations handler
    ///
    /// # Arguments
    /// * `db` - Database instance implementing the Database trait
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }
    
    /// Retrieves a value by key from the bucket
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// * `Ok(Some(value))` - Value found
    /// * `Ok(None)` - Key not found
    /// * `Err(e)` - Database error occurred
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let txn = self.db.begin_read()?;
        let result = txn.get(BUCKET_NAME, key)?;
        Ok(result.map(|v| v.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_existing_key() {
        // Test implementation
    }
}
```

**Visibility Patterns**:
- `pub use`: Public re-exports for external users
- `mod cursor`: Internal module, not exposed
- `pub struct BucketOps`: Public struct
- `pub fn new()`: Public constructor
- Private helper functions: No `pub` modifier

**Documentation Headers**:
- Module-level doc comments (`//!`) describing purpose
- Function-level doc comments (`///`) with Arguments, Returns, Examples sections
- Inline comments for complex logic

---

### Template 2: Model Layer Module

**Use Case**: Defining a new domain entity with validation

```rust
//! Plain state domain model
//!
//! This module defines the core domain entities for Ethereum state,
//! providing type-safe representations with validation and invariants.

// Re-exports
pub use self::account::PlainAccount;
pub use self::storage::StorageKey;
pub use self::address::Address;

// Internal modules
mod account;
mod storage;
mod address;
mod validation;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents an Ethereum account in plain state format
///
/// # Invariants
/// - Nonce is always >= 0
/// - Balance is a valid U256 value
/// - Code hash is exactly 32 bytes if present
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlainAccount {
    /// Account nonce (transaction count)
    pub(crate) nonce: u64,
    
    /// Account balance in wei
    pub(crate) balance: U256,
    
    /// Code hash (None for EOA accounts)
    pub(crate) code_hash: Option<[u8; 32]>,
}

impl PlainAccount {
    /// Creates a new plain account with validation
    ///
    /// # Arguments
    /// * `nonce` - Account nonce
    /// * `balance` - Account balance
    /// * `code_hash` - Optional code hash for contract accounts
    ///
    /// # Returns
    /// * `Ok(account)` - Valid account created
    /// * `Err(e)` - Validation failed
    ///
    /// # Examples
    /// ```
    /// use erc_mdbx_index::model::PlainAccount;
    /// 
    /// let account = PlainAccount::new(0, U256::zero(), None)?;
    /// assert!(account.is_eoa());
    /// ```
    pub fn new(nonce: u64, balance: U256, code_hash: Option<[u8; 32]>) -> Result<Self> {
        Self::validate_balance(&balance)?;
        
        Ok(Self {
            nonce,
            balance,
            code_hash,
        })
    }
    
    /// Returns the account nonce
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
    
    /// Returns the account balance
    pub fn balance(&self) -> &U256 {
        &self.balance
    }
    
    /// Checks if this is an externally owned account (EOA)
    pub fn is_eoa(&self) -> bool {
        self.code_hash.is_none()
    }
    
    // Private validation helper
    fn validate_balance(balance: &U256) -> Result<()> {
        // Validation logic
        Ok(())
    }
}

impl fmt::Display for PlainAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlainAccount(nonce: {}, balance: {})", self.nonce, self.balance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_eoa_account() {
        let account = PlainAccount::new(0, U256::zero(), None).unwrap();
        assert!(account.is_eoa());
    }
    
    #[test]
    fn test_create_contract_account() {
        let code_hash = [1u8; 32];
        let account = PlainAccount::new(1, U256::from(1000), Some(code_hash)).unwrap();
        assert!(!account.is_eoa());
    }
}
```

**Visibility Patterns**:
- `pub use`: Selective re-exports for public API
- `pub(crate)`: Internal to crate but accessible across modules
- `pub(super)`: Accessible to parent module only
- Public methods with private fields to enforce invariants

**Documentation Headers**:
- Invariants section in struct docs
- Examples in function docs
- Display implementation for debugging

---

## Visibility Pattern Guide

### Public (`pub`)
**Use When**: 
- External crates need access
- Part of stable public API
- Exported types and functions

**Example**:
```rust
pub struct DatabaseConfig { /* ... */ }
pub fn open_database(config: DatabaseConfig) -> Result<Database> { /* ... */ }
```

---

### Crate-Visible (`pub(crate)`)
**Use When**:
- Shared across modules within crate
- Internal implementation details
- Helper types not meant for external use

**Example**:
```rust
pub(crate) struct InternalCursor { /* ... */ }
pub(crate) fn internal_helper() -> Result<()> { /* ... */ }
```

---

### Parent-Visible (`pub(super)`)
**Use When**:
- Only parent module needs access
- Tightly scoped visibility
- Module-specific helpers

**Example**:
```rust
pub(super) fn validate_key(key: &[u8]) -> bool { /* ... */ }
pub(super) struct ValidationContext { /* ... */ }
```

---

### Module-Private (no modifier)
**Use When**:
- Implementation details
- Private helper functions
- Internal state

**Example**:
```rust
fn calculate_hash(data: &[u8]) -> [u8; 32] { /* ... */ }
struct PrivateCache { /* ... */ }
```

---

## Complete Module Examples

### Example 1: Reader Layer Module with Dependency Injection

**File**: `src/reader/state.rs`

```rust
//! State reading operations
//!
//! Provides high-level access to Ethereum state by coordinating
//! between database, model, and codec layers.

pub use self::cache::StateCache;
pub use self::config::StateReaderConfig;

mod cache;
mod config;

use crate::db::Database;
use crate::model::{PlainAccount, Address};
use crate::codec::AccountDecoder;
use std::sync::Arc;

/// High-level state reader with caching support
///
/// # Type Parameters
/// * `D` - Database implementation
/// * `C` - Cache implementation
pub struct StateReader<D, C> 
where
    D: Database,
    C: StateCache,
{
    db: Arc<D>,
    decoder: AccountDecoder,
    cache: C,
}

impl<D, C> StateReader<D, C>
where
    D: Database,
    C: StateCache,
{
    /// Creates a new state reader
    pub fn new(db: Arc<D>, cache: C) -> Self {
        Self {
            db,
            decoder: AccountDecoder::new(),
            cache,
        }
    }
    
    /// Retrieves an account by address
    ///
    /// # Arguments
    /// * `address` - The account address
    ///
    /// # Returns
    /// * `Ok(Some(account))` - Account found
    /// * `Ok(None)` - Account not found
    /// * `Err(e)` - Read error occurred
    ///
    /// # Examples
    /// ```
    /// use erc_mdbx_index::reader::StateReader;
    /// use erc_mdbx_index::model::Address;
    /// 
    /// let reader = StateReader::new(db, cache);
    /// let addr = Address::from_hex("0x...")?;
    /// let account = reader.get_account(&addr)?;
    /// ```
    pub fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>> {
        // Check cache first
        if let Some(account) = self.cache.get(address) {
            return Ok(Some(account));
        }
        
        // Read from database
        let txn = self.db.begin_read()?;
        let raw = txn.get("PlainState", address.as_bytes())?;
        
        let account = match raw {
            Some(data) => {
                let account = self.decoder.decode(&data)?;
                self.cache.insert(address.clone(), account.clone());
                Some(account)
            }
            None => None,
        };
        
        Ok(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MockDatabase;
    use crate::reader::MockCache;
    
    #[test]
    fn test_get_account_from_cache() {
        let db = Arc::new(MockDatabase::new());
        let mut cache = MockCache::new();
        
        let addr = Address::from_hex("0x123").unwrap();
        let expected = PlainAccount::new(1, U256::from(1000), None).unwrap();
        cache.insert(addr.clone(), expected.clone());
        
        let reader = StateReader::new(db, cache);
        let result = reader.get_account(&addr).unwrap();
        
        assert_eq!(result, Some(expected));
    }
}
```

---

### Example 2: Utility Layer Module

**File**: `src/util/signal.rs`

```rust
//! Signal handling utilities
//!
//! Provides graceful shutdown support for long-running processes
//! by capturing and handling OS signals.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use signal_hook::consts::{SIGINT, SIGTERM};

/// Signal handler for graceful shutdown
#[derive(Clone)]
pub struct SignalHandler {
    shutdown: Arc<AtomicBool>,
}

impl SignalHandler {
    /// Creates a new signal handler and registers signal hooks
    ///
    /// # Returns
    /// * `Ok(handler)` - Handler created and registered
    /// * `Err(e)` - Signal registration failed
    ///
    /// # Examples
    /// ```
    /// use erc_mdbx_index::util::SignalHandler;
    /// 
    /// let handler = SignalHandler::new()?;
    /// 
    /// while !handler.should_shutdown() {
    ///     // Process data
    /// }
    /// 
    /// println!("Shutting down gracefully...");
    /// ```
    pub fn new() -> Result<Self> {
        let shutdown = Arc::new(AtomicBool::new(false));
        
        let shutdown_clone = shutdown.clone();
        signal_hook::flag::register(SIGINT, shutdown_clone)?;
        
        let shutdown_clone = shutdown.clone();
        signal_hook::flag::register(SIGTERM, shutdown_clone)?;
        
        Ok(Self { shutdown })
    }
    
    /// Checks if shutdown was requested
    pub fn should_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }
    
    /// Manually triggers shutdown
    pub fn trigger_shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new().expect("Failed to register signal handlers")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manual_shutdown() {
        let handler = SignalHandler::new().unwrap();
        assert!(!handler.should_shutdown());
        
        handler.trigger_shutdown();
        assert!(handler.should_shutdown());
    }
}
```

---

## Module Checklist

Before considering a module complete:

- [ ] Module-level documentation (`//!`) present
- [ ] Public API documented with examples
- [ ] Appropriate visibility modifiers applied
- [ ] Re-exports organized in mod.rs
- [ ] Tests cover public API
- [ ] Error handling at module boundaries
- [ ] Follows layer dependency rules
- [ ] No glob imports
- [ ] Minimal public surface area
- [ ] Rust idioms followed (trait implementations, Display, Debug, etc.)

---

**Version**: 1.0.0  
**Last Updated**: 2025-12-05
