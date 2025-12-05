# Glob Import Refactoring Guide

## Overview

Glob imports (`use crate::*`) can lead to unclear dependencies, namespace pollution, and reduced code readability. This guide provides a systematic approach to replacing glob imports with explicit, targeted imports.

## What are Glob Imports?

### Bad Example
```rust
// ❌ Glob Import (Avoid!)
use crate::model::*;
use crate::db::*;
```

### Problems with Glob Imports
- Unclear origin of imported types
- Potential name conflicts
- Reduced code readability
- Makes it difficult to track dependencies
- Can lead to unexpected behavior

## Refactoring Strategy

### Step-by-Step Refactoring Process

1. **Identify Glob Imports**
```bash
# Find glob imports in your project
grep -r "use.*::\*" src/
```

2. **Analyze Import Context**
- Understand which types are actually used
- Check which module the types come from

3. **Replace with Explicit Imports**

```rust
// ❌ Before: Glob Import
use crate::model::*;

// ✅ After: Explicit Imports
use crate::model::{
    PlainAccount,
    Address,
    StorageKey,
};
```

### Advanced Importing Techniques

#### 1. Selective Imports
```rust
// Import specific items
use std::collections::{HashMap, BTreeMap};
```

#### 2. Rename on Import
```rust
use std::collections::HashMap as Map;
```

#### 3. Relative Imports
```rust
// In a submodule
use super::{ParentType, AnotherType};
```

## Rust-Specific Import Guidelines

### Module Visibility Imports

```rust
// Public re-exports
pub use self::internal_type::SomeType;

// Crate-level visibility
pub(crate) use self::helper::InternalFunction;
```

### Prelude Pattern
```rust
// In lib.rs or a dedicated prelude module
pub mod prelude {
    pub use crate::model::{
        PlainAccount,
        Address,
    };
}

// Usage in other modules
use crate::prelude::*;  // Controlled, intentional glob import
```

## Automated Refactoring

### Rust Analyzer
- Use rust-analyzer (VSCode extension)
- Automatic import organization
- Quick-fix for unused imports

### Cargo Clippy
```bash
# Run clippy to detect unused imports
cargo clippy
```

## Dependency Tracking

### Import Organization

```rust
// Recommended Import Order
use std::path::PathBuf;  // Standard library
use std::collections::HashMap;

use alloy_primitives::Address;  // External crates
use thiserror::Error;

use crate::model::PlainAccount;  // Local crates
use crate::error::Result;
```

## Common Refactoring Scenarios

### 1. Module-Wide Imports
```rust
// ❌ Before
mod database {
    use crate::*;  // Avoid!
}

// ✅ After
mod database {
    use crate::model::PlainAccount;
    use crate::db::Database;
}
```

### 2. Test Module Imports
```rust
#[cfg(test)]
mod tests {
    // ✅ Explicit, clear imports
    use super::*;  // Only import from immediate parent
    use crate::model::Address;
}
```

## Performance and Compilation

- Explicit imports can improve compile times
- Reduces symbol lookup overhead
- Clearer dependency graph
- More predictable code behavior

## Verification Checklist

- [ ] No remaining `use *` imports
- [ ] All used types explicitly imported
- [ ] Imports organized by source (std, external, local)
- [ ] No unused imports
- [ ] Compilation successful
- [ ] All tests pass

## Advanced Tools

### Import Sorting
- `cargo-sort-ck`: Sort and check imports
- `rustfmt`: Automatically format imports

```bash
# Install import sorting tools
cargo install cargo-sort-ck
```

## Red Flags

🚨 Avoid these import patterns:
- Multiple `use *` in the same file
- Importing entire modules without specifics
- Unclear import sources
- Deeply nested import paths

## Refactoring Example

### Full Refactoring Workflow

```rust
// ❌ Before
use crate::*;

pub fn process_account() {
    let account = PlainAccount::new();
    let db = ErigonDb::open();
}

// ✅ After
use crate::model::PlainAccount;
use crate::db::ErigonDb;

pub fn process_account() {
    let account = PlainAccount::new();
    let db = ErigonDb::open();
}
```

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05