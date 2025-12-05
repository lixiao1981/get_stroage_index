# NewType Pattern: Type Safety and Domain Semantics in Rust

## Overview

The NewType pattern is a powerful Rust idiom that provides compile-time type safety, prevents accidental misuse, and adds semantic meaning to primitive types.

## What is the NewType Pattern?

### Core Concept
- Wrap a primitive or existing type in a new, distinct type
- Zero-cost abstraction
- Prevent invalid operations
- Add domain-specific behavior
- Improve type safety

## Basic NewType Implementation

```rust
// Primitive Wrapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(pub [u8; 20]);

impl Address {
    // Validation during construction
    pub fn from_slice(slice: &[u8]) -> Result<Self, AddressError> {
        if slice.len() != 20 {
            return Err(AddressError::InvalidLength);
        }

        let mut addr = [0u8; 20];
        addr.copy_from_slice(&slice[..20]);
        Ok(Self(addr))
    }

    // Meaningful domain-specific methods
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }

    pub fn as_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

// Custom error for validation
#[derive(Debug, thiserror::Error)]
enum AddressError {
    #[error("Invalid Ethereum address length")]
    InvalidLength,
}
```

## Advanced NewType Techniques

### 1. Type-Level Validation

```rust
// Constrained NewType
#[derive(Debug, Clone, PartialEq)]
pub struct PositiveBalance(U256);

impl PositiveBalance {
    pub fn new(value: U256) -> Result<Self, BalanceError> {
        if value > U256::zero() {
            Ok(Self(value))
        } else {
            Err(BalanceError::NonPositive)
        }
    }

    pub fn value(&self) -> U256 {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
enum BalanceError {
    #[error("Balance must be positive")]
    NonPositive,
}
```

### 2. Wrapper with Custom Behavior

```rust
// Enhanced Semantic Wrapper
#[derive(Debug, Clone)]
pub struct StorageKey {
    inner: H256,
    namespace: Option<String>,
}

impl StorageKey {
    pub fn new(key: H256) -> Self {
        Self {
            inner: key,
            namespace: None,
        }
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub fn full_key(&self) -> Vec<u8> {
        match &self.namespace {
            Some(ns) => [ns.as_bytes(), self.inner.as_bytes()].concat(),
            None => self.inner.as_bytes().to_vec(),
        }
    }
}
```

### 3. Newtype with Trait Implementations

```rust
// Custom traits for NewTypes
#[derive(Debug, Clone, Copy)]
pub struct Nonce(pub u64);

impl Add for Nonce {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Nonce(self.0.saturating_add(rhs.0))
    }
}

impl PartialOrd for Nonce {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}
```

## Performance Considerations

### Zero-Cost Abstractions
- No runtime overhead
- Compile-time type checking
- No additional memory allocation
- Optimizes to same assembly as primitive types

```rust
// Compiler generates identical assembly
fn compare_addresses(a: Address, b: Address) -> bool {
    a == b  // Same as comparing raw byte arrays
}
```

## Conversion Strategies

```rust
impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }
}

impl TryFrom<Vec<u8>> for Address {
    type Error = AddressError;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Address::from_slice(&vec)
    }
}
```

## Testing NewTypes

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let valid_addr = Address::from_slice(&[0; 20]).unwrap();
        assert!(valid_addr.is_zero());
    }

    #[test]
    fn test_address_validation() {
        let invalid_addr = Address::from_slice(&[0; 10]);
        assert!(invalid_addr.is_err());
    }

    #[test]
    fn test_positive_balance() {
        let balance = PositiveBalance::new(U256::from(100)).unwrap();
        assert_eq!(balance.value(), U256::from(100));

        let zero_balance = PositiveBalance::new(U256::zero());
        assert!(zero_balance.is_err());
    }
}
```

## Best Practices

### Do's
- Use for domain-specific primitive types
- Implement validation in constructors
- Add meaningful methods
- Provide conversion traits
- Use for strong typing

### Don'ts
- Avoid wrapping types without clear purpose
- Don't bypass validation
- Don't create overly complex wrappers
- Avoid significant performance overhead

## Common Use Cases

- Ethereum addresses
- Account nonces
- Block numbers
- Storage keys
- Monetary amounts
- Cryptographic keys

## Potential Drawbacks

- Slight increase in code complexity
- Requires manual trait implementations
- Can lead to verbose code if overused

## Integration Patterns

### Serialization
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AccountState {
    address: Address,
    nonce: Nonce,
    balance: PositiveBalance,
}
```

## Version

**Current Version**: 1.0.0
**Last Updated**: 2025-12-05