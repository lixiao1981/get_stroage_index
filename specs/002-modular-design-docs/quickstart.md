# Rust Modular Design Quickstart

## Layer Selection Quick Guide

1. **Database Abstraction Layer (DAL)**
   - Handles direct database interactions
   - Encapsulates MDBX complexity
   - Provides thread-safe environment access

2. **Data Model Layer**
   - Defines Rust structures for Erigon storage
   - Implements domain-specific type conversions
   - No direct database or business logic

3. **Service Layer**
   - Implements concrete bucket operations
   - Coordinates between data model and database
   - Contains primary business logic

## Design Pattern Cheat Sheet

### When to Use Each Pattern

- **Builder**: Configuration-heavy structs
- **NewType**: Strong type safety
- **Trait-First**: Dependency injection, testability

## Architecture Compliance Checklist

1. ✅ Clear layer responsibilities
2. ✅ Explicit imports
3. ✅ No circular dependencies
4. ✅ Minimal public API surface
5. ✅ Proper error handling at boundaries
6. ✅ Zero-cost abstractions
7. ✅ Testable module structure
8. ✅ Performance-conscious design
9. ✅ Memory safety preserved
10. ✅ Rust idioms followed

## Quick Reference

- **Read docs**: `less RUST_MODULAR_DESIGN.md`
- **Validate architecture**: `cargo clippy`
- **Run tests**: `cargo test`