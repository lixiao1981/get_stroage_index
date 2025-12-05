//! Domain models for Ethereum account state

use alloy_primitives::{Address, FixedBytes, U256};

/// Represents an Ethereum account in the PlainState
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainAccount {
    /// Transaction count (nonce)
    pub nonce: u64,

    /// Account balance in wei
    pub balance: U256,

    /// Storage root hash (only for contract accounts)
    pub storage_root: Option<FixedBytes<32>>,

    /// Code hash (only for contract accounts)
    pub code_hash: Option<FixedBytes<32>>,

    /// Incarnation number for contract upgrades
    pub incarnation: Option<u64>,
}

impl PlainAccount {
    /// Create a new Externally Owned Account (EOA)
    pub fn new_eoa(nonce: u64, balance: U256) -> Self {
        Self {
            nonce,
            balance,
            storage_root: None,
            code_hash: None,
            incarnation: None,
        }
    }

    /// Create a new contract account
    pub fn new_contract(
        nonce: u64,
        balance: U256,
        storage_root: FixedBytes<32>,
        code_hash: FixedBytes<32>,
        incarnation: u64,
    ) -> Self {
        Self {
            nonce,
            balance,
            storage_root: Some(storage_root),
            code_hash: Some(code_hash),
            incarnation: Some(incarnation),
        }
    }

    /// Check if this is a contract account
    pub fn is_contract(&self) -> bool {
        self.code_hash.is_some()
    }

    /// Check if this is an EOA
    pub fn is_eoa(&self) -> bool {
        !self.is_contract()
    }
}

/// Represents a storage slot in a contract
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSlot {
    /// Contract address
    pub address: Address,

    /// Storage slot key (256-bit)
    pub key: U256,

    /// Storage slot value (256-bit)
    pub value: U256,
}

impl StorageSlot {
    /// Create a new storage slot
    pub fn new(address: Address, key: U256, value: U256) -> Self {
        Self {
            address,
            key,
            value,
        }
    }
}

/// Database version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbVersion {
    /// Schema version number
    pub version: String,
    
    /// Database type (e.g., "mdbx", "lmdb")
    pub db_type: String,
    
    /// Additional metadata
    pub metadata: Option<String>,
}

impl DbVersion {
    /// Create a new DbVersion
    pub fn new(version: impl Into<String>, db_type: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            db_type: db_type.into(),
            metadata: None,
        }
    }
    
    /// Create with metadata
    pub fn with_metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eoa_creation() {
        let account = PlainAccount::new_eoa(5, U256::from(1000u64));
        assert_eq!(account.nonce, 5);
        assert_eq!(account.balance, U256::from(1000u64));
        assert!(account.is_eoa());
        assert!(!account.is_contract());
        assert!(account.code_hash.is_none());
    }

    #[test]
    fn test_contract_creation() {
        let storage_root = FixedBytes::<32>::default();
        let code_hash = FixedBytes::<32>::default();
        let account = PlainAccount::new_contract(1, U256::from(500u64), storage_root, code_hash, 1);
        
        assert!(account.is_contract());
        assert!(!account.is_eoa());
        assert!(account.code_hash.is_some());
        assert_eq!(account.incarnation, Some(1));
    }
    
    #[test]
    fn test_storage_slot_creation() {
        let address = Address::default();
        let key = U256::from(42u64);
        let value = U256::from(100u64);
        
        let slot = StorageSlot::new(address, key, value);
        assert_eq!(slot.address, address);
        assert_eq!(slot.key, key);
        assert_eq!(slot.value, value);
    }
    
    #[test]
    fn test_db_version() {
        let version = DbVersion::new("2.54.0", "mdbx");
        assert_eq!(version.version, "2.54.0");
        assert_eq!(version.db_type, "mdbx");
        assert!(version.metadata.is_none());
        
        let version_with_meta = version.with_metadata("erigon v2.54.0");
        assert_eq!(version_with_meta.metadata, Some("erigon v2.54.0".to_string()));
    }
}
