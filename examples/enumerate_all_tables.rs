//! Enumerate ALL tables in MDBX database without knowing names in advance

use heed::{EnvOpenOptions, Database};
use heed::types::*;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <db_path>", args[0]);
        eprintln!("Example: {} /home/nvme/bsc-erigon/block_sync/chaindata", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    println!("🔍 Enumerating ALL tables in MDBX database");
    println!("📁 Path: {}\n", db_path);

    // Open environment
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(200)  // Increase to 200
            .map_size(10 * 1024 * 1024 * 1024)
            .open(db_path)?
    };

    let txn = env.read_txn()?;

    println!("{}", "=".repeat(70));
    println!("METHOD 1: Scan unnamed database for table names");
    println!("{}", "=".repeat(70));

    // The unnamed database contains the list of all named databases
    let unnamed_db: Database<Bytes, Bytes> = env.open_database(&txn, None)?
        .ok_or("Cannot open unnamed database")?;

    let mut table_names = Vec::new();
    
    for result in unnamed_db.iter(&txn)? {
        let (key, value) = result?;
        let name = String::from_utf8_lossy(key);
        
        // Print raw info
        println!("\n📋 Found entry:");
        println!("  Name: '{}'", name);
        println!("  Value length: {} bytes", value.len());
        println!("  Value (hex): {}", hex::encode(&value[..value.len().min(64)]));
        
        if !name.is_empty() {
            table_names.push(name.to_string());
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("METHOD 2: Try to open each discovered table");
    println!("{}", "=".repeat(70));

    for name in &table_names {
        println!("\n🔹 Table: '{}'", name);
        
        match env.open_database::<Bytes, Bytes>(&txn, Some(name)) {
            Ok(Some(db)) => {
                // Count entries
                let mut count = 0;
                let mut sample_keys = Vec::new();
                
                match db.iter(&txn) {
                    Ok(mut iter) => {
                        while let Some(Ok((key, value))) = iter.next() {
                            count += 1;
                            
                            if sample_keys.len() < 3 {
                                sample_keys.push((
                                    hex::encode(&key[..key.len().min(20)]),
                                    key.len(),
                                    value.len()
                                ));
                            }
                            
                            if count > 10000 {
                                break; // Just sample
                            }
                        }
                        
                        if count > 0 {
                            println!("  ✅ Accessible: {} entries (sampled)", count);
                            
                            if !sample_keys.is_empty() {
                                println!("  📊 Sample entries:");
                                for (i, (key_hex, key_len, val_len)) in sample_keys.iter().enumerate() {
                                    println!("    {}. key({} bytes): {}..., value({} bytes)", 
                                        i+1, key_len, key_hex, val_len);
                                }
                            }
                        } else {
                            println!("  ⚠️  Table exists but is empty");
                        }
                    }
                    Err(e) => {
                        println!("  ❌ Cannot iterate: {}", e);
                    }
                }
            }
            Ok(None) => {
                println!("  ⚠️  Cannot open (None returned)");
            }
            Err(e) => {
                println!("  ❌ Error opening: {}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("SUMMARY");
    println!("{}", "=".repeat(70));
    println!("📊 Total table registrations: {}", table_names.len());
    println!("📋 Table names found:");
    for name in &table_names {
        println!("  - {}", name);
    }

    Ok(())
}
