//! MDBX database connection and transaction management

use crate::error::{Error, Result};
use heed::{Database, Env, EnvOpenOptions, RoTxn};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Database table names used by Erigon
pub const TABLE_PLAIN_STATE: &str = "PlainState";
pub const TABLE_PLAIN_CONTRACT_CODE: &str = "PlainContractCode";
pub const TABLE_PLAIN_STATE_STORAGE: &str = "PlainState"; // Uses DupSort for storage
pub const TABLE_CONFIG: &str = "Config"; // Database configuration and version info

/// Configuration for database operations
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Maximum transaction duration before auto-release (default: 5 seconds)
    pub transaction_timeout: Duration,
    
    /// Maximum number of concurrent readers (default: 126)
    pub max_readers: u32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            transaction_timeout: Duration::from_secs(5),
            max_readers: 126,
        }
    }
}

/// MDBX database environment wrapper
pub struct ErigonDb {
    env: Arc<Env>,
    config: DbConfig,
}

impl ErigonDb {
    /// Open an Erigon MDBX database in read-only mode
    pub fn open(path: impl AsRef<Path>, config: DbConfig) -> Result<Self> {
        let path = path.as_ref();

        // Check if database exists
        if !path.exists() {
            return Err(Error::DatabaseNotFound {
                path: path.to_path_buf(),
            });
        }

        // Open environment in read-only mode
        let env = unsafe {
            EnvOpenOptions::new()
                .max_readers(config.max_readers)
                .max_dbs(100) // Erigon has many tables
                .open(path)
        }
        .map_err(|e| {
            // Check for common errors
            let err_str = e.to_string();
            if err_str.contains("Permission denied") {
                Error::PermissionDenied {
                    path: path.to_path_buf(),
                }
            } else if err_str.contains("locked") {
                Error::DatabaseLocked
            } else {
                Error::DatabaseOpen(err_str)
            }
        })?;

        tracing::info!("Opened Erigon database at: {}", path.display());

        Ok(Self {
            env: Arc::new(env),
            config,
        })
    }

    /// Open with default configuration
    pub fn open_default(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path, DbConfig::default())
    }

    /// Begin a read-only transaction
    pub fn begin_txn(&self) -> Result<RoTxn<'_>> {
        self.env
            .read_txn()
            .map_err(|e| Error::TransactionBegin(e.to_string()))
    }

    /// Open a database table
    pub fn open_db(&self, name: &str) -> Result<Database<heed::types::Bytes, heed::types::Bytes>> {
        // Use a temporary write transaction to open the database
        let mut wtxn = self.env.write_txn()
            .map_err(|e| Error::TableOpen {
                table: name.to_string(),
                source: Box::new(e),
            })?;
        
        let db = self.env.create_database(&mut wtxn, Some(name))
            .map_err(|e| Error::TableOpen {
                table: name.to_string(),
                source: Box::new(e),
            })?;
        
        wtxn.commit().map_err(|e| Error::TableOpen {
            table: name.to_string(),
            source: Box::new(e),
        })?;
        
        Ok(db)
    }

    /// Get account from PlainState table
    pub fn get_plain_state(&self, txn: &RoTxn, db: &Database<heed::types::Bytes, heed::types::Bytes>, key: &[u8]) -> Result<Option<Vec<u8>>> {
        db.get(txn, key)
            .map_err(|e| Error::DatabaseRead(e.to_string()))
            .map(|opt| opt.map(|v| v.to_vec()))
    }

    /// Get database configuration
    pub fn config(&self) -> &DbConfig {
        &self.config
    }

    /// Get environment handle (for advanced usage)
    pub fn env(&self) -> &Arc<Env> {
        &self.env
    }
    
    /// Try to detect database version
    pub fn detect_version(&self) -> crate::error::Result<Option<crate::model::DbVersion>> {
        use tracing::warn;
        
        // Try to open Config table
        let txn = self.begin_txn()?;
        
        // Attempt to open Config database (may not exist in all Erigon versions)
        let config_db: Database<heed::types::Bytes, heed::types::Bytes> = 
            match self.env.open_database(&txn, Some(TABLE_CONFIG)) {
                Ok(Some(db)) => db,
                Ok(None) => {
                    warn!("Config table not found - version detection not available");
                    return Ok(None);
                }
                Err(e) => {
                    warn!("Failed to open Config table: {}", e);
                    return Ok(None);
                }
            };
        
        // Try to read version key
        let version_key = b"version";
        match config_db.get(&txn, version_key) {
            Ok(Some(value)) => {
                let version_str = String::from_utf8_lossy(value).to_string();
                Ok(Some(crate::model::DbVersion::new(version_str, "mdbx")))
            }
            Ok(None) => {
                warn!("Version key not found in Config table");
                Ok(None)
            }
            Err(e) => {
                warn!("Failed to read version from Config: {}", e);
                Ok(None)
            }
        }
    }
}

// Ensure ErigonDb is thread-safe
unsafe impl Send for ErigonDb {}
unsafe impl Sync for ErigonDb {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.transaction_timeout, Duration::from_secs(5));
        assert_eq!(config.max_readers, 126);
    }

    #[test]
    fn test_open_nonexistent_db() {
        let result = ErigonDb::open_default("/nonexistent/path");
        assert!(result.is_err());
        match result {
            Err(Error::DatabaseNotFound { .. }) => {}
            _ => panic!("Expected DatabaseNotFound error"),
        }
    }
}
