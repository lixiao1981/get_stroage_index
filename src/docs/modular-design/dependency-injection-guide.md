# Dependency Injection Guide for Rust

## Overview

This guide explains dependency injection strategies specific to Rust's ownership model, covering trait-based injection, lifetime management, and zero-cost abstraction patterns.

## Strategy 1: Trait-Based Injection with Arc/Rc

### Use Case
Shared ownership across multiple components that need to outlive a single scope.

### Pattern

```rust
use std::sync::Arc;

// Define abstract interface
pub trait Database: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
}

// Consumer with injected dependency
pub struct StateReader {
    db: Arc<dyn Database>,
}

impl StateReader {
    /// Constructor injection with Arc for shared ownership
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }
    
    pub fn read_state(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.db.get(key)
    }
}

// Production implementation
struct MdbxDatabase {
    // Internal state
}

impl Database for MdbxDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Real implementation
        Ok(Some(vec![]))
    }
    
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        // Real implementation
        Ok(())
    }
}

// Test implementation
#[cfg(test)]
struct MockDatabase {
    data: std::collections::HashMap<Vec<u8>, Vec<u8>>,
}

#[cfg(test)]
impl Database for MockDatabase {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(key).cloned())
    }
    
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }
}

// Usage in production
fn main() -> Result<()> {
    let db = Arc::new(MdbxDatabase::new()?);
    let reader = StateReader::new(db.clone());
    
    // db can be shared with other components
    let writer = StateWriter::new(db.clone());
    
    Ok(())
}

// Usage in tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_reader() {
        let mut mock_db = MockDatabase { data: HashMap::new() };
        mock_db.data.insert(b"key1".to_vec(), b"value1".to_vec());
        
        let reader = StateReader::new(Arc::new(mock_db));
        let result = reader.read_state(b"key1").unwrap();
        
        assert_eq!(result, Some(b"value1".to_vec()));
    }
}
```

### When to Use
- ✅ Multi-threaded environments (requires `Send + Sync`)
- ✅ Shared ownership across components
- ✅ Dynamic dispatch acceptable (I/O-bound operations)
- ❌ Hot paths requiring zero-cost abstraction

### Trade-offs
- **Pros**: Flexible, supports runtime polymorphism, easy mocking
- **Cons**: Small runtime overhead (vtable dispatch ~2-10ns), heap allocation for Arc

---

## Strategy 2: Lifetime-Parameterized Injection

### Use Case
Zero-cost abstraction with static dispatch, optimal for performance-critical code.

### Pattern

