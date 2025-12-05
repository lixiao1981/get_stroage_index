# Layer Definitions

## Overview

Our architecture follows a clear, hierarchical layer design to ensure modularity, testability, and separation of concerns.

## Layers

### Database Abstraction Layer (DAL)

**Responsibilities**:
- Encapsulate MDBX database interactions
- Manage transactions and cursors
- Provide thread-safe database access
- Abstract low-level database operations

**Allowed Dependencies**: None
**Dependencies From**: All layers

### Model Layer

**Responsibilities**:
- Define domain-specific data structures
- Implement type-safe representations
- Provide encoding/decoding traits
- Maintain domain invariants

**Allowed Dependencies**:
- None
**Dependencies From**:
- Codec Layer
- Reader Layer

### Codec Layer

**Responsibilities**:
- Handle data encoding/decoding
- Implement RLP parsing
- Convert between binary and structured representations
- Provide serialization/deserialization utilities

**Allowed Dependencies**:
- Model Layer
**Dependencies From**:
- Reader Layer

### Reader Layer

**Responsibilities**:
- Implement business logic for data retrieval
- Coordinate between database and model layers
- Provide high-level, domain-specific reading operations
- Manage complex state reading scenarios

**Allowed Dependencies**:
- Database Layer
- Model Layer
- Codec Layer
**Dependencies From**:
- None (top layer)

### Utility Layer

**Responsibilities**:
- Provide cross-cutting helper functions
- Implement configuration management
- Handle signal processing
- Offer generic, reusable utilities

**Allowed Dependencies**: None
**Dependencies From**: All layers

## Dependency Rules

1. Lower layers cannot depend on higher layers
2. Each layer has a minimal, well-defined interface
3. Dependency injection supported via traits
4. Error conversion at layer boundaries

## Testing Strategy

- Each layer must be independently testable
- Mock implementations for dependency injection
- Comprehensive unit test coverage
- Integration tests verifying layer interactions