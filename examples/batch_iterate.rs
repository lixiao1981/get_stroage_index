//! Example: Batch iterate over all accounts in the database
//!
//! This example demonstrates how to:
//! 1. Create an account iterator with configurable batch size
//! 2. Process accounts in batches
//! 3. Handle checkpoint-based resumption

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
            eprintln!("Usage: cargo run --example batch_iterate <db_path> [batch_size]");
            eprintln!("Example: cargo run --example batch_iterate /path/to/erigon/chaindata 1000");
            std::process::exit(1);
        });

    // Batch size (default: 100)
    let batch_size: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    println!("Opening Erigon database at: {}", db_path);
    println!("Batch size: {}\n", batch_size);

    // Open the database
    let reader = StateReader::open(&db_path)?;

    // Create iterator
    let mut iter = reader.iter_accounts(batch_size)?;

    let mut total_accounts = 0;
    let mut total_balance = alloy_primitives::U256::ZERO;
    let mut contract_count = 0;
    let mut batch_num = 0;

    // Iterate through batches
    loop {
        let batch = iter.next_batch()?;
        
        if batch.is_empty() {
            println!("\n✓ Reached end of database");
            break;
        }

        batch_num += 1;
        println!("Batch #{}: {} accounts", batch_num, batch.len());

        for (address, account) in batch {
            total_accounts += 1;
            total_balance += account.balance;
            
            if account.is_contract() {
                contract_count += 1;
            }

            // Print first few accounts in each batch
            if total_accounts <= 5 {
                println!("  {}: balance={} wei, nonce={}, contract={}",
                    address,
                    account.balance,
                    account.nonce,
                    account.is_contract()
                );
            }
        }

        // Stop after a few batches for demo purposes
        if batch_num >= 10 {
            println!("\n... (stopping after 10 batches for demo)");
            break;
        }
    }

    // Print summary
    println!("\n--- Summary ---");
    println!("Total accounts processed: {}", total_accounts);
    println!("Total balance: {} wei", total_balance);
    println!("Contract accounts: {}", contract_count);
    println!("EOA accounts: {}", total_accounts - contract_count);
    
    if let Some(checkpoint) = iter.checkpoint() {
        println!("\nCheckpoint: {}", checkpoint);
        println!("(Can resume iteration from this address)");
    }

    Ok(())
}