```rust
// Define abstract interface with associated types
pub trait Database {
    type Transaction<'txn>: Transaction
    where
        Self: 'txn;
    
    fn begin_read(&self) -> Result<Self::Transaction<'_>>;
}

pub trait Transaction {
    fn get(&self, key: &[u8]) -> Result<Option<&[u8]>>;
}

// Consumer with generic type parameter (monomorphization)
pub struct StateReader<D> {
    db: D,
}

impl<D: Database> StateReader<D> {
    /// Constructor injection with generic type
    /// 
    /// # Type Parameter
    /// * `D` - Database implementation (static dispatch)
    pub fn new(db: D) -> Self {
        Self { db }
    }
    
    /// Zero-cost read operation
    pub fn read_state<'a>(&'a self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let txn = self.db.begin_read()?;
        let result = txn.get(key)?;
        
        // Must clone because txn is borrowed
        Ok(result.map(|slice| slice.to_vec()))
    }
}

// Production implementation
struct MdbxDatabase {
    // Internal state
}

struct MdbxTransaction<'db> {
    _marker: std::marker::PhantomData<&'db MdbxDatabase>,
}

impl Database for MdbxDatabase {
    type Transaction<'txn> = MdbxTransaction<'txn>
    where
        Self: 'txn;
    
    fn begin_read(&self) -> Result<Self::Transaction<'_>> {
        Ok(MdbxTransaction {
            _marker: std::marker::PhantomData,
        })
    }
}

impl<'db> Transaction for MdbxTransaction<'db> {
    fn get(&self, key: &[u8]) -> Result<Option<&[u8]>> {
        // Real implementation with zero-copy read
        Ok(Some(b"value"))
    }
}

// Test implementation
#[cfg(test)]
struct MockDatabase {
    data: std::collections::HashMap<Vec<u8>, Vec<u8>>,
}

#[cfg(test)]
struct MockTransaction<'db> {
    db: &'db MockDatabase,
}

#[cfg(test)]
impl Database for MockDatabase {
    type Transaction<'txn> = MockTransaction<'txn>
    where
        Self: 'txn;
    
    fn begin_read(&self) -> Result<Self::Transaction<'_>> {
        Ok(MockTransaction { db: self })
    }
}

#[cfg(test)]
impl<'db> Transaction for MockTransaction<'db> {
    fn get(&self, key: &[u8]) -> Result<Option<&[u8]>> {
        Ok(self.db.data.get(key).map(|v| v.as_slice()))
    }
}

// Usage in production (zero runtime overhead)
fn main() -> Result<()> {
    let db = MdbxDatabase::new()?;
    let reader = StateReader::new(db);  // Monomorphized to MdbxDatabase
    
    let result = reader.read_state(b"key1")?;
    Ok(())
}

// Usage in tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_reader_zero_cost() {
        let mut mock_db = MockDatabase { data: HashMap::new() };
        mock_db.data.insert(b"key1".to_vec(), b"value1".to_vec());
        
        let reader = StateReader::new(mock_db);  // Monomorphized to MockDatabase
        let result = reader.read_state(b"key1").unwrap();
        
        assert_eq!(result, Some(b"value1".to_vec()));
    }
}
```

### When to Use
- ✅ Performance-critical hot paths
- ✅ Zero runtime overhead required
- ✅ Static dispatch preferred
- ❌ Need runtime polymorphism

### Trade-offs
- **Pros**: Zero runtime overhead, compiler optimizations, no heap allocation
- **Cons**: Code duplication (monomorphization), longer compile times, larger binary size

---

## Strategy 3: Constructor Injection with Builder Pattern

### Use Case
Complex dependency graphs with optional configuration.

### Pattern

```rust
// Dependencies
pub trait Database: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}

pub trait Cache: Send + Sync {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn insert(&self, key: Vec<u8>, value: Vec<u8>);
}

pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

// Complex component with multiple dependencies
pub struct StateReader {
    db: Arc<dyn Database>,
    cache: Option<Arc<dyn Cache>>,
    logger: Option<Arc<dyn Logger>>,
    config: ReaderConfig,
}

// Builder for flexible construction
pub struct StateReaderBuilder {
    db: Option<Arc<dyn Database>>,
    cache: Option<Arc<dyn Cache>>,
    logger: Option<Arc<dyn Logger>>,
    config: ReaderConfig,
}

impl StateReaderBuilder {
    pub fn new() -> Self {
        Self {
            db: None,
            cache: None,
            logger: None,
            config: ReaderConfig::default(),
        }
    }
    
    /// Required dependency
    pub fn database(mut self, db: Arc<dyn Database>) -> Self {
        self.db = Some(db);
        self
    }
    
    /// Optional dependency
    pub fn cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }
    
    /// Optional dependency
    pub fn logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = Some(logger);
        self
    }
    
    /// Configuration
    pub fn config(mut self, config: ReaderConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Build with validation
    pub fn build(self) -> Result<StateReader> {
        let db = self.db.ok_or_else(|| Error::MissingDependency("database"))?;
        
        Ok(StateReader {
            db,
            cache: self.cache,
            logger: self.logger,
            config: self.config,
        })
    }
}

impl StateReader {
    /// Builder-based constructor
    pub fn builder() -> StateReaderBuilder {
        StateReaderBuilder::new()
    }
    
    pub fn get_account(&self, address: &[u8]) -> Result<Option<Vec<u8>>> {
        // Log if logger present
        if let Some(logger) = &self.logger {
            logger.log(&format!("Reading account: {:?}", address));
        }
        
        // Check cache if present
        if let Some(cache) = &self.cache {
            if let Some(value) = cache.get(address) {
                return Ok(Some(value));
            }
        }
        
        // Read from database
        let result = self.db.get(address)?;
        
        // Update cache if present and value found
        if let (Some(cache), Some(ref value)) = (&self.cache, &result) {
            cache.insert(address.to_vec(), value.clone());
        }
        
        Ok(result)
    }
}

// Usage in production
fn main() -> Result<()> {
    let db = Arc::new(MdbxDatabase::new()?);
    let cache = Arc::new(LruCache::new(1000));
    let logger = Arc::new(FileLogger::new("app.log")?);
    
    let reader = StateReader::builder()
        .database(db)
        .cache(cache)
        .logger(logger)
        .config(ReaderConfig {
            batch_size: 100,
            timeout_ms: 5000,
        })
        .build()?;
    
    Ok(())
}

// Usage in tests (minimal dependencies)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_reader_minimal() {
        let mock_db = Arc::new(MockDatabase::new());
        
        let reader = StateReader::builder()
            .database(mock_db)
            .build()
            .unwrap();
        
        let result = reader.get_account(b"address").unwrap();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_state_reader_with_cache() {
        let mock_db = Arc::new(MockDatabase::new());
        let mock_cache = Arc::new(MockCache::new());
        
        let reader = StateReader::builder()
            .database(mock_db)
            .cache(mock_cache)
            .build()
            .unwrap();
        
        // Test caching behavior
    }
}
```

