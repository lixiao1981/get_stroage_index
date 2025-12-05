# Layer Placement Code Examples

## Overview

This document provides concrete code examples demonstrating how to place functionality in different architectural layers for the erc-mdbx-index project.

## Database Abstraction Layer (DAL) Examples

### 1. Transaction Management

```rust
// db/transaction.rs
pub struct ReadTransaction<'env> {
    inner: libmdbx::Transaction<'env, RO>,
}

impl<'env> ReadTransaction<'env> {
    pub fn get(&self, bucket: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Low-level byte retrieval from database
        self.inner.get(bucket, key)
            .map_err(|e| DatabaseError::ReadFailed(e))
    }

    pub fn cursor(&self, bucket: &str) -> Result<Box<dyn Cursor>> {
        // Create low-level database cursor
        let raw_cursor = self.inner.cursor(bucket)?;
        Ok(Box::new(MdbxCursor::new(raw_cursor)))
    }
}

pub trait Cursor {
    fn seek(&mut self, key: &[u8]) -> Result<Option<(Vec<u8>, Vec<u8>)>>;
    fn next(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>>;
}
```

### 2. Database Configuration

```rust
// db/environment.rs
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

    pub fn writable(mut self) -> Self {
        self.read_only = false;
        self
    }

    pub fn build(self) -> Result<DatabaseConfig> {
        Ok(DatabaseConfig {
            path: self.path.ok_or(DatabaseError::MissingPath)?,
            max_dbs: self.max_dbs,
            read_only: self.read_only,
        })
    }
}
```

## Model Layer Examples

### 1. Domain Entity with Validation

```rust
// model/account.rs
#[derive(Debug, Clone, PartialEq)]
pub struct PlainAccount {
    address: Address,
    nonce: u64,
    balance: U256,
    storage_root: Option<H256>,
    code_hash: Option<H256>,
}

impl PlainAccount {
    pub fn new(
        address: Address,
        nonce: u64,
        balance: U256
    ) -> Result<Self> {
        // Validation logic
        if balance > U256::max_value() / 2 {
            return Err(ModelError::InvalidBalance);
        }

        Ok(Self {
            address,
            nonce,
            balance,
            storage_root: None,
            code_hash: None,
        })
    }

    pub fn update_balance(&mut self, delta: I256) -> Result<()> {
        let new_balance = self.balance.checked_add(delta)
            .ok_or(ModelError::BalanceOverflow)?;

        self.balance = new_balance;
        Ok(())
    }
}
```

### 2. Strong Type Wrapper

```rust
// model/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 20 {
            return Err(ModelError::InvalidAddressLength);
        }
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&slice[..20]);
        Ok(Self(addr))
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}
```

## Codec Layer Examples

### 1. RLP Encoding/Decoding

```rust
// codec/rlp.rs
pub struct AccountCodec;

impl AccountCodec {
    pub fn encode(account: &PlainAccount) -> Vec<u8> {
        rlp::encode(&(
            account.nonce,
            account.balance,
            account.storage_root,
            account.code_hash
        ))
    }

    pub fn decode(data: &[u8]) -> Result<PlainAccount> {
        let (nonce, balance, storage_root, code_hash) =
            rlp::decode(data)?;

        Ok(PlainAccount {
            address: Address::zero(), // Separate address handling
            nonce,
            balance,
            storage_root,
            code_hash,
        })
    }
}
```

### 2. Flexible Transformation

```rust
// codec/transformer.rs
pub trait Transformer<I, O> {
    fn transform(&self, input: I) -> Result<O>;
}

pub struct RlpToJsonTransformer;

impl Transformer<Vec<u8>, serde_json::Value> for RlpToJsonTransformer {
    fn transform(&self, rlp_data: Vec<u8>) -> Result<serde_json::Value> {
        let account = AccountCodec::decode(&rlp_data)?;
        Ok(serde_json::json!({
            "nonce": account.nonce,
            "balance": account.balance.to_string(),
            "storage_root": account.storage_root,
            "code_hash": account.code_hash
        }))
    }
}
```

## Reader Layer Examples

### 1. State Reading Operations

```rust
// reader/state_reader.rs
pub struct StateReader<D: Database, C: AccountCodec> {
    db: D,
    codec: C,
}

impl<D: Database, C: AccountCodec> StateReader<D, C> {
    pub fn get_account(&self, address: Address) -> Result<Option<PlainAccount>> {
        // Coordinate between database and model layers
        let raw_bytes = self.db.get(address.as_bytes())?;

        raw_bytes.map(|bytes| {
            let mut account = self.codec.decode(&bytes)?;
            account.address = address; // Set address from lookup
            Ok(account)
        }).transpose()
    }

    pub fn list_accounts(&self, start: Option<Address>) -> Result<Vec<PlainAccount>> {
        // Complex reading strategy
        let mut accounts = Vec::new();
        let mut cursor = self.db.cursor(ACCOUNTS_BUCKET)?;

        // Iterate and decode accounts
        while let Some((addr_bytes, account_bytes)) = cursor.next()? {
            let address = Address::from_slice(&addr_bytes)?;
            let account = self.codec.decode(&account_bytes)?;
            accounts.push(account);
        }

        Ok(accounts)
    }
}
```

### 2. Specialized Iteration

```rust
// reader/account_iterator.rs
pub struct AccountIterator<'a, D: Database, C: AccountCodec> {
    db: &'a D,
    codec: &'a C,
    cursor: Box<dyn Cursor>,
}

impl<'a, D: Database, C: AccountCodec> Iterator for AccountIterator<'a, D, C> {
    type Item = Result<PlainAccount>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor.next() {
            Ok(Some((addr_bytes, account_bytes))) => {
                let address = match Address::from_slice(&addr_bytes) {
                    Ok(addr) => addr,
                    Err(e) => return Some(Err(e)),
                };

                let mut account = match self.codec.decode(&account_bytes) {
                    Ok(acc) => acc,
                    Err(e) => return Some(Err(e)),
                };

                account.address = address;
                Some(Ok(account))
            },
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
```

## Layer Interaction Example

```rust
// High-level usage demonstrating layer interactions
fn main() -> Result<()> {
    // DAL: Configure database
    let config = DatabaseConfigBuilder::new()
        .path("/path/to/erigon/chaindata".into())
        .writable()
        .build()?;

    let db = Database::open(&config)?;

    // Codec: Create transformer
    let rlp_transformer = RlpToJsonTransformer;

    // Reader: Coordinate layers
    let state_reader = StateReader::new(db, rlp_transformer);

    // Retrieve and process account
    let address = Address::from_slice(&[0; 20])?;
    let account = state_reader.get_account(address)?;

    // Model: Domain logic
    if let Some(mut account) = account {
        account.update_balance(I256::from(1000))?;
    }

    Ok(())
}
```

## Best Practices

1. Keep layers independent
2. Use traits for abstractions
3. Minimize dependencies between layers
4. Validate at layer boundaries
5. Provide clear error handling

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05