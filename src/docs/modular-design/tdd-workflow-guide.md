# Test-Driven Development Workflow for Modular Rust

## Overview

This guide demonstrates the Red-Green-Refactor cycle for developing modular Rust code, aligned with the erc-mdbx-index constitution's TDD principle.

## Red-Green-Refactor Cycle

```
RED → Write failing test
  ↓
GREEN → Write minimal code to pass
  ↓
REFACTOR → Improve code quality
  ↓
REPEAT
```

---

## Complete Example: Building a State Reader

### Phase 1: RED - Write Failing Test

**Step 1.1**: Define the interface (trait-first design)

```rust
// src/reader/state.rs

/// State reading operations
pub trait StateReader {
    fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>>;
}
```

**Step 1.2**: Write the test BEFORE implementation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Address, PlainAccount};
    
    #[test]
    fn test_get_existing_account() {
        // Arrange: Setup test data
        let address = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb").unwrap();
        let expected_account = PlainAccount::new(
            42,
            U256::from(1_000_000_000u64),
            None
        ).unwrap();
        
        // Create reader with mock database
        let reader = MdbxStateReader::with_mock_db(vec![
            (address.clone(), expected_account.clone())
        ]);
        
        // Act: Call the method we're testing
        let result = reader.get_account(&address).unwrap();
        
        // Assert: Verify expectations
        assert!(result.is_some());
        let account = result.unwrap();
        assert_eq!(account.nonce(), 42);
        assert_eq!(account.balance(), &U256::from(1_000_000_000u64));
    }
    
    #[test]
    fn test_get_nonexistent_account() {
        let address = Address::from_hex("0x0000000000000000000000000000000000000000").unwrap();
        let reader = MdbxStateReader::with_mock_db(vec![]);
        
        let result = reader.get_account(&address).unwrap();
        assert!(result.is_none());
    }
}
```

**Step 1.3**: Run tests (they should FAIL)

```bash
$ cargo test test_get_existing_account
   Compiling erc-mdbx-index v0.1.0
error[E0425]: cannot find function `with_mock_db` in this scope
  --> src/reader/state.rs:25:34
   |
25 |         let reader = MdbxStateReader::with_mock_db(vec![
   |                                       ^^^^^^^^^^^^^ not found in this scope

error: aborting due to previous error
```

✅ **RED phase complete**: Test fails as expected (code doesn't exist yet)

---

### Phase 2: GREEN - Write Minimal Implementation

**Step 2.1**: Create stub implementation to make tests compile

```rust
// src/reader/state.rs

use crate::db::Database;
use crate::model::{Address, PlainAccount};
use std::sync::Arc;

pub struct MdbxStateReader {
    db: Arc<dyn Database>,
}

impl MdbxStateReader {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }
    
    #[cfg(test)]
    pub fn with_mock_db(test_data: Vec<(Address, PlainAccount)>) -> Self {
        let mock_db = Arc::new(MockDatabase::new(test_data));
        Self::new(mock_db)
    }
}

impl StateReader for MdbxStateReader {
    fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>> {
        // TODO: Implement
        unimplemented!()
    }
}

#[cfg(test)]
struct MockDatabase {
    data: std::collections::HashMap<Address, PlainAccount>,
}

#[cfg(test)]
impl MockDatabase {
    fn new(test_data: Vec<(Address, PlainAccount)>) -> Self {
        Self {
            data: test_data.into_iter().collect(),
        }
    }
}

#[cfg(test)]
impl Database for MockDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let address = Address::from_bytes(key)?;
        Ok(self.data.get(&address).map(|account| {
            AccountEncoder::encode(account).unwrap()
        }))
    }
}
```

**Step 2.2**: Run tests (still FAIL but compile now)

```bash
$ cargo test test_get_existing_account
running 1 test
test reader::state::tests::test_get_existing_account ... FAILED

failures:

