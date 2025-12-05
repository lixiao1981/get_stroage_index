# Builder Pattern: Advanced Guide for Rust

## Overview

The Builder pattern is a powerful design technique that provides a flexible approach to object construction, particularly useful in complex configuration scenarios.

## What is the Builder Pattern?

### Core Concept
- Separate object construction from its representation
- Allow step-by-step configuration of complex objects
- Enable creation of different object variants
- Improve code readability and maintainability

## Basic Builder Implementation

```rust
pub struct DatabaseConfig {
    path: PathBuf,
    max_dbs: u32,
    max_readers: u32,
    read_only: bool,
}

pub struct DatabaseConfigBuilder {
    path: Option<PathBuf>,
    max_dbs: u32,
    max_readers: u32,
    read_only: bool,
}

impl DatabaseConfigBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            max_dbs: 10,      // Sensible default
            max_readers: 126, // Typical MDBX default
            read_only: true,  // Safe default
        }
    }

    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn max_dbs(mut self, count: u32) -> Self {
        self.max_dbs = count;
        self
    }

    pub fn max_readers(mut self, count: u32) -> Self {
        self.max_readers = count;
        self
    }

    pub fn writable(mut self) -> Self {
        self.read_only = false;
        self
    }

    pub fn build(self) -> Result<DatabaseConfig, ConfigError> {
        Ok(DatabaseConfig {
            path: self.path.ok_or(ConfigError::MissingPath)?,
            max_dbs: self.max_dbs,
            max_readers: self.max_readers,
            read_only: self.read_only,
        })
    }
}

// Error handling for configuration
#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error("Database path is required")]
    MissingPath,
}
```

## Advanced Builder Techniques

### 1. Generic Builder with Type Parameters

```rust
pub struct GenericBuilder<T> {
    inner: T,
}

impl<T> GenericBuilder<T> {
    pub fn new(initial: T) -> Self {
        Self { inner: initial }
    }

    pub fn transform<R>(self, transformer: impl FnOnce(T) -> R) -> GenericBuilder<R> {
        GenericBuilder { inner: transformer(self.inner) }
    }

    pub fn build(self) -> T {
        self.inner
    }
}

// Usage example
let config = GenericBuilder::new(HashMap::new())
    .transform(|mut map| {
        map.insert("key", "value");
        map
    })
    .build();
```

### 2. Validation and Constraints

```rust
pub struct AccountBuilder {
    address: Option<Address>,
    nonce: u64,
    balance: U256,
}

impl AccountBuilder {
    pub fn new() -> Self {
        Self {
            address: None,
            nonce: 0,
            balance: U256::zero(),
        }
    }

    pub fn address(mut self, addr: Address) -> Self {
        self.address = Some(addr);
        self
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn balance(mut self, balance: U256) -> Self {
        // Validation during construction
        if balance > MAX_ALLOWED_BALANCE {
            panic!("Balance exceeds maximum allowed");
        }
        self.balance = balance;
        self
    }

    pub fn build(self) -> Result<PlainAccount, BuildError> {
        Ok(PlainAccount {
            address: self.address.ok_or(BuildError::MissingAddress)?,
            nonce: self.nonce,
            balance: self.balance,
            storage_root: None,
            code_hash: None,
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum BuildError {
    #[error("Address is required")]
    MissingAddress,
}
```

### 3. Conditional Configuration

```rust
pub struct OptimizedReaderBuilder<D: Database> {
    db: Option<D>,
    cache_size: usize,
    read_ahead: bool,
}

impl<D: Database> OptimizedReaderBuilder<D> {
    pub fn new() -> Self {
        Self {
            db: None,
            cache_size: 1024, // Default cache
            read_ahead: true, // Optimize for sequential reads
        }
    }

    pub fn database(mut self, db: D) -> Self {
        self.db = Some(db);
        self
    }

    // Conditional configuration
    pub fn configure_for_large_state(mut self) -> Self {
        self.cache_size = 4096;
        self.read_ahead = true;
        self
    }

    pub fn build(self) -> Result<StateReader<D>, BuildError> {
        Ok(StateReader::new(
            self.db.ok_or(BuildError::NoDatabaseProvided)?,
            self.cache_size,
            self.read_ahead
        ))
    }
}
```

## Performance Considerations

### Zero-Cost Abstractions
- Compile-time method resolution
- No runtime overhead
- Monomorphization by Rust compiler

### Memory Efficiency
- No heap allocations during method chaining
- Moves instead of clones
- Optimization opportunities

## Testing Strategies

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_builder() {
        let config = DatabaseConfigBuilder::new()
            .path("/tmp/test".into())
            .max_dbs(20)
            .writable()
            .build()
            .unwrap();

        assert_eq!(config.max_dbs, 20);
        assert!(!config.read_only);
    }

    #[test]
    #[should_panic(expected = "Balance exceeds maximum")]
    fn test_account_builder_balance_validation() {
        AccountBuilder::new()
            .balance(U256::MAX)  // Should panic
            .build()
            .unwrap();
    }
}
```

## Best Practices

### Do's
- Use for complex object construction
- Implement validation in `build()`
- Provide sensible defaults
- Use method chaining
- Leverage type system for constraints

### Don'ts
- Avoid overly complex builders
- Don't skip validation
- Prevent impossible states
- Minimize builder-specific logic

## Common Pitfalls

- Creating builders for simple structs
- Not implementing proper validation
- Forgetting to mark methods as consuming (`self`)
- Overcomplicating the builder interface

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05