# Mock Pull Request Examples with Annotations

## Purpose
Demonstrate model code reviews for modular design best practices. Each example illustrates annotated review comments for code structure, errors, API compliance, and layer rules.

---
## Example 1: Correct Layer Placement
```rust
// src/dal/database.rs
pub struct DatabaseConfig {
    path: PathBuf,
    max_dbs: u32,
}
// Reviewer: ✅ Layer placement correct, no business logic in DAL
```

---
## Example 2: Error Handling Annotation
```rust
// src/model/account.rs
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Invalid account address")]
    InvalidAddress,
}
// Reviewer: ✅ Error type centralized and context provided
```

---
## Example 3: Violation - Glob Import
```rust
// src/codec/rlp.rs
use crate::*; // 🚫 Glob import
// Reviewer: ❌ Violates explicit import compliance (see checklist)
```

---
## Example 4: API Visibility Annotation
```rust
mod db {
    pub(crate) fn helper_fn() {}
    pub struct Database;
}
// Reviewer: ✅ Visibility scoped appropriately for modularity
```

---
## Example 5: Error Boundary Conversion Annotation
```rust
impl From<CodecError> for Error {
    fn from(err: CodecError) -> Self {
        Error::StateRead {
            context: "Failed to decode account".to_string(),
            source: Box::new(err),
        }
    }
}
// Reviewer: ✅ Error conversion preserves original context
```

---
## Reviewer Guidelines
- Always annotate layer placement, error handling, API design, imports, and compliance.
- Use architectural-compliance-checklist.md for validation.
- Require refactoring for any violations noted.

## References
- See architectural-compliance-checklist.md for all annotated review criteria.