### When to Use
- ✅ Multiple dependencies (3+)
- ✅ Optional dependencies
- ✅ Complex configuration
- ✅ Need validation before construction

### Trade-offs
- **Pros**: Flexible, self-documenting, compile-time validation
- **Cons**: More boilerplate, requires builder maintenance

---

## Comparison Matrix

| Strategy | Runtime Overhead | Compile Time | Testability | Flexibility | Best For |
|----------|-----------------|--------------|-------------|-------------|----------|
| Arc/Rc Trait Objects | Low (~2-10ns) | Fast | Excellent | High | Shared ownership, I/O-bound |
| Lifetime-Parameterized | Zero | Slow | Excellent | Medium | Hot paths, zero-cost |
| Constructor + Builder | Low (~2-10ns) | Medium | Excellent | Very High | Complex dependencies |

---

## Decision Tree

```
Need dependency injection?
├─ Yes, performance critical?
│  ├─ Yes → Use Lifetime-Parameterized (Strategy 2)
│  └─ No → Continue
├─ Multiple dependencies (3+)?
│  ├─ Yes → Use Constructor + Builder (Strategy 3)
│  └─ No → Continue
└─ Need shared ownership?
   ├─ Yes → Use Arc Trait Objects (Strategy 1)
   └─ No → Use Lifetime-Parameterized (Strategy 2)
```

---

## Best Practices

### 1. Prefer Trait Abstractions
```rust
// ✓ Good: Abstract trait
pub trait Database {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}

// ✗ Bad: Concrete type
pub struct StateReader {
    db: MdbxDatabase,  // Tightly coupled
}
```

### 2. Use Arc for Thread-Safe Sharing
```rust
// ✓ Good: Arc for shared ownership across threads
pub struct Reader {
    db: Arc<dyn Database>,
}

// ✗ Bad: Rc is not thread-safe
pub struct Reader {
    db: Rc<dyn Database>,  // Won't implement Send/Sync
}
```

### 3. Leverage Rust's Type System
```rust
// ✓ Good: Type-level enforcement
impl<D: Database> StateReader<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }
}

// ✗ Bad: Runtime checks
pub struct StateReader {
    db: Option<Arc<dyn Database>>,
}

impl StateReader {
    pub fn get(&self) -> Result<()> {
        let db = self.db.as_ref().ok_or(Error::NoDatabase)?;
        // ...
    }
}
```

---

**Version**: 1.0.0  
**Last Updated**: 2025-12-05
