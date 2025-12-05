# Constitution Alignment Guide

## Overview

This document explicitly maps the erc-mdbx-index project constitution principles to the modular design documentation, ensuring that architectural decisions align with project values.

## Constitution Principle Mapping

### 1. 内存安全优先 (Memory Safety First)

**English**: Memory Safety First

**Alignment in Documentation**:
- **[Dependency Injection Guide](dependency-injection-guide.md)**: Explains ownership model considerations for trait-based injection
- **[Zero-Cost Abstractions Guide](zero-cost-abstractions-guide.md)**: Documents when to use static vs dynamic dispatch
- **[Layer Definitions](layer-definitions.md)**: Database Abstraction Layer encapsulates unsafe MDBX operations

**Key Practices**:
```rust
// Safe abstraction over unsafe FFI boundary
pub trait Database: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}

// Ensures data doesn't escape transaction scope
impl Database for MdbxDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let txn = self.begin_read()?;
        let data = txn.get(key)?;
        // Clone data before transaction ends - "read and copy" strategy
        Ok(data.map(|v| v.to_vec()))
    }
}
```

**Constitution Reference**: "明确区分'零拷贝读取'与'所有权拷贝'" (Clear distinction between zero-copy reads and ownership copies)

---

### 2. 测试驱动开发 (Test-Driven Development)

**English**: Test-Driven Development

**Alignment in Documentation**:
- **[Integration Test Examples](integration-test-examples.md)**: Demonstrates cross-layer test setup with fixtures
- **[Trait-First Pattern Guide](trait-first-pattern-guide.md)**: Shows how trait abstraction enables mocking
- **[TDD Workflow Guide](tdd-workflow-guide.md)**: Red-Green-Refactor cycle for modular Rust code

**Key Practices**:
```rust
// Step 1: Define trait (interface first)
pub trait StateReader {
    fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>>;
}

// Step 2: Write test with mock
#[cfg(test)]
mod tests {
    struct MockStateReader {
        accounts: HashMap<Address, PlainAccount>,
    }
    
    impl StateReader for MockStateReader {
        fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>> {
            Ok(self.accounts.get(address).cloned())
        }
    }
    
    #[test]
    fn test_account_lookup() {
        let mut mock = MockStateReader::new();
        mock.accounts.insert(test_address(), test_account());
        
        let result = mock.get_account(&test_address())?;
        assert!(result.is_some());
    }
}

// Step 3: Implement concrete type after tests pass
struct MdbxStateReader { /* ... */ }
impl StateReader for MdbxStateReader { /* ... */ }
```

**Constitution Reference**: "遵循红-绿-重构(Red-Green-Refactor)迭代周期" (Follow red-green-refactor iterative cycle)

---

### 3. 分层架构 (Layered Architecture)

**English**: Layered Architecture

**Alignment in Documentation**:
- **[Layer Definitions](layer-definitions.md)**: Defines DAL/Model/Codec/Reader/Util layers
- **[Dependency Direction Rules](dependency-direction-rules.md)**: Enforces upward-only dependencies
- **[Layer Placement Examples](layer-placement-examples.md)**: Concrete examples of correct layer usage

**Layer Terminology Mapping**:

| Constitution Term | Documentation Term | Scope |
|-------------------|-------------------|-------|
| 数据库抽象层 (DAL) | Database Abstraction Layer | MDBX environment, transactions, cursors |
| 数据模型层 (Model) | Model Layer | PlainAccount, domain entities, type-safe wrappers |
| 业务逻辑层 (Service) | Reader Layer + Codec Layer | State reading (Reader) + RLP encoding/decoding (Codec) |
| 跨层工具 (Utilities) | Utility Layer | Configuration, signal handling, generic helpers |

**Key Practices**:
```rust
// Correct dependency flow (upward only)
// db/ → Independent, no imports from higher layers
// model/ → Independent, defines domain types
// codec/ → Imports model/ for type definitions
// reader/ → Imports db/, model/, codec/ for orchestration
// util/ → Independent utilities used by all layers

// Example: Reader layer correctly depends on lower layers
use crate::db::Database;           // ✓ Lower layer
use crate::model::PlainAccount;    // ✓ Lower layer
use crate::codec::RlpDecoder;      // ✓ Lower layer

pub struct StateReader<D: Database> {
    db: D,
    decoder: RlpDecoder,
}
```

**Constitution Reference**: "代码结构遵循清晰的分层模式以确保可维护性和扩展性" (Code structure follows clear layered pattern for maintainability and extensibility)

---

### 4. 性能至上 (Performance First)

**English**: Performance First

