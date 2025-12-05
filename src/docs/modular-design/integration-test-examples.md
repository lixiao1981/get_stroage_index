# Integration Test Examples

## Overview

This guide demonstrates cross-layer integration testing patterns for modular Rust code, including fixture-based testing and mock implementations.

## Example 1: Cross-Layer Database to Reader Integration

### Test Setup

```rust
//! Integration test for StateReader across db, model, and codec layers
use erc_mdbx_index::db::MdbxDatabase;
use erc_mdbx_index::reader::StateReader;
use erc_mdbx_index::model::Address;
use tempfile::TempDir;

#[test]
fn test_state_reader_integration() -> Result<()> {
    // Setup: Create temporary database
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.mdbx");
    
    // Initialize database with test data
    let db = Arc::new(MdbxDatabase::open(&db_path)?);
    seed_test_data(&db)?;
    
    // Create reader with real dependencies
    let reader = StateReader::new(db);
    
    // Test: Read account that was seeded
    let address = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb")?;
    let account = reader.get_account(&address)?;
    
    // Verify: Account exists with expected values
    assert!(account.is_some());
    let account = account.unwrap();
    assert_eq!(account.nonce(), 42);
    assert_eq!(account.balance(), &U256::from(1_000_000_000u64));
    
    Ok(())
}

fn seed_test_data(db: &Arc<MdbxDatabase>) -> Result<()> {
    let mut txn = db.begin_write()?;
    
    // Insert test account using RLP encoding
    let address = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb")?;
    let account = PlainAccount::new(42, U256::from(1_000_000_000u64), None)?;
    
    let encoded = AccountEncoder::encode(&account)?;
    txn.put("PlainState", address.as_bytes(), &encoded)?;
    
    txn.commit()?;
    Ok(())
}
```

### Mock Implementations

```rust
// Mock database for testing without I/O
#[cfg(test)]
pub struct MockDatabase {
    data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

#[cfg(test)]
impl MockDatabase {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn insert(&self, key: Vec<u8>, value: Vec<u8>) {
        self.data.lock().unwrap().insert(key, value);
    }
}

#[cfg(test)]
impl Database for MockDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.data.lock().unwrap().get(key).cloned())
    }
    
    fn begin_read(&self) -> Result<Box<dyn Transaction>> {
        Ok(Box::new(MockTransaction {
            data: self.data.clone(),
        }))
    }
}

// Test using mock
#[test]
fn test_state_reader_with_mock() {
    let mock_db = Arc::new(MockDatabase::new());
    
    // Pre-populate mock database
    let address = Address::from_hex("0x123").unwrap();
    let account = PlainAccount::new(1, U256::from(100), None).unwrap();
    let encoded = AccountEncoder::encode(&account).unwrap();
    
    mock_db.insert(address.as_bytes().to_vec(), encoded);
    
    // Test reader with mock
    let reader = StateReader::new(mock_db);
    let result = reader.get_account(&address).unwrap();
    
    assert!(result.is_some());
    assert_eq!(result.unwrap().nonce(), 1);
}
```

---

## Example 2: Codec and Model Layer Integration

### Fixture-Based Testing

