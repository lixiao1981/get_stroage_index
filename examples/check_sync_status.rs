//! Example: Check Erigon synchronization status
//!
//! This tool checks multiple tables to determine sync progress

use heed::{EnvOpenOptions, Database};
use heed::types::Bytes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example check_sync_status <db_path>");
            eprintln!("Example: cargo run --example check_sync_status /home/nvme/bsc-erigon/block_sync/chaindata");
            std::process::exit(1);
        });

    println!("🔍 Checking Erigon Sync Status\n");
    println!("Database: {}\n", db_path);

    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(100)
            .open(&db_path)?
    };

    let txn = env.read_txn()?;

    // Check critical tables for sync status
    let tables_to_check = vec![
        ("Headers", "Block headers"),
        ("Bodies", "Block bodies with transactions"),
        ("Receipts", "Transaction receipts"),
        ("PlainState", "Account state (target table)"),
        ("Code", "Smart contract bytecode"),
        ("HashedAccounts", "Hashed account state"),
        ("AccountHistory", "Historical account changes"),
        ("StorageHistory", "Historical storage changes"),
    ];

    println!("📊 Table Status:\n");
    
    let mut total_records = 0u64;
    let mut tables_with_data = 0;

    for (table_name, description) in &tables_to_check {
        match env.open_database::<Bytes, Bytes>(&txn, Some(table_name)) {
            Ok(Some(db)) => {
                match db.len(&txn) {
                    Ok(count) => {
                        let status = if count > 0 {
                            tables_with_data += 1;
                            total_records += count;
                            "✓"
                        } else {
                            "○"
                        };
                        println!("  {} {:<20} {:>15} records - {}", 
                            status, table_name, 
                            format_number(count), 
                            description
                        );
                    }
                    Err(e) => {
                        println!("  ? {:<20} Error: {}", table_name, e);
                    }
                }
            }
            Ok(None) | Err(_) => {
                println!("  ✗ {:<20} Not found", table_name);
            }
        }
    }

    println!("\n📈 Summary:\n");
    println!("  Tables with data: {}/{}", tables_with_data, tables_to_check.len());
    println!("  Total records: {}", format_number(total_records));

    // Analyze sync status
    println!("\n🎯 Sync Status Analysis:\n");
    
    let has_headers = check_table_exists(&env, &txn, "Headers")?;
    let has_bodies = check_table_exists(&env, &txn, "Bodies")?;
    let has_plainstate = check_table_exists(&env, &txn, "PlainState")?;
    
    if total_records == 0 {
        println!("  ⚠️  EMPTY DATABASE");
        println!("     - Erigon has just started");
        println!("     - No blocks synced yet");
        println!("     - Check Erigon logs for progress");
    } else if has_headers > 0 && has_plainstate == 0 {
        println!("  🔄 SYNCING IN PROGRESS");
        println!("     - Headers synced: ~{} blocks", format_number(has_headers));
        println!("     - PlainState is empty: account state not yet processed");
        println!("     - This is normal during initial sync");
        println!("     - PlainState fills up after block execution completes");
    } else if has_plainstate > 0 {
        println!("  ✅ SYNC COMPLETE (or nearly complete)");
        println!("     - PlainState has {} accounts", format_number(has_plainstate));
        println!("     - Ready for use!");
    }

    // Provide guidance
    println!("\n💡 Next Steps:\n");
    
    if has_plainstate == 0 {
        println!("  1. Wait for Erigon to sync more blocks");
        println!("  2. Check Erigon logs: look for 'Executed blocks' or 'stage Progress'");
        println!("  3. For BSC, initial sync can take 2-7 days depending on:");
        println!("     - Network speed");
        println!("     - Disk I/O (NVMe recommended)");
        println!("     - CPU cores (more cores = faster)");
        println!("\n  Monitor sync with:");
        println!("     tail -f /path/to/erigon.log | grep -i 'stage\\|progress\\|block'");
    } else {
        println!("  ✓ Database is ready!");
        println!("  Try: cargo run --example batch_iterate {} 100", db_path);
    }

    Ok(())
}

fn check_table_exists(
    env: &heed::Env, 
    txn: &heed::RoTxn, 
    table_name: &str
) -> Result<u64, Box<dyn std::error::Error>> {
    match env.open_database::<Bytes, Bytes>(txn, Some(table_name)) {
        Ok(Some(db)) => Ok(db.len(txn)?),
        _ => Ok(0),
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
