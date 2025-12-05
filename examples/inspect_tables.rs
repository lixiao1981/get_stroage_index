//! Example: Inspect all tables in the MDBX database
//!
//! This diagnostic tool lists all available tables and their record counts
//! to help understand the database structure.

use heed::{EnvOpenOptions, Database};
use heed::types::Bytes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example inspect_tables <db_path>");
            eprintln!("Example: cargo run --example inspect_tables /home/nvme/bsc-erigon/block_sync");
            std::process::exit(1);
        });

    println!("Inspecting MDBX database at: {}\n", db_path);

    // Open environment
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)  // Allow up to 100 named databases
            .open(&db_path)?
    };

    println!("✓ Database opened successfully\n");

    // Start read transaction
    let txn = env.read_txn()?;

    // Try to list all databases
    println!("--- Named Databases ---");
    
    // Common Erigon table names to check
    let common_tables = vec![
        "PlainState",
        "PlainCodeHash", 
        "PlainAccountChangeSet",
        "PlainStorageChangeSet",
        "Code",
        "CodeHash",
        "AccountHistory",
        "StorageHistory",
        "Headers",
        "Bodies",
        "Receipts",
        "Config",
        "TxLookup",
        "Senders",
        "BlockReceipts",
        "HashedAccounts",
        "HashedStorage",
    ];

    let mut found_tables = Vec::new();

    for table_name in &common_tables {
        // Try to open the database
        match env.open_database::<Bytes, Bytes>(&txn, Some(table_name)) {
            Ok(Some(db)) => {
                match db.len(&txn) {
                    Ok(count) => {
                        println!("  ✓ {}: {} records", table_name, count);
                        found_tables.push((*table_name, count));
                    }
                    Err(_) => {
                        println!("  ? {}: exists but count failed", table_name);
                        found_tables.push((*table_name, 0));
                    }
                }
            }
            Ok(None) | Err(_) => {
                // Table doesn't exist, skip silently
            }
        }
    }

    if found_tables.is_empty() {
        println!("  ⚠ No standard Erigon tables found");
        println!("\nThis might mean:");
        println!("  1. Wrong database path (need the chaindata/mdbx.dat directory)");
        println!("  2. Database is empty or corrupted");
        println!("  3. Non-standard Erigon configuration");
    } else {
        println!("\n--- Summary ---");
        println!("Found {} tables with data:", found_tables.iter().filter(|(_, c)| *c > 0).count());
        for (name, count) in found_tables.iter().filter(|(_, c)| *c > 0) {
            println!("  • {}: {:>12} records", name, count);
        }
    }

    // Check if PlainState exists with details
    if let Some((_, count)) = found_tables.iter().find(|(n, _)| *n == "PlainState") {
        println!("\n--- PlainState Details ---");
        if *count > 0 {
            let db: Database<Bytes, Bytes> = env
                .open_database(&txn, Some("PlainState"))?
                .ok_or("PlainState table not found")?;
            
            // Get first few keys
            println!("First 5 account addresses:");
            let iter = db.iter(&txn)?;
            for (i, result) in iter.enumerate() {
                if i >= 5 { break; }
                match result {
                    Ok((key, _)) => {
                        if key.len() == 20 {
                            println!("  {}: 0x{}", i + 1, hex::encode(key));
                        } else {
                            println!("  {}: unexpected key length {} bytes", i + 1, key.len());
                        }
                    }
                    Err(e) => {
                        println!("  Error reading entry: {}", e);
                        break;
                    }
                }
            }
        } else {
            println!("  ⚠ PlainState table exists but is empty");
        }
    }

    Ok(())
}