---- reader::state::tests::test_get_existing_account stdout ----
thread 'reader::state::tests::test_get_existing_account' panicked at 'not yet implemented'
```

**Step 2.3**: Implement minimal logic to pass tests

```rust
impl StateReader for MdbxStateReader {
    fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>> {
        // Read raw bytes from database
        let raw_data = self.db.get(address.as_bytes())?;
        
        // Decode if data exists
        match raw_data {
            Some(bytes) => {
                let account = AccountDecoder::decode(&bytes)?;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }
}
```

**Step 2.4**: Run tests (should PASS now)

```bash
$ cargo test test_get_existing_account
running 2 tests
test reader::state::tests::test_get_existing_account ... ok
test reader::state::tests::test_get_nonexistent_account ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

✅ **GREEN phase complete**: Tests pass with minimal implementation

---

### Phase 3: REFACTOR - Improve Code Quality

**Step 3.1**: Extract decoder as dependency (better testability)

```rust
pub struct MdbxStateReader {
    db: Arc<dyn Database>,
    decoder: AccountDecoder,  // Extracted dependency
}

impl MdbxStateReader {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            decoder: AccountDecoder::new(),
        }
    }
}

impl StateReader for MdbxStateReader {
    fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>> {
        let raw_data = self.db.get(address.as_bytes())?;
        
        match raw_data {
            Some(bytes) => {
                let account = self.decoder.decode(&bytes)?;  // Use injected decoder
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }
}
```

**Step 3.2**: Run tests (should still PASS)

```bash
$ cargo test
test result: ok. 2 passed; 0 failed; 0 ignored
```

**Step 3.3**: Add error handling test

```rust
#[test]
fn test_get_account_with_corrupted_data() {
    let address = Address::from_hex("0x123").unwrap();
    
    let mut mock_db = MockDatabase::new(vec![]);
    mock_db.insert_raw(address.as_bytes().to_vec(), vec![0xFF, 0xFF]);  // Invalid RLP
    
    let reader = MdbxStateReader::new(Arc::new(mock_db));
    let result = reader.get_account(&address);
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::CodecError(_)));
}
```

**Step 3.4**: Run all tests

```bash
$ cargo test
test result: ok. 3 passed; 0 failed; 0 ignored
```

✅ **REFACTOR phase complete**: Code improved, all tests still pass

---

## TDD Best Practices for Modular Code

### Practice 1: Test at Module Boundaries

```rust
// ✓ Good: Test the public interface
#[test]
fn test_state_reader_api() {
    let reader = StateReader::new(mock_db());
    let result = reader.get_account(&address);
    // Assert on public behavior
}

// ✗ Bad: Test internal implementation details
#[test]
fn test_internal_cache_structure() {
    let reader = StateReader::new(mock_db());
    assert_eq!(reader.cache.len(), 0);  // Brittle, couples to implementation
}
```

---

### Practice 2: One Test, One Assertion Focus

```rust
// ✓ Good: Clear test intent
#[test]
fn test_account_nonce_is_correct() {
    let account = reader.get_account(&addr).unwrap().unwrap();
    assert_eq!(account.nonce(), 42);
}

#[test]
fn test_account_balance_is_correct() {
    let account = reader.get_account(&addr).unwrap().unwrap();
    assert_eq!(account.balance(), &U256::from(1000));
}

// ✗ Bad: Multiple unrelated assertions
#[test]
fn test_account() {
    let account = reader.get_account(&addr).unwrap().unwrap();
    assert_eq!(account.nonce(), 42);
    assert_eq!(account.balance(), &U256::from(1000));
    assert!(account.is_eoa());
    // If first assertion fails, others never run
}
```

---

### Practice 3: Use Descriptive Test Names

```rust
// ✓ Good: Test name describes scenario
#[test]
fn get_account_returns_none_when_address_not_in_database() { }

#[test]
fn get_account_returns_decoded_account_when_address_exists() { }

#[test]
fn get_account_returns_error_when_rlp_data_is_corrupted() { }

// ✗ Bad: Generic test names
#[test]
fn test_get_account() { }

#[test]
fn test_get_account_2() { }
```

---

### Practice 4: Mock at Architectural Boundaries

```rust
// ✓ Good: Mock at layer boundary (Database trait)
#[cfg(test)]
impl Database for MockDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.test_data.get(key).cloned())
    }
}

let reader = StateReader::new(Arc::new(MockDatabase::new()));

// ✗ Bad: Mock internal components
let reader = StateReader {
    db: real_db,
    cache: MockCache::new(),           // Don't mock internal cache
    decoder: MockDecoder::new(),       // Don't mock internal decoder
};
```

---

## Common TDD Patterns

### Pattern 1: Table-Driven Tests

```rust
#[test]
fn test_decode_account_variants() {
    let test_cases = vec![
        ("EOA zero balance", vec![0xc0], PlainAccount::new(0, U256::zero(), None).unwrap()),
        ("EOA with balance", vec![0xc2, 0x05, 0x82, 0x03, 0xe8], PlainAccount::new(5, U256::from(1000), None).unwrap()),
        ("Contract account", vec![/* ... */], PlainAccount::new(10, U256::from(5000), Some([1u8; 32])).unwrap()),
    ];
    
    for (name, rlp, expected) in test_cases {
        let decoder = AccountDecoder::new();
        let result = decoder.decode(&rlp).expect(&format!("Failed: {}", name));
        assert_eq!(result, expected, "Test case: {}", name);
    }
}
```

---

### Pattern 2: Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode_roundtrip(nonce: u64, balance: u64) {
        let account = PlainAccount::new(nonce, U256::from(balance), None).unwrap();
        
        let encoded = AccountEncoder::encode(&account).unwrap();
        let decoded = AccountDecoder::decode(&encoded).unwrap();
        
        prop_assert_eq!(decoded, account);
    }
}
```

---

### Pattern 3: Builder Pattern for Test Data

```rust
struct TestAccountBuilder {
    nonce: u64,
    balance: U256,
    code_hash: Option<[u8; 32]>,
}

impl TestAccountBuilder {
    fn new() -> Self {
        Self {
            nonce: 0,
            balance: U256::zero(),
            code_hash: None,
        }
    }
    
    fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }
    
    fn with_balance(mut self, balance: U256) -> Self {
        self.balance = balance;
        self
    }
    
    fn build(self) -> PlainAccount {
        PlainAccount::new(self.nonce, self.balance, self.code_hash).unwrap()
    }
}

#[test]
fn test_with_builder() {
    let account = TestAccountBuilder::new()
        .with_nonce(100)
        .with_balance(U256::from(5000))
        .build();
    
    assert_eq!(account.nonce(), 100);
}
```

---

## TDD Workflow Checklist

Before writing any production code:

- [ ] Write trait/interface definition first
- [ ] Write at least one failing test
- [ ] Run tests to confirm they fail
- [ ] Write minimal code to pass tests
- [ ] Run tests to confirm they pass
- [ ] Refactor code for quality
- [ ] Run tests again to ensure no regression
- [ ] Add edge case tests
- [ ] Document test scenarios

**Version**: 1.0.0  
**Last Updated**: 2025-12-05
