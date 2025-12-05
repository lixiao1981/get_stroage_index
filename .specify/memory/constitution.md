<!--
SYNC IMPACT REPORT
==================
Version Change: [Initial Template] → 1.0.0
Rationale: Initial constitution creation based on erc_CONSTITUTION.md source document

Modified Principles:
- NEW: I. Memory Safety First
- NEW: II. Test-Driven Development
- NEW: III. Layered Architecture
- NEW: IV. Performance First
- NEW: V. Fail Fast

Added Sections:
- Non-Negotiables (mandatory constraints)
- Design Philosophy (guiding architectural principles)
- Success Metrics (quantifiable targets)

Removed Sections:
- None (initial creation)

Template Consistency:
- ✅ plan-template.md: Constitution Check section aligns with all 5 principles
- ✅ spec-template.md: Requirements section supports TDD and testing mandates
- ✅ tasks-template.md: Test-first workflow aligns with Principle II
- ⚠️  commands/*.md: No command files exist yet to validate

Follow-up TODOs:
- None (all placeholders filled)

Last Updated: 2025-12-05
-->

# erc-mdbx-index Constitution

## Core Principles

### I. Memory Safety First

Rust's memory safety guarantee is the foundation of this project. All operations crossing FFI boundaries MUST:
- Clearly distinguish "zero-copy reads" from "ownership copies"
- Strictly follow MDBX transaction lifecycle management specifications
- Ensure data references read from transactions do not escape transaction scope
- Adopt "read-then-copy" strategy to avoid dangling pointers

**Rationale**: Working with libmdbx via FFI introduces unsafe boundaries. Memory corruption or use-after-free bugs would be catastrophic in a data indexing tool processing billions of state reads. This principle ensures correctness through Rust's type system.

### II. Test-Driven Development

Facing complex binary data structures, we MUST build a complete "simulation and verification" closed loop:
- Use fixture-driven testing strategy
- Create temporary MDBX environments via tempfile for unit tests
- Follow Red-Green-Refactor iteration cycle
- Decoding logic MUST be independent of database environment for isolated testing

**Rationale**: Erigon's RLP encoding variations and DupSort layout complexities cannot be validated through manual inspection alone. TDD provides fast feedback and regression protection when dealing with low-level binary formats.

### III. Layered Architecture

Code structure follows a clear layered pattern to ensure maintainability and extensibility:
- **Database Abstraction Layer (DAL)**: Encapsulates libmdbx complexity, provides thread-safe environment access
- **Data Model Layer (Model)**: Defines Rust structures specifically for Erigon storage layout
- **Business Logic Layer (Service)**: Implements concrete bucket operations and state reads

**Rationale**: Separating concerns allows independent evolution of database access patterns, data models, and business logic. This modularity is essential for supporting future Erigon versions (Erigon 3, Akula) with different data layouts.

### IV. Performance First

The core value of direct reading lies in extreme performance:
- Target throughput: >1,000,000 Ops/sec (vs JSON-RPC's ~6,000 Ops/sec)
- Adopt "short transaction" mode to avoid database file bloat
- Support batched cursor operations for checkpoint-resumable bulk traversal
- Leverage multi-core CPU via rayon for parallel processing

**Rationale**: This project exists to break the "RPC wall" between blockchain nodes and data users. If we don't achieve orders-of-magnitude speedup, there's no reason to use direct MDBX access over standard RPC.

### V. Fail Fast

Production environments MUST anticipate various failure modes:
- Database version mismatch MUST immediately error to prevent incorrect data output
- Register signal handlers to ensure graceful shutdown and lock release
- Explicitly verify current block height state is "Committed" before reading

**Rationale**: Silent failures in data indexing lead to corrupted analytics and incorrect insights. Fast, loud failures enable quick detection and remediation.

## Non-Negotiables

The following constraints are absolute and cannot be compromised:

1. **NEVER introduce undefined behavior in unsafe code** - All FFI interactions MUST undergo rigorous review
2. **NEVER hold read transactions for extended periods** - Prevents MDBX page reclamation leading to disk bloat
3. **NEVER assume RLP field completeness** - MUST handle Erigon's field omission optimization mechanisms
4. **NEVER ignore version compatibility** - MUST detect and handle Erigon database schema changes

## Design Philosophy

### Zero-Cost Abstractions

Leverage Rust's features to provide high-level APIs without sacrificing low-level performance.

**Application**: Use type-safe wrappers around raw MDBX cursors that compile to identical assembly as direct FFI calls.

### Context-Aware Decoding

Decoders MUST infer default values for missing fields based on RLP list length and remaining content.

**Application**: When decoding Erigon accounts, RLP list length determines which fields were omitted (nonce=0, balance=0 shortcuts).

### Concurrency-Friendly

Environment shared via Arc, supports multi-threaded concurrent read-only transactions.

**Application**: Multiple worker threads can hold independent read transactions against the same MDBX environment without blocking.

### Backward Compatibility

Design with consideration for adapting to future Erigon versions (Erigon 3, Akula) with new data layouts.

**Application**: Abstract storage layout behind traits that can be swapped for different Erigon schema versions.

## Success Metrics

| Metric | Target Value |
|--------|--------------|
| Single-core account read throughput | ≥1,250,000 ops/sec |
| Performance improvement vs JSON-RPC | ≥100x |
| Test coverage | ≥90% |
| Memory safety vulnerabilities | 0 |
| Production reliability | 99.9% uptime |

## Governance

This constitution supersedes all other development practices. Any amendments MUST include:
- Documentation of the change rationale
- Approval from project maintainers
- Migration plan for affected code

All pull requests and code reviews MUST verify compliance with these principles. Complexity that violates principles MUST be explicitly justified with evidence that simpler alternatives are insufficient.

Use `CLAUDE.md` for runtime development guidance and tooling instructions.

**Version**: 1.0.0 | **Ratified**: 2025-12-05 | **Last Amended**: 2025-12-05
