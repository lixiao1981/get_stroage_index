//! High-level state reader for querying account and storage data

use crate::codec::decode_account;
use crate::db::{ErigonDb, TABLE_PLAIN_STATE};
use crate::error::Result;
use crate::model::PlainAccount;
use alloy_primitives::{Address, U256};
use heed::Database;
use tracing::{debug, error, warn};

/// High-level reader for Erigon PlainState data
pub struct StateReader {
    db: ErigonDb,
    plain_state_db: Database<heed::types::Bytes, heed::types::Bytes>,
    storage_db: Database<heed::types::Bytes, heed::types::Bytes>,
}

impl StateReader {
    /// Create a new StateReader
    pub fn new(db: ErigonDb) -> Result<Self> {
        let plain_state_db = db.open_db(TABLE_PLAIN_STATE)?;
        // Storage uses composite keys in PlainState table with DupSort
        let storage_db = db.open_db("PlainState")?;
        Ok(Self { db, plain_state_db, storage_db })
    }

    /// Open a database and create a StateReader
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let db = ErigonDb::open_default(path)?;
        Self::new(db)
    }
    
    /// Get database version information
    pub fn get_version(&self) -> Result<Option<crate::model::DbVersion>> {
        self.db.detect_version()
    }

    /// Get account state by address
    ///
    /// Returns None if the account doesn't exist
    pub fn get_account(&self, address: Address) -> Result<Option<PlainAccount>> {
        // Begin read transaction
        let txn = self.db.begin_txn()?;
        
        // Query by address (20 bytes)
        let key = address.as_slice();
        
        match self.db.get_plain_state(&txn, &self.plain_state_db, key)? {
            Some(data) => {
                // Decode RLP data
                let account = decode_account(&data)
                    .map_err(|e| {
                        error!("Failed to decode account at {}: {}", address, e);
                        e
                    })?;
                
                debug!(
                    "Retrieved account {}: nonce={}, balance={}, is_contract={}",
                    address, account.nonce, account.balance, account.is_contract()
                );
                
                Ok(Some(account))
            }
            None => {
                debug!("Account {} not found", address);
                Ok(None)
            }
        }
    }

    /// Check if an account exists
    pub fn account_exists(&self, address: Address) -> Result<bool> {
        self.get_account(address).map(|acc| acc.is_some())
    }

    /// Get account balance
    pub fn get_balance(&self, address: Address) -> Result<Option<U256>> {
        self.get_account(address)
            .map(|acc| acc.map(|a| a.balance))
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: Address) -> Result<Option<u64>> {
        self.get_account(address).map(|acc| acc.map(|a| a.nonce))
    }

    /// Check if address is a contract
    pub fn is_contract(&self, address: Address) -> Result<bool> {
        self.get_account(address)
            .map(|acc| acc.map(|a| a.is_contract()).unwrap_or(false))
    }

    /// Iterate over all accounts in batches
    ///
    /// Returns an iterator that yields batches of (Address, PlainAccount) pairs
    pub fn iter_accounts(&self, batch_size: usize) -> Result<AccountIterator<'_>> {
        AccountIterator::new(&self.db, &self.plain_state_db, batch_size)
    }

    /// Get storage slot value for a contract
    ///
    /// Erigon uses composite keys: address (20 bytes) + incarnation (8 bytes) + slot_key (32 bytes)
    pub fn get_storage(&self, address: Address, slot_key: U256) -> Result<Option<U256>> {
        let txn = self.db.begin_txn()?;
        
        // For storage queries, we need to find the account first to get incarnation
        let account = match self.get_account(address)? {
            Some(acc) => acc,
            None => return Ok(None), // Account doesn't exist
        };
        
        let incarnation = account.incarnation.unwrap_or(0);
        
        // Build composite key: address (20) + incarnation (8) + slot_key (32)
        let mut key = Vec::with_capacity(60);
        key.extend_from_slice(address.as_slice());
        key.extend_from_slice(&incarnation.to_be_bytes());
        
        // Convert U256 slot_key to 32 bytes big-endian
        let mut slot_bytes = [0u8; 32];
        slot_key.to_be_bytes_vec().iter().enumerate().for_each(|(i, &b)| {
            if i < 32 {
                slot_bytes[32 - slot_key.to_be_bytes_vec().len() + i] = b;
            }
        });
        key.extend_from_slice(&slot_bytes);
        
        // Query storage
        match self.db.get_plain_state(&txn, &self.storage_db, &key)? {
            Some(value_bytes) => {
                // Storage value is stored as RLP-encoded bytes
                if value_bytes.is_empty() {
                    Ok(Some(U256::ZERO))
                } else {
                    // Decode as big-endian U256
                    let value = U256::from_be_slice(&value_bytes);
                    debug!(
                        "Retrieved storage for {} slot {}: {}",
                        address, slot_key, value
                    );
                    Ok(Some(value))
                }
            }
            None => {
                debug!("Storage slot {} not found for {}", slot_key, address);
                Ok(None)
            }
        }
    }
}

