# Architectural Terminology Guide

## Purpose

This document establishes a consistent vocabulary for discussing architectural concepts, design patterns, and system components in the erc-mdbx-index project.

## Layers and Components

### 1. Layer Definitions

#### Database Abstraction Layer (DAL)
- **Definition**: Low-level interaction with persistent storage systems
- **Synonyms**: Persistence Layer, Storage Layer
- **Responsibilities**:
  - Database connection management
  - Transaction handling
  - Low-level data retrieval and storage

#### Model Layer
- **Definition**: Domain-specific data representations and business objects
- **Synonyms**: Domain Model, Data Model
- **Responsibilities**:
  - Define core data structures
  - Maintain business logic constraints
  - Provide type-safe representations of domain concepts

#### Codec Layer
- **Definition**: Data encoding and decoding mechanisms
- **Synonyms**: Serialization Layer, Transformation Layer
- **Responsibilities**:
  - Convert between binary and structured representations
  - Handle RLP (Recursive Length Prefix) encoding/decoding
  - Provide serialization utilities

#### Reader Layer
- **Definition**: High-level business logic for data retrieval and processing
- **Synonyms**: Service Layer, Business Logic Layer
- **Responsibilities**:
  - Coordinate between database and model layers
  - Implement complex state reading scenarios
  - Provide domain-specific reading operations

#### Utility Layer
- **Definition**: Cross-cutting helper functions and generic utilities
- **Synonyms**: Support Layer, Infrastructure Layer
- **Responsibilities**:
  - Configuration management
  - Signal processing
  - Generic, reusable helper functions

## Design Patterns

### 1. Trait-First Design
- **Definition**: Interface-driven development using Rust traits
- **Key Characteristics**:
  - Defines abstract interfaces
  - Enables dependency injection
  - Supports runtime polymorphism
- **Antonyms**: Concrete-first design, Implementation-driven development

### 2. NewType Pattern
- **Definition**: Wrapper type for providing type safety and semantic meaning
- **Key Characteristics**:
  - Prevents incorrect type usage
  - Adds domain-specific semantics
  - Enables type-level validation
- **Antonyms**: Primitive obsession, Untyped representations

### 3. Builder Pattern
- **Definition**: Flexible object construction with step-by-step configuration
- **Key Characteristics**:
  - Separates construction from representation
  - Allows complex object initialization
  - Supports optional parameters
- **Antonyms**: Monolithic constructors, All-args constructors

### 4. Dependency Injection
- **Definition**: Providing dependencies from external sources
- **Key Characteristics**:
  - Reduces tight coupling
  - Enables easier testing
  - Increases module flexibility
- **Antonyms**: Hard-coded dependencies, Tight coupling

## Error Handling Terminology

### Error Categories
- **Domain Errors**: Errors specific to business logic
- **Infrastructure Errors**: Errors related to system resources
- **Transient Errors**: Temporary, potentially recoverable errors
- **Fatal Errors**: Unrecoverable errors requiring system intervention

### Error Handling Strategies
- **Error Propagation**: Passing errors up the call stack
- **Error Translation**: Converting lower-level errors to domain-specific errors
- **Error Wrapping**: Adding context to existing errors

## Dependency Management

### Dependency Types
- **Static Dependency**: Compile-time resolved dependencies
- **Dynamic Dependency**: Runtime-resolved dependencies
- **Optional Dependency**: Configurable, non-essential dependencies

### Dependency Rules
- **Unidirectional**: Dependencies flow from higher to lower layers
- **Minimal Surface**: Expose only essential interfaces
- **Explicit**: Clear, intentional dependency declarations

## Coding Conventions

### Naming Conventions
- **Types/Traits**: PascalCase
- **Functions/Variables**: snake_case
- **Constants**: SCREAMING_SNAKE_CASE
- **Prefixes**:
  - `I` for interfaces/traits (optional)
  - `Abstract` for abstract base types
  - `Base` for base implementations

### Documentation Conventions
- **One-line Summary**: Concise description of purpose
- **Detailed Description**: Explain why, not just what
- **Examples**: Provide usage demonstrations
- **Edge Cases**: Document known limitations

## Version Control Terminology

- **Feature Branch**: Isolated development branch for specific feature
- **Main Branch**: Primary integration branch
- **WIP (Work in Progress)**: Incomplete, ongoing work
- **POC (Proof of Concept)**: Experimental implementation

## Performance Terminology

- **Zero-Cost Abstraction**: Compile-time resolved abstractions with no runtime overhead
- **Static Dispatch**: Compile-time method resolution
- **Dynamic Dispatch**: Runtime method resolution
- **Inline Expansion**: Compiler-level function inlining for performance

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05

## Usage Guidelines

1. Use these terms consistently across documentation, code, and communication
2. When in doubt, refer to this guide
3. Update the guide as project evolves
4. Discuss and align on new terms as a team