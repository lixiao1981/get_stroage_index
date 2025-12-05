# Error Boundary Conversion Guide

## Overview

Error handling is critical in maintaining a robust, maintainable codebase. This guide demonstrates how to effectively manage and convert errors across different layers of the erc-mdbx-index project.

## Error Handling Principles

### Key Objectives
- Preserve error context
- Minimize information loss
- Provide meaningful error messages
- Enable easy debugging
- Support graceful error recovery

## Centralized Error Handling

### Unified Error Enum

```rust
// error.rs - Central error definition
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DatabaseError),

    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),

    #[error("Model validation error: {0}")]
    Model(#[from] ModelError),

    #[error("State read error: {context}")]
    StateRead {
        context: String,
        #[source]
        source: Box<dyn std::error::Error>,
    },
}
```

## Layer-Specific Error Handling

### 1. Database Layer Errors

```rust
// db/error.rs
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Transaction begin failed: {0}")]
    TransactionBegin(#[source] libmdbx::Error),

    #[error("Key not found in bucket: {bucket}")]
    KeyNotFound { bucket: String },

    #[error("Database environment error: {0}")]
    Environment(#[source] libmdbx::Error),
}
```

### 2. Codec Layer Error Conversion

```rust
// codec/error.rs
#[derive(Debug, Error)]
pub enum CodecError {
    #[error("RLP decode failed: {0}")]
    RlpDecode(#[from] alloy_rlp::Error),

    #[error("Invalid account format: expected {expected} bytes, got {actual}")]
    InvalidFormat { expected: usize, actual: usize },
}

// Error conversion example
impl From<CodecError> for Error {
    fn from(err: CodecError) -> Self {
        match err {
            CodecError::RlpDecode(inner) => Error::StateRead {
                context: "Failed to decode account RLP".to_string(),
                source: Box::new(inner),
            },
            CodecError::InvalidFormat { expected, actual } => Error::StateRead {
                context: format!("Invalid account format (expected {}, got {})", expected, actual),
                source: Box::new(err),
            }
        }
    }
}
```

### 3. Model Layer Error Handling

```rust
// model/error.rs
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Balance overflow")]
    BalanceOverflow,

    #[error("Invalid address length")]
    InvalidAddressLength,

    #[error("Account validation failed: {0}")]
    ValidationFailed(String),
}
```

### 4. Reader Layer Error Conversion

```rust
// reader/state_reader.rs
impl<D: Database, C: AccountCodec> StateReader<D, C> {
    pub fn get_account(&self, address: Address) -> Result<Option<PlainAccount>> {
        let raw_bytes = self.db.get(address.as_bytes())
            .map_err(|db_err| Error::StateRead {
                context: format!("Failed to retrieve account for address {}", address),
                source: Box::new(db_err),
            })?;

        raw_bytes.map(|bytes| {
            self.codec.decode(&bytes)
                .map_err(|codec_err| Error::StateRead {
                    context: format!("Failed to decode account for address {}", address),
                    source: Box::new(codec_err),
                })
        }).transpose()
    }
}
```

## Error Conversion Patterns

### 1. Source Preservation
```rust
// Preserve original error source
pub fn convert_error(err: SpecificError) -> GeneralError {
    GeneralError::Detailed {
        context: "Operation failed".to_string(),
        source: Box::new(err),
    }
}
```

### 2. Context Enrichment
```rust
// Add meaningful context during error conversion
pub fn enrich_error(err: LowLevelError) -> HighLevelError {
    HighLevelError::Context(
        "Additional diagnostic information".to_string(),
        Box::new(err)
    )
}
```

## Advanced Error Handling Techniques

### Error Chaining
```rust
pub fn complex_operation() -> Result<(), Error> {
    let intermediate_result = step_one()?;
    let final_result = step_two(intermediate_result)
        .map_err(|err| Error::StateRead {
            context: "Multi-step operation failed".to_string(),
            source: Box::new(err),
        })?;

    Ok(())
}
```

## Error Handling Best Practices

### Do's
- Use `thiserror` for comprehensive error definitions
- Preserve error sources
- Add meaningful context
- Use `#[from]` for automatic conversions
- Create layer-specific error types

### Don'ts
- Suppress errors silently
- Lose original error information
- Create overly generic error types
- Catch all errors without specific handling

## Debugging and Logging

```rust
// Enhanced error logging
pub fn log_error(err: &Error) {
    tracing::error!(
        error = %err,
        source = ?err.source(),
        "Detailed error occurred"
    );
}
```

## Verification Checklist

- [ ] Errors have meaningful messages
- [ ] Original error sources are preserved
- [ ] Errors provide diagnostic context
- [ ] Layer boundaries have clear error conversion
- [ ] No silent error suppression

## Performance Considerations

- Error creation is zero-cost
- Dynamic error trait for source preservation
- Minimal overhead with `thiserror`

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05