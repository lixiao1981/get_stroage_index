//! Example: Query account state from Erigon database
//!
//! This example demonstrates how to:
//! 1. Open an Erigon MDBX database
//! 2. Query account information by address
//! 3. Check balance, nonce, and contract status

use get_stroage_index::StateReader;
use alloy_primitives::Address;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Path to Erigon database (adjust to your setup)
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example query_account <db_path> [address]");
            eprintln!("Example: cargo run --example query_account /path/to/erigon/chaindata 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb");
            std::process::exit(1);
        });

    // Address to query (default: Vitalik's address)
    let address_str = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string());

    let address = Address::from_str(&address_str)?;

    println!("Opening Erigon database at: {}", db_path);
    println!("Querying address: {}\n", address);

    // Open the database
    let reader = StateReader::open(&db_path)?;

    // Query the account
    match reader.get_account(address)? {
        Some(account) => {
            println!("✓ Account found!");
            println!("  Nonce:      {}", account.nonce);
            println!("  Balance:    {} wei", account.balance);
            println!("  Is Contract: {}", account.is_contract());
            
            if let Some(code_hash) = account.code_hash {
                println!("  Code Hash:  {}", code_hash);
            }
            
            if let Some(incarnation) = account.incarnation {
                println!("  Incarnation: {}", incarnation);
            }
        }
        None => {
            println!("✗ Account not found (balance is zero or never interacted)");
        }
    }

    // Additional queries
    println!("\n--- Additional Queries ---");
    
    if let Some(balance) = reader.get_balance(address)? {
        println!("Balance only:  {} wei", balance);
    }
    
    if let Some(nonce) = reader.get_nonce(address)? {
        println!("Nonce only:    {}", nonce);
    }
    
    println!("Is contract:   {}", reader.is_contract(address)?);
    println!("Account exists: {}", reader.account_exists(address)?);

    Ok(())
}
