//! High-performance MDBX PlainState reader for Erigon databases
//!
//! This library provides direct read-only access to Erigon's MDBX database,
//! specifically for reading account state from the PlainState bucket.
//!
//! # Architecture
//!
//! The library follows a layered architecture:
//! - `db`: Database connection and transaction management
//! - `model`: Domain entities (PlainAccount, StorageSlot)
//! - `codec`: RLP encoding/decoding
//! - `reader`: High-level read operations
//! - `error`: Error types and handling

// Layer: Database access
pub mod db;

// Layer: Domain models
pub mod model;

// Layer: Encoding/decoding
pub mod codec;

// Layer: High-level readers
pub mod reader;

// Layer: Error handling
pub mod error;

// Re-export commonly used types
pub use error::{Error, Result};
pub use model::{DbVersion, PlainAccount, StorageSlot};
pub use reader::StateReader;
