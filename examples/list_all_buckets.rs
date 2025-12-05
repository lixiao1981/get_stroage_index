//! List ALL buckets/DBIs using multiple MDBX inspection methods
//! Based on Erigon's erigon-lib/kv BucketMigratorRO.ListBuckets() approach

use heed::{EnvOpenOptions, Database, RoTxn};
use heed::types::*;
use std::env;
use std::collections::HashSet;

/// Method 1: Scan unnamed database (standard approach)
fn list_from_unnamed_db(env: &heed::Env, txn: &RoTxn) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(70));
    println!("METHOD 1: List from unnamed database (standard)");
    println!("{}", "=".repeat(70));
    
    let unnamed_db: Database<Bytes, Bytes> = env.open_database(txn, None)?
        .ok_or("Cannot open unnamed database")?;

    let mut names = Vec::new();
    
    for result in unnamed_db.iter(txn)? {
        let (key, value) = result?;
        let name = String::from_utf8_lossy(key).to_string();
        
        if !name.is_empty() {
            println!("  ✓ Found: '{}' (metadata: {} bytes)", name, value.len());
            names.push(name);
        } else {
            println!("  ⚠️  Empty key with {} byte value", value.len());
        }
    }
    
    println!("  📊 Total: {} named databases", names.len());
    Ok(names)
}

/// Method 2: Try all known Erigon table names
fn try_known_tables(env: &heed::Env, txn: &RoTxn) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(70));
    println!("METHOD 2: Try all known Erigon table names");
    println!("{}", "=".repeat(70));
    
    // Comprehensive list from Erigon's kv/tables.go
    let known_names = vec![
        // === Erigon v2 Legacy Tables ===
        "PlainState",
        "AccountHistory", 
        "StorageHistory",
        "Code",
        "ContractCode",
        "AccountChangeSet",
        "StorageChangeSet",
        "PlainContractCode",
        "IncarnationMap",
        "CliqueSeparate",
        "CliqueSnapshot",
        "CliqueLastSnapshot",
        
        // === Erigon v3 Domain Tables ===
        // Account domain
        "AccountKeys", "AccountVals", "AccountHistoryKeys", "AccountHistoryVals", "AccountIdx",
        // Storage domain
        "StorageKeys", "StorageVals", "StorageHistoryKeys", "StorageHistoryVals", "StorageIdx",
        // Code domain
        "CodeKeys", "CodeVals", "CodeHistoryKeys", "CodeHistoryVals", "CodeIdx",
        // Commitment domain
        "CommitmentKeys", "CommitmentVals", "CommitmentHistoryKeys", "CommitmentHistoryVals", "CommitmentIdx",
        
        // === Block Data Tables ===
        "Headers",
        "HeaderNumber",
        "HeaderCanonical",
        "HeaderTD",
        "BlockBody",
        "BlockTransaction",
        "EthTx",
        "NonCanonicalTxs",
        "Receipts",
        "Log",
        "CallTraceSet",
        "CallFromIndex",
        "CallToIndex",
        "Senders",
        
        // === Sync/Chain Tables ===
        "SyncStageProgress",
        "SyncStageUnwind",
        "PlainAccountChangeSet",
        "PlainStorageChangeSet",
        "Sequence",
        "EpochInfo",
        "PendingBlockBaseFeee",
        
        // === Bor/Polygon Tables ===
        "BorReceipts",
        "BorTxLookup",
        "BorSeparate",
        
        // === Trie/Hash Tables ===
        "TrieOfAccounts",
        "TrieOfStorage",
        "HashedAccounts",
        "HashedStorage",
        
        // === Transaction Pool ===
        "PoolTransaction",
        "PoolInfo",
        
        // === Snapshots ===
        "SnapshotInfo",
        "HeadersSnapshotInfo",
        "BodiesSnapshotInfo",
        "StateSnapshotInfo",
        
        // === Misc ===
        "Migrations",
        "ConfigTable",
        "DatabaseInfo",
        "TxLookup",
        "BloomBits",
        "Preimages",
        "BadHeaderNumber",
    ];
    
    let mut found = Vec::new();
    
    for name in known_names {
        match env.open_database::<Bytes, Bytes>(txn, Some(name)) {
            Ok(Some(db)) => {
                // Try to get first entry
                match db.iter(txn) {
                    Ok(mut iter) => {
                        if iter.next().is_some() {
                            println!("  ✅ '{}': has data", name);
                            found.push(name.to_string());
                        } else {
                            println!("  ⚠️  '{}': exists but empty", name);
                        }
                    }
                    Err(e) => {
                        println!("  ❌ '{}': cannot iterate ({})", name, e);
                    }
                }
            }
            Ok(None) => {
                // This is normal - table doesn't exist
            }
            Err(e) => {
                println!("  ❌ '{}': error ({})", name, e);
            }
        }
    }
    
    println!("  📊 Total: {} tables with data", found.len());
    Ok(found)
}

