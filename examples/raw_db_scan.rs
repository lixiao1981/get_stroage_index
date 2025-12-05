//! Raw MDBX database scanner - attempts to open ALL possible database handles

use heed::{EnvOpenOptions, RoTxn, Database};
use heed::types::*;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <db_path>", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    println!("🔍 Raw MDBX Database Scanner");
    println!("📁 Path: {}\n", db_path);

    // Open environment with maximum settings
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)
            .map_size(10 * 1024 * 1024 * 1024) // 10GB
            .open(db_path)?
    };

    let txn = env.read_txn()?;

    println!("{}", "=".repeat(70));
    println!("STEP 1: Unnamed Database Scan");
    println!("{}", "=".repeat(70));
    
    // Try unnamed database
    let unnamed_db: Database<Bytes, Bytes> = env.open_database(&txn, None)?
        .ok_or("Failed to open unnamed database")?;
    
    let stats = txn.database_stat(&unnamed_db)?;
    println!("✓ Unnamed DB Stats:");
    println!("  - Entries: {}", stats.entries());
    println!("  - Depth: {}", stats.depth());
    println!("  - Branch pages: {}", stats.branch_pages());
    println!("  - Leaf pages: {}", stats.leaf_pages());
    println!("  - Overflow pages: {}", stats.overflow_pages());
    
    println!("\n📋 All entries in unnamed DB:");
    let mut count = 0;
    for result in unnamed_db.iter(&txn)? {
        let (key, value) = result?;
        count += 1;
        
        // Try to interpret as string
        let key_str = String::from_utf8_lossy(key);
        let value_preview = if value.len() <= 32 {
            format!("{:02x?}", value)
        } else {
            format!("{:02x?}... ({} bytes)", &value[..32], value.len())
        };
        
        println!("  {}: key='{}' value={}", count, key_str, value_preview);
        
        if count > 50 {
            println!("  ... (truncated, {} total entries)", stats.entries());
            break;
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("STEP 2: Named Database Discovery");
    println!("{}", "=".repeat(70));

    // Known Erigon table names from source code
    let table_names = vec![
        // Core tables
        "PlainState",
        "AccountHistory", 
        "StorageHistory",
        "AccountChangeSet",
        "StorageChangeSet",
        "Code",
        "PlainContractCode",
        "CodeHash",
        "TrieAccount",
        "TrieStorage",
        
        // Headers & Bodies
        "Headers",
        "HeaderNumber",
        "BlockBody",
        "BlockReceipts",
        "Transactions",
        
        // Indices
        "Log",
        "TxLookup",
        "CallTraceSet",
        
        // Misc
        "IncarnationMap",
        "CliqueSeparate",
        "CliqueSnapshot",
        "CliqueLastSnapshot",
        "BorReceipts",
        
        // Alternative naming
        "PlainAccountState",
        "Account",
        "Storage",
        "History",
        "ChangeSet",
    ];

    for name in table_names {
        match env.open_database::<Bytes, Bytes>(&txn, Some(name)) {
            Ok(Some(db)) => {
                match txn.database_stat(&db) {
                    Ok(stats) => {
                        let entries = stats.entries();
                        if entries > 0 {
                            println!("✓ '{}': {} entries", name, entries);
                            
                            // Show first entry
                            if let Ok(mut iter) = db.iter(&txn) {
                                if let Some(Ok((key, value))) = iter.next() {
                                    println!("    First key: {:02x?}... ({} bytes)", 
                                        &key[..key.len().min(20)], key.len());
                                    println!("    First value: {:02x?}... ({} bytes)",
                                        &value[..value.len().min(20)], value.len());
                                }
                            }
                        } else {
                            println!("○ '{}': exists but empty", name);
                        }
                    }
                    Err(e) => println!("⚠️  '{}': stat error: {}", name, e),
                }
            }
            Ok(None) => {
                // println!("  '{}': does not exist", name);
            }
            Err(e) => println!("⚠️  '{}': open error: {}", name, e),
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("STEP 3: Permission & Access Check");
    println!("{}", "=".repeat(70));
    
    // Check file permissions
    use std::fs;
    let mdbx_path = format!("{}/mdbx.dat", db_path);
    match fs::metadata(&mdbx_path) {
        Ok(meta) => {
            println!("✓ File accessible");
            println!("  - Size: {} bytes ({} GB)", 
                meta.len(), meta.len() / 1024 / 1024 / 1024);
            println!("  - Read-only: {}", meta.permissions().readonly());
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                println!("  - Mode: {:o}", meta.permissions().mode());
            }
        }
        Err(e) => println!("⚠️  Cannot access file: {}", e),
    }

    println!("\n{}", "=".repeat(70));
    println!("DIAGNOSIS:");
    println!("{}", "=".repeat(70));
    
    if unnamed_db.is_empty(&txn)? {
        println!("❌ Unnamed database is empty - no table metadata");
    } else {
        println!("✓ Unnamed database contains {} entries", stats.entries());
    }
    
    println!("\n💡 Next Steps:");
    println!("  1. Check if you have read permissions: ls -la {}", db_path);
    println!("  2. Try running with sudo if owned by root");
    println!("  3. Verify Erigon is not exclusively locking the database");
    println!("  4. Check Erigon logs for actual table names being used");

    Ok(())
}