/// Iterator for batch processing of accounts
pub struct AccountIterator<'a> {
    db: &'a ErigonDb,
    plain_state_db: &'a Database<heed::types::Bytes, heed::types::Bytes>,
    batch_size: usize,
    checkpoint: Option<Address>,
}

impl<'a> AccountIterator<'a> {
    fn new(
        db: &'a ErigonDb,
        plain_state_db: &'a Database<heed::types::Bytes, heed::types::Bytes>,
        batch_size: usize,
    ) -> Result<Self> {
        Ok(Self {
            db,
            plain_state_db,
            batch_size,
            checkpoint: None,
        })
    }

    /// Get the next batch of accounts
    pub fn next_batch(&mut self) -> Result<Vec<(Address, PlainAccount)>> {
        use crate::codec::decode_account;
        
        // Begin a new transaction
        let txn = self.db.begin_txn()?;
        
        let mut results = Vec::with_capacity(self.batch_size);
        
        // Always use range, starting from checkpoint or beginning
        let start_key = self.checkpoint
            .map(|addr| addr.as_slice().to_vec())
            .unwrap_or_else(|| vec![0u8; 0]);
        
        let iter = if start_key.is_empty() {
            // Start from beginning
            self.plain_state_db.iter(&txn)
                .map_err(|e| crate::error::Error::DatabaseRead(e.to_string()))?
        } else {
            // Start from checkpoint - skip the checkpoint itself
            let mut found_checkpoint = false;
            let all_iter = self.plain_state_db.iter(&txn)
                .map_err(|e| crate::error::Error::DatabaseRead(e.to_string()))?;
            
            // Collect remaining items after checkpoint
            for result in all_iter {
                if results.len() >= self.batch_size {
                    break;
                }
                
                match result {
                    Ok((key, value)) => {
                        // Skip until we pass the checkpoint
                        if !found_checkpoint {
                            if key == start_key.as_slice() {
                                found_checkpoint = true;
                            }
                            continue;
                        }
                        
                        // Key should be 20 bytes (address)
                        if key.len() != 20 {
                            warn!("Invalid key length: {} bytes, expected 20", key.len());
                            continue;
                        }
                        
                        let address = Address::from_slice(key);
                        
                        // Decode account
                        match decode_account(value) {
                            Ok(account) => {
                                results.push((address, account));
                                self.checkpoint = Some(address);
                            }
                            Err(e) => {
                                error!("Failed to decode account at {}: {}", address, e);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Database iteration error: {}", e);
                        break;
                    }
                }
            }
            
            debug!("Retrieved batch of {} accounts", results.len());
            return Ok(results);
        };
        
        // Collect batch from beginning
        for result in iter.take(self.batch_size) {
            match result {
                Ok((key, value)) => {
                    // Key should be 20 bytes (address)
                    if key.len() != 20 {
                        warn!("Invalid key length: {} bytes, expected 20", key.len());
                        continue;
                    }
                    
                    let address = Address::from_slice(key);
                    
                    // Decode account
                    match decode_account(value) {
                        Ok(account) => {
                            results.push((address, account));
                            self.checkpoint = Some(address);
                        }
                        Err(e) => {
                            error!("Failed to decode account at {}: {}", address, e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    error!("Database iteration error: {}", e);
                    break;
                }
            }
        }
        
        debug!("Retrieved batch of {} accounts", results.len());
        Ok(results)
    }

    /// Get current checkpoint
    pub fn checkpoint(&self) -> Option<Address> {
        self.checkpoint
    }

    /// Resume from checkpoint
    pub fn resume_from(&mut self, checkpoint: Address) {
        self.checkpoint = Some(checkpoint);
    }
    
    /// Reset iterator to start
    pub fn reset(&mut self) {
        self.checkpoint = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_reader_creation() {
        // Test will require actual database file
        // For now, just ensure types compile
    }
}
