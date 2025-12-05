# Error Type and API Design Review Guidelines

## Error Type Guidelines
- Use centralized error enums for each architectural layer
- Leverage the `thiserror` crate for error definitions
- Always preserve original error sources with the `#[source]` attribute
- Provide human-readable error context for debugging
- Prevent silent swallowing of errors
- Validate error conversions at layer boundaries

### Code Example
```rust
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("MDBX error: {0}")]
    Mdbx(#[from] mdbx::Error),
    #[error("Schema violation: {detail}")]
    SchemaViolation { detail: String },
}

// Centralized error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database: {0}")]
    Db(#[from] DatabaseError),
    #[error("Codec: {0}")]
    Codec(#[from] CodecError),
}
```

## API Design Guidelines
- Keep API surfaces minimal and clear; only expose public endpoints needed for domain logic
- Return `Result<T, Error>` for fallible operations
- Use trait-first interfaces for flexibility and testability
- Apply layered visibility rules (`pub(crate)`, `pub(super)`, etc.) to avoid leaking internal details
- Document every public function with expected errors and error propagation rules

### Code Example
```rust
pub trait AccountApi {
    fn get_account(&self, address: Address) -> Result<Option<PlainAccount>, Error>;
    fn list_accounts(&self) -> Result<Vec<PlainAccount>, Error>;
}

// Layered visibility example
mod db {
    pub(crate) fn db_internal_helper();
    pub struct Database;
}
```

## Review Checklist
- [ ] Centralized error enums defined per layer
- [ ] Layered error conversions implemented
- [ ] All public APIs documented for error behavior
- [ ] APIs use Result<T, Error> for fallible operations
- [ ] Layered visibility correctly applied
- [ ] Trait-first design for flexibility

## References
- See modular-design/error-boundary-guide.md for error conversions
- See architectural-compliance-checklist.md for compliance criteria