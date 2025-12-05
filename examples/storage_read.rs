//! Example: Read contract storage slots
//!
//! This example demonstrates how to:
//! 1. Query contract storage slots
//! 2. Handle composite key structure (address + incarnation + slot)

use get_stroage_index::StateReader;
use alloy_primitives::{Address, U256};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Path to Erigon database
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example storage_read <db_path> <contract_address> <slot>");
            eprintln!("Example: cargo run --example storage_read /path/to/chaindata 0x... 0");
            std::process::exit(1);
        });

    // Contract address
    let address_str = std::env::args()
        .nth(2)
        .unwrap_or_else(|| {
            eprintln!("Please provide a contract address");
            std::process::exit(1);
        });

    let address = Address::from_str(&address_str)?;

    // Storage slot (default: slot 0)
    let slot: u64 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let slot_key = U256::from(slot);

    println!("Opening Erigon database at: {}", db_path);
    println!("Contract address: {}", address);
    println!("Storage slot: {}\n", slot);

    // Open the database
    let reader = StateReader::open(&db_path)?;

    // First, check if it's a contract
    match reader.get_account(address)? {
        Some(account) => {
            println!("✓ Account found");
            println!("  Is contract: {}", account.is_contract());
            
            if !account.is_contract() {
                println!("\n⚠ Warning: This address is not a contract (no code hash)");
                println!("  Storage query may return empty results");
            }
            
            if let Some(incarnation) = account.incarnation {
                println!("  Incarnation: {}", incarnation);
            }
        }
        None => {
            println!("✗ Account not found");
            return Ok(());
        }
    }

    // Query storage
    println!("\n--- Querying Storage ---");
    match reader.get_storage(address, slot_key)? {
        Some(value) => {
            println!("✓ Storage slot {} value: {}", slot, value);
            println!("  Hex: 0x{:x}", value);
            
            // Try to interpret as common types
            println!("\n--- Possible Interpretations ---");
            println!("  As u64:   {}", value.to::<u64>());
            println!("  As bool:  {}", if value.is_zero() { "false" } else { "true" });
            
            // Check if it looks like an address (first 20 bytes)
            let bytes = value.to_be_bytes::<32>();
            if bytes[0..12].iter().all(|&b| b == 0) {
                let addr_bytes = &bytes[12..32];
                let potential_addr = Address::from_slice(addr_bytes);
                println!("  As address: {}", potential_addr);
            }
        }
        None => {
            println!("✗ Storage slot {} is empty or not found", slot);
            println!("  (This is normal for unused slots)");
        }
    }

    // Query additional slots
    println!("\n--- Querying First 5 Slots ---");
    for i in 0..5 {
        let slot_i = U256::from(i);
        if let Some(value) = reader.get_storage(address, slot_i)? {
            if !value.is_zero() {
                println!("  Slot {}: 0x{:x}", i, value);
            }
        }
    }

    Ok(())
}