```rust
//! Integration test for RLP codec with model layer types
use erc_mdbx_index::codec::{AccountEncoder, AccountDecoder};
use erc_mdbx_index::model::PlainAccount;

// Test fixture structure
struct AccountFixture {
    description: &'static str,
    account: PlainAccount,
    expected_rlp: Vec<u8>,
}

impl AccountFixture {
    fn fixtures() -> Vec<Self> {
        vec![
            Self {
                description: "EOA with zero balance",
                account: PlainAccount::new(0, U256::zero(), None).unwrap(),
                expected_rlp: vec![0xc0],  // Empty RLP list
            },
            Self {
                description: "EOA with balance",
                account: PlainAccount::new(5, U256::from(1000), None).unwrap(),
                expected_rlp: vec![0xc2, 0x05, 0x82, 0x03, 0xe8],
            },
            Self {
                description: "Contract account",
                account: PlainAccount::new(
                    10,
                    U256::from(5000),
                    Some([1u8; 32])
                ).unwrap(),
                expected_rlp: vec![/* RLP bytes */],
            },
        ]
    }
}

#[test]
fn test_account_encoding_roundtrip() {
    let encoder = AccountEncoder::new();
    let decoder = AccountDecoder::new();
    
    for fixture in AccountFixture::fixtures() {
        // Test encoding
        let encoded = encoder.encode(&fixture.account)
            .expect(&format!("Failed to encode: {}", fixture.description));
        
        assert_eq!(
            encoded,
            fixture.expected_rlp,
            "Encoding mismatch for: {}",
            fixture.description
        );
        
        // Test decoding (roundtrip)
        let decoded = decoder.decode(&encoded)
            .expect(&format!("Failed to decode: {}", fixture.description));
        
        assert_eq!(
            decoded,
            fixture.account,
            "Roundtrip mismatch for: {}",
            fixture.description
        );
    }
}

#[test]
fn test_field_omission_handling() {
    let decoder = AccountDecoder::new();
    
    // Erigon optimization: omitted nonce field defaults to 0
    let rlp_without_nonce = vec![0xc0];  // Empty RLP list
    let account = decoder.decode(&rlp_without_nonce).unwrap();
    
    assert_eq!(account.nonce(), 0, "Omitted nonce should default to 0");
    assert_eq!(account.balance(), &U256::zero(), "Omitted balance should default to 0");
}
```

---

## Fixture Patterns

### File-Based Fixtures

```rust
// Load test data from files
#[test]
fn test_with_file_fixtures() -> Result<()> {
    let fixtures_dir = Path::new("tests/fixtures/accounts");
    
    for entry in std::fs::read_dir(fixtures_dir)? {
        let path = entry?.path();
        if path.extension() != Some("json".as_ref()) {
            continue;
        }
        
        let fixture: AccountTestCase = serde_json::from_reader(
            std::fs::File::open(&path)?
        )?;
        
        let reader = StateReader::new(setup_db_with_fixture(&fixture)?);
        let result = reader.get_account(&fixture.address)?;
        
        assert_eq!(result, fixture.expected_account);
    }
    
    Ok(())
}

#[derive(Deserialize)]
struct AccountTestCase {
    address: Address,
    expected_account: Option<PlainAccount>,
    rlp_bytes: Vec<u8>,
}
```

### Programmatic Fixtures

```rust
// Generate test data programmatically
pub struct TestDataBuilder {
    accounts: Vec<(Address, PlainAccount)>,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self { accounts: Vec::new() }
    }
    
    pub fn with_eoa(mut self, nonce: u64, balance: U256) -> Self {
        let address = Address::random();
        let account = PlainAccount::new(nonce, balance, None).unwrap();
        self.accounts.push((address, account));
        self
    }
    
    pub fn with_contract(mut self, code_hash: [u8; 32]) -> Self {
        let address = Address::random();
        let account = PlainAccount::new(0, U256::zero(), Some(code_hash)).unwrap();
        self.accounts.push((address, account));
        self
    }
    
    pub fn build(self) -> Arc<MockDatabase> {
        let db = Arc::new(MockDatabase::new());
        
        for (address, account) in self.accounts {
            let encoded = AccountEncoder::encode(&account).unwrap();
            db.insert(address.as_bytes().to_vec(), encoded);
        }
        
        db
    }
}

#[test]
fn test_with_builder_fixtures() {
    let db = TestDataBuilder::new()
        .with_eoa(0, U256::zero())
        .with_eoa(100, U256::from(1_000_000))
        .with_contract([0x42; 32])
        .build();
    
    let reader = StateReader::new(db);
    // Test with generated data
}
```

---

## Best Practices

1. **Isolate External Dependencies**: Use temporary directories for file-based tests
2. **Clean Up Resources**: Use `Drop` or `defer` patterns to ensure cleanup
3. **Test Real Paths**: Integration tests should use production code paths
4. **Mock Sparingly**: Only mock at architectural boundaries
5. **Use Fixtures**: Reusable test data improves maintainability

**Version**: 1.0.0