**Alignment in Documentation**:
- **[Zero-Cost Abstractions Guide](zero-cost-abstractions-guide.md)**: When to use generics vs trait objects
- **[Performance Considerations](performance-considerations.md)**: Trait dispatch overhead analysis
- **[Short Transaction Pattern](short-transaction-pattern.md)**: Avoiding MDBX file bloat

**Key Practices**:
```rust
// Static dispatch (zero overhead) - preferred for hot paths
pub struct StateReader<D: Database> {
    db: D,  // Generic type parameter - monomorphized at compile time
}

impl<D: Database> StateReader<D> {
    pub fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>> {
        // No runtime dispatch overhead
        self.db.get(address.as_bytes())
    }
}

// Dynamic dispatch (2-10ns overhead) - acceptable for I/O-bound operations
pub struct MultiReader {
    readers: Vec<Box<dyn StateReader>>,  // Trait object for runtime polymorphism
}

// Short transaction pattern to prevent MDBX bloat
impl Database for MdbxDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let txn = self.begin_read()?;       // Start transaction
        let result = txn.get(key)?.map(|v| v.to_vec());  // Copy data
        drop(txn);                          // End transaction immediately
        Ok(result)
    }
}
```

**Performance Targets** (from Constitution):
- Single-core account read throughput: ≥1,250,000 ops/sec
- Performance improvement vs JSON-RPC: ≥100x
- Memory safety overhead: 0 (zero-cost abstractions)

**Constitution Reference**: "目标吞吐量：>1,000,000 Ops/sec" (Target throughput: >1,000,000 Ops/sec)

---

### 5. 快速失败 (Fail Fast)

**English**: Fail Fast

**Alignment in Documentation**:
- **[Error Boundary Guide](error-boundary-guide.md)**: Module-level error type conversion
- **[Error Type API Review Guide](error-type-api-review-guide.md)**: Proper error handling at layer boundaries
- **[Architectural Compliance Checklist](architectural-compliance-checklist.md)**: Pre-commit validation

**Key Practices**:
```rust
// Define module-specific error types with context
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
    
    #[error("Transaction aborted: {reason}")]
    TransactionAborted { reason: String },
    
    #[error("MDBX error: {0}")]
    Mdbx(#[from] libmdbx::Error),
}

// Convert at layer boundaries
#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("Database access failed: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
}

// Fail fast on version mismatch
pub fn open_database(path: &Path) -> Result<Database> {
    let db = MdbxDatabase::open(path)?;
    let version = db.get_version()?;
    
    if version != EXPECTED_VERSION {
        return Err(DatabaseError::VersionMismatch {
            expected: EXPECTED_VERSION,
            found: version,
        });
    }
    
    Ok(db)
}
```

**Constitution Reference**: "数据库版本不匹配时立即报错，防止输出错误数据" (Report error immediately on database version mismatch to prevent incorrect data output)

---

## Constitution Non-Negotiables Coverage

### 1. "绝不在unsafe代码中引入未定义行为"
**Translation**: Never introduce undefined behavior in unsafe code

**Documentation Coverage**:
- **Memory Safety Patterns**: Guidelines for safe FFI wrappers around MDBX
- **Database Abstraction Layer**: Encapsulates all unsafe operations
- **Code Review Checklist**: Mandatory unsafe code review items

**Example Pattern**:
```rust
// Safe wrapper around unsafe MDBX call
pub fn get_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
    unsafe {
        let mut data_ptr: *const u8 = std::ptr::null();
        let mut data_len: usize = 0;
        
        let rc = mdbx_get(
            self.txn,
            self.dbi,
            key.as_ptr(),
            key.len(),
            &mut data_ptr,
            &mut data_len,
        );
        
        if rc == MDBX_NOTFOUND {
            return Ok(None);
        }
        
        if rc != MDBX_SUCCESS {
            return Err(MdbxError::from_code(rc));
        }
        
        // Safe: pointer and length validated by MDBX
        // Data copied immediately to ensure it doesn't escape transaction
        let slice = std::slice::from_raw_parts(data_ptr, data_len);
        Ok(Some(slice.to_vec()))
    }
}
```

---

### 2. "绝不长时间持有读事务"
**Translation**: Never hold read transactions for extended periods

**Documentation Coverage**:
- **Short Transaction Pattern**: Immediate transaction closure pattern
- **Database Layer Guidelines**: Transaction lifecycle management
- **Performance Considerations**: Impact of long-lived transactions on MDBX page reclamation

