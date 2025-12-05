# Rust Modular Design Guidelines for erc-mdbx-index

## Introduction

This document provides comprehensive guidelines for modular design in the erc-mdbx-index project, focusing on architectural principles, design patterns, and best practices for Rust development.

## Table of Contents

1. [Layer Definitions](modular-design/layer-definitions.md)
2. [Design Patterns](modular-design/design-patterns.md)
3. [Refactoring Guide](modular-design/refactoring-guide.md)
4. [Architectural Rules](modular-design/architectural-rules.md)

## Core Design Principles

### 1. Single Responsibility
- Each module focuses on a single, well-defined functionality
- Clear separation of concerns across layers
- Minimal, focused public APIs

### 2. Explicit Dependencies
- Use explicit `use` statements
- Avoid glob imports (`use xxx::*`)
- Define dependencies through traits
- Follow strict layer dependency rules

### 3. Interface Segregation
- Use traits to define abstract interfaces
- Separate interface from implementation
- Support dependency injection for testing
- Minimize public surface area

### 4. Error Boundaries
- Define module-specific error types
- Convert errors at module boundaries
- Use `thiserror` for comprehensive error handling
- Provide context-rich error messages

### 5. Testability
- Design modules to support unit testing
- Use dependency injection
- Create mock implementations
- Provide test fixtures and helpers

## Key Artifacts

- Detailed layer definitions
- Comprehensive design pattern catalog
- Refactoring guidelines
- Architectural compliance checklist

## Contribution Guidelines

1. Follow modular design principles
2. Maintain clear, minimal public APIs
3. Write comprehensive unit tests
4. Document architectural decisions
5. Use traits for abstractions
6. Minimize runtime overhead

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05