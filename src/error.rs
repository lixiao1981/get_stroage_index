//! Error types for the MDBX PlainState reader

use std::path::PathBuf;

/// Result type alias using our custom Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading from the Erigon database
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Database file not found or inaccessible
    #[error("Database not found at path: {path}")]
    DatabaseNotFound { path: PathBuf },

    /// Permission denied when opening database
    #[error("Permission denied accessing database at: {path}")]
    PermissionDenied { path: PathBuf },

    /// Database is locked by another process (e.g., Erigon running in write mode)
    #[error("Database is locked by another process")]
    DatabaseLocked,

    /// Failed to open database environment
    #[error("Failed to open database: {0}")]
    DatabaseOpen(String),

    /// Failed to begin transaction
    #[error("Failed to begin transaction: {0}")]
    TransactionBegin(String),

    /// Failed to open database table/bucket
    #[error("Failed to open table '{table}': {source}")]
    TableOpen {
        table: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Failed to read data from database
    #[error("Failed to read from database: {0}")]
    DatabaseRead(String),

    /// RLP decoding error
    #[error("RLP decoding failed for address {address}: {source}")]
    RlpDecode {
        address: String,
        source: alloy_rlp::Error,
    },

    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidData(String),

    /// Database version mismatch
    #[error("Incompatible database version: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    /// Database version information missing
    #[error("Database version information not found")]
    VersionMissing,

    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Create a new RLP decode error
    pub fn rlp_decode(address: impl Into<String>, source: alloy_rlp::Error) -> Self {
        Self::RlpDecode {
            address: address.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_database_not_found_error() {
        let err = Error::DatabaseNotFound {
            path: PathBuf::from("/nonexistent/path"),
        };
        assert!(err.to_string().contains("/nonexistent/path"));
    }

    #[test]
    fn test_rlp_decode_error() {
        let err = Error::rlp_decode(
            "0x1234567890123456789012345678901234567890",
            alloy_rlp::Error::UnexpectedString,
        );
        assert!(err.to_string().contains("0x1234567890123456789012345678901234567890"));
    }

    #[test]
    fn test_version_mismatch_error() {
        let err = Error::VersionMismatch {
            expected: "2.0".to_string(),
            found: "3.0".to_string(),
        };
        assert!(err.to_string().contains("2.0"));
        assert!(err.to_string().contains("3.0"));
    }
}
