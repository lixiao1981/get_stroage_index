# Architectural Compliance Checklist for erc-mdbx-index

## Purpose

This checklist ensures that new code contributions adhere to the project's modular design principles, maintaining high-quality, maintainable, and performant Rust code.

## 🏗️ Comprehensive Architectural Compliance Checklist

### 1. Layer Placement ✅
**Criteria**: Verify that the new code is placed in the correct architectural layer.

**Checklist**:
- [ ] Code is in the appropriate layer (DAL, Model, Codec, Reader, Utility)
- [ ] Dependencies flow only from higher to lower layers
- [ ] No circular dependencies introduced

**Validation Example**:
```rust
// ✅ Correct: Database-related logic in DAL
pub struct DatabaseConfig {
    path: PathBuf,
    max_dbs: u32,
}

// ❌ Incorrect: Business logic in database layer
pub struct DatabaseConfig {
    fn validate_account_balance(&self) -> Result<()> {
        // Business logic does not belong here!
    }
}
```

### 2. Trait-Based Abstraction 🌐
**Criteria**: Use trait-based designs for flexibility and testability.

**Checklist**:
- [ ] Define behavior through traits before implementation
- [ ] Use generic type parameters for trait bounds
- [ ] Prefer static dispatch when possible
- [ ] Create minimal, focused traits

**Validation Example**:
```rust
// ✅ Correct: Trait-first design
pub trait AccountReader {
    fn get_account(&self, address: Address) -> Result<Option<PlainAccount>>;
}

// Multiple implementations possible
impl AccountReader for MdbxAccountReader { /* ... */ }
impl AccountReader for MockAccountReader { /* ... */ }
```

### 3. Error Handling 🚨
**Criteria**: Implement comprehensive, context-rich error handling.

**Checklist**:
- [ ] Use `thiserror` for error definitions
- [ ] Preserve original error sources
- [ ] Add meaningful context to errors
- [ ] Convert errors at layer boundaries
- [ ] Avoid silent error suppression

**Validation Example**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DatabaseError),

    #[error("State read error: {context}")]
    StateRead {
        context: String,
        #[source]
        source: Box<dyn std::error::Error>,
    },
}
```

### 4. Dependency Management 🔗
**Criteria**: Manage imports and dependencies explicitly.

**Checklist**:
- [ ] No glob imports (`use crate::*`)
- [ ] Explicit, targeted imports
- [ ] Organize imports by source (std, external, local)
- [ ] Minimize dependencies between modules

**Validation Example**:
```rust
// ✅ Correct: Explicit imports
use std::path::PathBuf;
use crate::model::PlainAccount;
use crate::db::Database;

// ❌ Incorrect: Glob imports
use crate::*;
```

### 5. Type Safety 🛡️
**Criteria**: Leverage Rust's type system for safety and clarity.

**Checklist**:
- [ ] Use NewType pattern for domain-specific types
- [ ] Add validation in constructors
- [ ] Prevent invalid state representations
- [ ] Use strong typing to eliminate invalid operations

**Validation Example**:
```rust
#[derive(Debug, Clone, Copy)]
pub struct Address([u8; 20]);

impl Address {
    pub fn from_slice(slice: &[u8]) -> Result<Self, AddressError> {
        if slice.len() != 20 {
            return Err(AddressError::InvalidLength);
        }
        // Validation and safe construction
    }
}
```

### 6. Performance Considerations 🚀
**Criteria**: Maintain zero-cost abstractions and efficient code.

**Checklist**:
- [ ] Prefer static dispatch
- [ ] Minimize dynamic trait objects
- [ ] Use `#[inline]` judiciously
- [ ] Avoid unnecessary allocations
- [ ] Benchmark complex abstractions

**Validation Example**:
```rust
// ✅ Preferred: Static dispatch
fn process_accounts<R: AccountReader>(reader: &R) {
    // Compile-time method resolution
}

// ❌ Avoid: Dynamic dispatch when not necessary
fn process_accounts(reader: &dyn AccountReader) {
    // Runtime method resolution
}
```

### 7. Testability 🧪
**Criteria**: Design for easy testing and mocking.

**Checklist**:
- [ ] Use dependency injection
- [ ] Create mock implementations
- [ ] Design interfaces for easy substitution
- [ ] Write unit tests for each module
- [ ] Test multiple implementation scenarios

**Validation Example**:
```rust
pub struct StateReader<R: AccountReader> {
    reader: R,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_mock_reader() {
        let mock_reader = MockAccountReader::new();
        let state_reader = StateReader::new(mock_reader);
        // Test logic
    }
}
```

### 8. Explicit Configuration 🛠️
**Criteria**: Use builder pattern for complex configurations.

**Checklist**:
- [ ] Implement builder for complex object creation
- [ ] Provide sensible defaults
- [ ] Validate configuration during build
- [ ] Support optional parameters

**Validation Example**:
```rust
pub struct DatabaseConfigBuilder {
    path: Option<PathBuf>,
    max_dbs: u32,
    read_only: bool,
}

impl DatabaseConfigBuilder {
    pub fn build(self) -> Result<DatabaseConfig> {
        Ok(DatabaseConfig {
            path: self.path.ok_or(ConfigError::MissingPath)?,
            max_dbs: self.max_dbs,
            read_only: self.read_only,
        })
    }
}
```

### 9. Error Boundary Conversion 🔀
**Criteria**: Convert errors between layers while preserving context.

**Checklist**:
- [ ] Define a centralized error enum
- [ ] Use `#[from]` for automatic conversions
- [ ] Add context during error translation
- [ ] Preserve original error sources

**Validation Example**:
```rust
impl From<CodecError> for Error {
    fn from(err: CodecError) -> Self {
        Error::StateRead {
            context: "Failed to decode account".to_string(),
            source: Box::new(err),
        }
    }
}
```

### 10. Modularity and Separation of Concerns 📦
**Criteria**: Maintain clear boundaries between modules.

**Checklist**:
- [ ] Each module has a single, well-defined responsibility
- [ ] Minimize public API surface
- [ ] Use visibility modifiers (`pub(crate)`, `pub(super)`)
- [ ] Avoid cross-layer dependencies
- [ ] Design for extension, not modification

**Validation Example**:
```rust
// ✅ Correct: Clear module responsibilities
mod db {
    pub(crate) fn internal_helper();  // Crate-level visibility
    pub struct Database;  // Public interface
}

mod model {
    pub struct PlainAccount;  // Domain model
}
```

## Compliance Scoring

- **0-3 checks passed**: 🔴 Significant architectural issues
- **4-6 checks passed**: 🟡 Needs substantial improvement
- **7-9 checks passed**: 🟢 Good architectural practices
- **10/10 checks passed**: ✨ Architectural Excellence

## Remediation Guidance

If a check fails:
1. Understand the underlying principle
2. Refactor to align with architectural guidelines
3. Consult senior developers if unsure
4. Update documentation if needed

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05