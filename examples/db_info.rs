//! Example: Display Erigon database information
//!
//! This example demonstrates how to:
//! 1. Open an Erigon database
//! 2. Detect database version (if available)
//! 3. Display basic statistics

use get_stroage_index::StateReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Path to Erigon database
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example db_info <db_path>");
            eprintln!("Example: cargo run --example db_info /path/to/erigon/chaindata");
            std::process::exit(1);
        });

    println!("📊 Erigon Database Information");
    println!("================================\n");
    println!("Database path: {}\n", db_path);

    // Open the database
    println!("Opening database...");
    let reader = StateReader::open(&db_path)?;
    println!("✓ Database opened successfully\n");

    // Try to detect version
    println!("--- Version Detection ---");
    match reader.get_version()? {
        Some(version) => {
            println!("✓ Version detected");
            println!("  Type:     {}", version.db_type);
            println!("  Version:  {}", version.version);
            if let Some(metadata) = version.metadata {
                println!("  Metadata: {}", metadata);
            }
        }
        None => {
            println!("⚠ Version information not available");
            println!("  (This is normal for some Erigon versions)");
        }
    }

    // Sample some accounts
    println!("\n--- Sample Accounts ---");
    let mut iter = reader.iter_accounts(10)?;
    
    match iter.next_batch() {
        Ok(batch) if !batch.is_empty() => {
            println!("✓ First {} accounts:", batch.len());
            for (i, (address, account)) in batch.iter().enumerate().take(5) {
                println!("  {}. {} - {} wei, nonce={}, contract={}",
                    i + 1,
                    address,
                    account.balance,
                    account.nonce,
                    account.is_contract()
                );
            }
            if batch.len() > 5 {
                println!("  ... and {} more", batch.len() - 5);
            }
        }
        Ok(_) => {
            println!("⚠ No accounts found in database");
        }
        Err(e) => {
            println!("✗ Error reading accounts: {}", e);
        }
    }

    // Test some well-known addresses
    println!("\n--- Well-Known Addresses ---");
    let known_addresses = vec![
        ("Vitalik.eth", "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"),
        ("Zero Address", "0x0000000000000000000000000000000000000000"),
    ];

    for (name, addr_str) in known_addresses {
        if let Ok(address) = addr_str.parse() {
            match reader.get_account(address) {
                Ok(Some(account)) => {
                    println!("  {} ({}): {} wei",
                        name, address, account.balance);
                }
                Ok(None) => {
                    println!("  {} ({}): not found", name, address);
                }
                Err(e) => {
                    println!("  {} ({}): error - {}", name, address, e);
                }
            }
        }
    }

    println!("\n✓ Database information complete");
    Ok(())
}