/// Method 3: Check database statistics
fn show_db_stats(env: &heed::Env, txn: &RoTxn) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(70));
    println!("METHOD 3: Database environment info");
    println!("{}", "=".repeat(70));
    
    let info = env.info();
    println!("  Map size: {} bytes ({:.2} GB)", info.map_size, info.map_size as f64 / 1024.0 / 1024.0 / 1024.0);
    println!("  Last page number: {}", info.last_page_number);
    println!("  Last transaction ID: {}", info.last_txn_id);
    println!("  Max readers: {}", info.maximum_number_of_readers);
    println!("  Num readers: {}", info.number_of_readers);
    
    // Try to get unnamed database statistics
    if let Ok(Some(unnamed_db)) = env.open_database::<Bytes, Bytes>(txn, None) {
        if let Ok(stat) = unnamed_db.stat(txn) {
            println!("\n  Unnamed DB statistics:");
            println!("    Page size: {} bytes", stat.page_size);
            println!("    Depth: {}", stat.depth);
            println!("    Branch pages: {}", stat.branch_pages);
            println!("    Leaf pages: {}", stat.leaf_pages);
            println!("    Overflow pages: {}", stat.overflow_pages);
            println!("    Entries: {}", stat.entries);
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <db_path>", args[0]);
        eprintln!("Example: {} /home/nvme/bsc-erigon/block_sync/chaindata", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    println!("🔍 MDBX Bucket/DBI Lister (Erigon-compatible)");
    println!("📁 Database: {}\n", db_path);

    // Open environment with large max_dbs
    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(300)  // Support up to 300 databases
            .map_size(10 * 1024 * 1024 * 1024)
            .open(db_path)?
    };

    let txn = env.read_txn()?;

    // Show database statistics first
    show_db_stats(&env, &txn)?;

    // Method 1: Standard approach
    let method1_tables = list_from_unnamed_db(&env, &txn)?;

    // Method 2: Try all known names
    let method2_tables = try_known_tables(&env, &txn)?;

    // Combine and deduplicate
    println!("\n{}", "=".repeat(70));
    println!("FINAL SUMMARY");
    println!("{}", "=".repeat(70));
    
    let mut all_tables: HashSet<String> = HashSet::new();
    all_tables.extend(method1_tables.iter().cloned());
    all_tables.extend(method2_tables.iter().cloned());
    
    let mut sorted: Vec<_> = all_tables.into_iter().collect();
    sorted.sort();
    
    println!("\n📊 Total unique tables discovered: {}", sorted.len());
    println!("\n📋 Complete table list:");
    for (i, name) in sorted.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }
    
    if sorted.is_empty() {
        println!("\n❌ NO TABLES FOUND!");
        println!("\n💡 Possible reasons:");
        println!("  1. Database is pre-allocated but not initialized");
        println!("  2. Erigon is still in snapshot download phase");
        println!("  3. State construction hasn't started yet");
        println!("  4. Wrong database path (check Erigon config)");
    } else if sorted.len() == 1 && sorted[0] == "PlainState" {
        println!("\n⚠️  ONLY PlainState found - database minimally initialized");
        println!("\n💡 This suggests:");
        println!("  • Erigon created the table structure");
        println!("  • But hasn't populated it with data yet");
        println!("  • Check Erigon sync status: journalctl -u bsc-erigon -f");
    } else {
        println!("\n✅ Database appears operational");
    }

    Ok(())
}
