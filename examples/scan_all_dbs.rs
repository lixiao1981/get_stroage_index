//! Scan all Erigon databases (chaindata, parlia, blobs)

use heed::{EnvOpenOptions, Database};
use heed::types::*;
use std::env;
use std::path::Path;

fn scan_database(db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(70));
    println!("📁 Database: {}", db_path);
    println!("{}", "=".repeat(70));
    
    if !Path::new(db_path).exists() {
        println!("⚠️  Path does not exist!");
        return Ok(());
    }

    // Check if it's a directory with mdbx.dat
    let mdbx_file = format!("{}/mdbx.dat", db_path);
    if !Path::new(&mdbx_file).exists() {
        println!("⚠️  No mdbx.dat file found!");
        return Ok(());
    }

    // Get file size
    let metadata = std::fs::metadata(&mdbx_file)?;
    let size_gb = metadata.len() / 1024 / 1024 / 1024;
    println!("📏 File size: {} GB", size_gb);

    // Open environment
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)
            .map_size(10 * 1024 * 1024 * 1024)
            .open(db_path)?
    };

    let txn = env.read_txn()?;

    // Check unnamed database
    let unnamed_db: Database<Bytes, Bytes> = match env.open_database(&txn, None)? {
        Some(db) => db,
        None => {
            println!("⚠️  Cannot open unnamed database");
            return Ok(());
        }
    };

    let mut unnamed_count = 0;
    for _ in unnamed_db.iter(&txn)? {
        unnamed_count += 1;
    }
    
    println!("📊 Unnamed DB: {} entries", unnamed_count);
    
    if unnamed_count > 0 {
        println!("\n  Table registrations:");
        for result in unnamed_db.iter(&txn)? {
            let (key, _value) = result?;
            let key_str = String::from_utf8_lossy(key);
            println!("    - {}", key_str);
        }
    }

    // Try to open known tables
    let tables = vec![
        "PlainState", "AccountHistory", "StorageHistory",
        "AccountChangeSet", "StorageChangeSet",
        "Headers", "BlockBody", "Transactions",
        "Code", "IncarnationMap",
    ];

    println!("\n  Named tables:");
    let mut found_tables = Vec::new();
    
    for name in &tables {
        if let Ok(Some(db)) = env.open_database::<Bytes, Bytes>(&txn, Some(name)) {
            let mut count = 0;
            for _ in db.iter(&txn)? {
                count += 1;
                if count > 1000 {
                    break; // Just sample for large tables
                }
            }
            
            if count > 0 {
                println!("    ✓ '{}': {} entries (sampled)", name, count);
                found_tables.push((*name, count));
            }
        }
    }

    if found_tables.is_empty() {
        println!("    ⚠️  No tables with data found");
    } else {
        println!("\n  📈 Summary: {} tables with data", found_tables.len());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let base_path = if args.len() >= 2 {
        args[1].clone()
    } else {
        "/home/nvme/bsc-erigon/block_sync".to_string()
    };

    println!("🔍 Erigon Multi-Database Scanner");
    println!("📂 Base path: {}\n", base_path);

    // Scan all known Erigon databases
    let databases = vec![
        format!("{}/chaindata", base_path),
        format!("{}/parlia", base_path),
        format!("{}/blobs/blob", base_path),
        format!("{}/blobpool", base_path),
    ];

    let mut found_any = false;
    
    for db_path in databases {
        if let Err(e) = scan_database(&db_path) {
            println!("  ❌ Error: {}", e);
        } else {
            found_any = true;
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("CONCLUSIONS:");
    println!("{}", "=".repeat(70));
    
    if !found_any {
        println!("❌ No databases successfully scanned");
    } else {
        println!("✓ Scan complete - check results above");
    }
    
    println!("\n💡 If PlainState is empty in all databases:");
    println!("   1. Your archive node may not use PlainState");
    println!("   2. Data is in History tables (different architecture)");
    println!("   3. Need to query eth_getStorageAt via RPC instead");

    Ok(())
}
