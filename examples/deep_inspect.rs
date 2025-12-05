//! Deep inspection of MDBX database internals
//! This tool analyzes the low-level MDBX structure to understand why data isn't visible

use anyhow::{Context, Result};
use heed::{EnvOpenOptions, RoTxn};
use std::env;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_chaindata>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} /home/nvme/bsc-erigon/block_sync/chaindata", args[0]);
        std::process::exit(1);
    }

    let db_path = Path::new(&args[1]);
    
    println!("🔬 Deep MDBX Database Inspection\n");
    println!("Path: {}\n", db_path.display());

    // Open environment with maximum compatibility
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(200)
            .map_size(1024 * 1024 * 1024 * 1024) // 1TB
            .open(db_path)
            .context("Failed to open MDBX environment")?
    };

    let rtxn = env.read_txn()?;

    println!("📊 Environment Information:");
    println!("  Max databases: 200");
    println!("  Map size: 1 TB");
    
    // Get environment info
    let info = env.info();
    println!("  Actual map size: {} bytes ({:.2} GB)", info.map_size, info.map_size as f64 / 1024.0 / 1024.0 / 1024.0);
    println!("  Last page number: {}", info.last_page_number);
    println!("  Last transaction ID: {}", info.last_txn_id);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Try to open unnamed database directly
    println!("🔍 Inspecting Unnamed Database (raw iteration):\n");
    
    let unnamed_db = env.open_database::<heed::types::Bytes, heed::types::Bytes>(&rtxn, None)?
        .context("Failed to open unnamed database")?;
    
    let unnamed_stat = unnamed_db.stat(&rtxn)?;
    println!("  Unnamed DB Statistics:");
    println!("    Depth: {}", unnamed_stat.depth);
    println!("    Branch pages: {}", unnamed_stat.branch_pages);
    println!("    Leaf pages: {}", unnamed_stat.leaf_pages);
    println!("    Overflow pages: {}", unnamed_stat.overflow_pages);
    println!("    Entries: {}\n", unnamed_stat.entries);

    println!("  Unnamed DB Contents:");
    let mut count = 0;
    for result in unnamed_db.iter(&rtxn)? {
        let (key_bytes, value_bytes) = result?;
        count += 1;
        
        let key_str = String::from_utf8_lossy(key_bytes);
        let key_hex = hex::encode(&key_bytes[..key_bytes.len().min(32)]);
        
        println!("    Entry {}: ", count);
        println!("      Key (string): '{}'", key_str);
        println!("      Key (hex): {}", key_hex);
        println!("      Key length: {} bytes", key_bytes.len());
        println!("      Value length: {} bytes", value_bytes.len());
        
        if count < 20 {
            println!("      Value (first 64 bytes hex): {}", hex::encode(&value_bytes[..value_bytes.len().min(64)]));
        }
        println!();
        
        if count >= 50 {
            println!("    ... (showing first 50 entries only)\n");
            break;
        }
    }
    
    if count == 0 {
        println!("    ⚠️  Unnamed database is completely empty!\n");
    } else {
        println!("  Total entries in unnamed DB: {}\n", count);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Try different approaches to open PlainState
    println!("🔍 Attempting to Open PlainState (multiple methods):\n");
    
    let table_names = vec![
        "PlainState",
        "PLAINSTATE", 
        "plainstate",
        "PlainSTATE",
        "Plain-State",
        "Plain_State",
    ];
    
    for table_name in &table_names {
        print!("  Trying '{}' ... ", table_name);
        
        match try_open_table(&env, &rtxn, table_name) {
            Ok((stat, first_keys)) => {
                println!("✓ OPENED!");
                println!("    Entries: {}", stat.entries);
                println!("    Depth: {}", stat.depth);
                println!("    Leaf pages: {}", stat.leaf_pages);
                
                if !first_keys.is_empty() {
                    println!("    First {} keys:", first_keys.len().min(5));
                    for (i, key) in first_keys.iter().take(5).enumerate() {
                        println!("      {}: {} ({} bytes)", i + 1, key, key.len() / 2);
                    }
                } else {
                    println!("    ⚠️  Table exists but is EMPTY");
                }
                println!();
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
            }
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Scan for all possible database names in the unnamed database
    println!("🔍 Extracting Table Names from Unnamed DB:\n");
    
    let mut table_names_found = Vec::new();
    for result in unnamed_db.iter(&rtxn)? {
        let (key_bytes, _) = result?;
        if let Ok(name) = std::str::from_utf8(key_bytes) {
            if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') && name.len() > 2 {
                table_names_found.push(name.to_string());
            }
        }
    }
    
    if table_names_found.is_empty() {
        println!("  ⚠️  No table names found in unnamed database!\n");
        println!("  This suggests the database is not initialized or uses a different structure.\n");
    } else {
        println!("  Found {} potential table names:", table_names_found.len());
        for (i, name) in table_names_found.iter().enumerate() {
            println!("    {}. {}", i + 1, name);
        }
        println!();
        
        // Try to open each discovered table
        println!("  Attempting to open discovered tables:\n");
        for name in &table_names_found {
            print!("    '{}' ... ", name);
            match try_open_table(&env, &rtxn, name) {
                Ok((stat, _)) => {
                    println!("✓ Opened! Entries: {}", stat.entries);
                }
                Err(e) => {
                    println!("✗ Failed: {}", e);
                }
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("📋 Summary:\n");
    println!("  If all tables show 0 entries:");
    println!("    → Database is truly empty (Erigon hasn't synced yet)");
    println!("  If tables can't be opened:");
    println!("    → Different MDBX version or incompatible flags");
    println!("  If unnamed DB is empty:");
    println!("    → Database not initialized or corrupted");
    println!("\n  Next steps:");
    println!("    1. Check Erigon logs: journalctl -u bsc-erigon -n 100");
    println!("    2. Verify Erigon is running: ps aux | grep erigon");
    println!("    3. Check sync status in Erigon logs for 'Execution' stage");

    Ok(())
}

fn try_open_table(
    env: &heed::Env, 
    rtxn: &RoTxn, 
    name: &str
) -> Result<(heed::DatabaseStat, Vec<String>)> {
    let db = env.open_database::<heed::types::Bytes, heed::types::Bytes>(rtxn, Some(name))?
        .context(format!("Table '{}' does not exist", name))?;
    
    let stat = db.stat(rtxn)?;
    
    let mut first_keys = Vec::new();
    for result in db.iter(rtxn)?.take(10) {
        let (key_bytes, _) = result?;
        first_keys.push(hex::encode(key_bytes));
    }
    
    Ok((stat, first_keys))
}
