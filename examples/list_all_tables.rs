//! Example: List ALL tables in the MDBX database
//!
//! This tool uses low-level APIs to discover all named databases

use heed::EnvOpenOptions;
use heed::types::Bytes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example list_all_tables <db_path>");
            std::process::exit(1);
        });

    println!("🔍 Discovering all tables in MDBX database\n");
    println!("Path: {}\n", db_path);

    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(200)  // Increase limit
            .open(&db_path)?
    };

    let txn = env.read_txn()?;

    println!("📊 Database Statistics:\n");
    
    // Try to open the unnamed (default) database
    println!("\n--- Unnamed Database (default) ---");
    match env.open_database::<Bytes, Bytes>(&txn, None) {
        Ok(Some(db)) => {
            let count = db.len(&txn)?;
            println!("  ✓ Found: {} records", count);
            
            if count > 0 && count < 50 {
                println!("\n  First few entries (might be table names):");
                let iter = db.iter(&txn)?;
                for (i, result) in iter.enumerate() {
                    if i >= 20 { break; }
                    match result {
                        Ok((key, _)) => {
                            if let Ok(key_str) = std::str::from_utf8(key) {
                                println!("    {}: {}", i + 1, key_str);
                            } else {
                                println!("    {}: [binary data, {} bytes]", i + 1, key.len());
                            }
                        }
                        Err(e) => {
                            println!("    Error: {}", e);
                            break;
                        }
                    }
                }
            }
        }
        Ok(None) => println!("  ○ Empty"),
        Err(e) => println!("  ✗ Error: {}", e),
    }

    // Try common variations of table names
    println!("\n--- Attempting Common Table Name Variations ---");
    
    let table_variations = vec![
        // Standard Erigon
        "PlainState", "PlainCodeHash", "Headers", "Bodies", "Receipts",
        // Lowercase
        "plainstate", "plaincodehash", "headers", "bodies", "receipts",
        // All caps
        "PLAINSTATE", "HEADERS", "BODIES",
        // With prefixes
        "Account", "Storage", "Code", "Hash", "History",
        "account", "storage", "code", "hash", "history",
        // BSC specific (guesses)
        "State", "Accounts", "AccountData", "ChainData",
        "state", "accounts", "accountdata", "chaindata",
        // Other possibilities
        "BlockData", "TxData", "Data",
    ];

    let mut found_tables = Vec::new();
    
    for name in &table_variations {
        match env.open_database::<Bytes, Bytes>(&txn, Some(name)) {
            Ok(Some(db)) => {
                match db.len(&txn) {
                    Ok(count) if count > 0 => {
                        println!("  ✓ {}: {} records", name, count);
                        found_tables.push((name.to_string(), count));
                    }
                    Ok(_) => {
                        println!("  ○ {}: exists but empty", name);
                    }
                    Err(e) => {
                        println!("  ? {}: error - {}", name, e);
                    }
                }
            }
            Ok(None) => {
                // Table doesn't exist
            }
            Err(_) => {
                // Error opening
            }
        }
    }

    if found_tables.is_empty() {
        println!("\n⚠️  No named tables found with data");
        println!("\nPossible reasons:");
        println!("  1. Database is truly empty (Erigon just started)");
        println!("  2. Different table naming scheme (custom Erigon build?)");
        println!("  3. Data stored in unnamed database");
        println!("  4. Permission issues (try with sudo if needed)");
        
        println!("\n💡 Suggestions:");
        println!("  - Check if Erigon is actually running and syncing");
        println!("  - Check Erigon configuration for custom table names");
        println!("  - Verify database permissions (file is owned by root)");
        println!("  - Try running with sudo:");
        println!("    sudo $(which cargo) run --example list_all_tables {}", db_path);
    } else {
        println!("\n✅ Found {} tables with data:", found_tables.len());
        for (name, count) in &found_tables {
            println!("  • {}: {} records", name, count);
        }
    }

    // Check file size vs records
    let file_size = std::fs::metadata(format!("{}/mdbx.dat", db_path))
        .map(|m| m.len())
        .unwrap_or(0);
    
    println!("\n📏 Database File Size:");
    println!("  mdbx.dat: {} GB", file_size / 1_000_000_000);
    
    if file_size > 100_000_000_000 && found_tables.is_empty() {
        println!("\n⚠️  WARNING: Large file ({} GB) but no accessible tables!", 
                 file_size / 1_000_000_000);
        println!("  This suggests a permission or compatibility issue.");
    }

    Ok(())
}