**Example Pattern**:
```rust
// ✓ Correct: Short-lived transaction
pub fn batch_read(&self, keys: &[&[u8]]) -> Result<Vec<Option<Vec<u8>>>> {
    keys.iter()
        .map(|key| {
            let txn = self.begin_read()?;   // New transaction per read
            let result = txn.get(key)?.map(|v| v.to_vec());
            drop(txn);                       // Immediate closure
            Ok(result)
        })
        .collect()
}

// ✗ Incorrect: Long-lived transaction
pub fn batch_read_bad(&self, keys: &[&[u8]]) -> Result<Vec<Option<Vec<u8>>>> {
    let txn = self.begin_read()?;           // Single transaction
    let results: Vec<_> = keys.iter()
        .map(|key| txn.get(key).map(|v| v.map(|d| d.to_vec())))
        .collect::<Result<_>>()?;
    drop(txn);                              // Held for entire batch
    Ok(results)
}
```

---

### 3. "绝不假设RLP字段完整性"
**Translation**: Never assume RLP field completeness

**Documentation Coverage**:
- **Codec Layer Error Handling**: Defensive RLP parsing
- **Context-Aware Decoding**: Field omission optimization handling
- **Model Layer Validation**: Default value inference

**Example Pattern**:
```rust
// Context-aware decoder handles field omissions
pub struct AccountDecoder;

impl AccountDecoder {
    pub fn decode(data: &[u8]) -> Result<PlainAccount> {
        let rlp = rlp::Rlp::new(data);
        
        // Erigon optimization: fields may be omitted
        let nonce = if rlp.item_count()? > 0 {
            rlp.val_at(0)?
        } else {
            0  // Default value for omitted field
        };
        
        let balance = if rlp.item_count()? > 1 {
            rlp.val_at(1)?
        } else {
            U256::zero()  // Default value
        };
        
        Ok(PlainAccount { nonce, balance })
    }
}
```

---

### 4. "绝不忽略版本兼容性"
**Translation**: Never ignore version compatibility

**Documentation Coverage**:
- **Architectural Evolution Guidelines**: Schema change handling
- **Backward Compatibility Patterns**: Version detection and adaptation
- **Migration Strategies**: Phased adoption of new data layouts

**Example Pattern**:
```rust
pub enum DatabaseVersion {
    Erigon2,
    Erigon3,
    Akula,
}

pub struct VersionedReader {
    version: DatabaseVersion,
}

impl VersionedReader {
    pub fn open(path: &Path) -> Result<Self> {
        let version = Self::detect_version(path)?;
        Ok(Self { version })
    }
    
    pub fn get_account(&self, address: &Address) -> Result<Option<PlainAccount>> {
        match self.version {
            DatabaseVersion::Erigon2 => self.get_account_v2(address),
            DatabaseVersion::Erigon3 => self.get_account_v3(address),
            DatabaseVersion::Akula => self.get_account_akula(address),
        }
    }
}
```

---

## Compliance Verification

### Pre-Implementation Checklist

Before starting a new module or feature, verify:

- [ ] Layer placement follows dependency rules (no downward dependencies)
- [ ] Memory safety patterns applied to all unsafe code
- [ ] Test-first approach: traits defined, tests written, then implementation
- [ ] Performance targets identified (static vs dynamic dispatch decision made)
- [ ] Error types defined at module boundaries with proper conversion
- [ ] Short transaction pattern applied to database access
- [ ] Defensive RLP parsing with field omission handling
- [ ] Version compatibility strategy documented

### Post-Implementation Review

After completing a module or feature:

- [ ] All constitution principles reflected in code
- [ ] Non-negotiables verified (no unsafe UB, short transactions, RLP defense, version checks)
- [ ] Architecture review checklist completed
- [ ] Integration tests demonstrate constitution compliance
- [ ] Documentation updated to reflect architectural decisions

---

## Quick Reference

| Constitution Principle | Key Documentation Files | Primary Patterns |
|------------------------|-------------------------|------------------|
| Memory Safety First | dependency-injection-guide.md, zero-cost-abstractions-guide.md | Safe FFI wrappers, ownership-aware traits |
| Test-Driven Development | integration-test-examples.md, tdd-workflow-guide.md | Trait-first design, mock implementations |
| Layered Architecture | layer-definitions.md, dependency-direction-rules.md | Upward-only dependencies, explicit imports |
| Performance First | performance-considerations.md, short-transaction-pattern.md | Static dispatch preference, immediate transaction closure |
| Fail Fast | error-boundary-guide.md, error-type-api-review-guide.md | Module-specific errors, version checks |

---

**Version**: 1.0.0  
**Last Updated**: 2025-12-05  
**Aligned With**: erc-mdbx-index Constitution v2.0
