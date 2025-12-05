//! Example: Inspect the unnamed (default) MDBX database
//!
//! BSC-Erigon might store data in the default database instead of named tables

use heed::EnvOpenOptions;
use heed::types::Bytes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example inspect_default_db <db_path>");
            std::process::exit(1);
        });

    println!("🔍 Inspecting Unnamed (Default) Database\n");
    println!("Path: {}\n", db_path);

    let env = unsafe {
        EnvOpenOptions::new()
            .max_dbs(200)
            .open(&db_path)?
    };

    let txn = env.read_txn()?;

    // Open the unnamed database
    match env.open_database::<Bytes, Bytes>(&txn, None) {
        Ok(Some(db)) => {
            let count = db.len(&txn)?;
            println!("📊 Unnamed Database Statistics:");
            println!("  Total entries: {}\n", count);

            if count == 0 {
                println!("⚠️  Database is empty");
                return Ok(());
            }

            // Analyze key patterns
            println!("🔑 Analyzing Key Patterns:\n");
            
            let mut address_like_keys = 0;  // 20 bytes
            let mut hash_like_keys = 0;     // 32 bytes
            let mut short_keys = 0;         // < 20 bytes
            let mut long_keys = 0;          // > 32 bytes
            let mut text_keys = 0;          // UTF-8 strings
            
            let sample_size = 100.min(count as usize);
            let iter = db.iter(&txn)?;
            
            for (i, result) in iter.enumerate() {
                if i >= sample_size { break; }
                
                match result {
                    Ok((key, value)) => {
                        let key_len = key.len();
                        
                        // Categorize by length
                        match key_len {
                            0..=19 => short_keys += 1,
                            20 => address_like_keys += 1,
                            21..=31 => {},
                            32 => hash_like_keys += 1,
                            _ => long_keys += 1,
                        }
                        
                        // Check if key is UTF-8
                        if std::str::from_utf8(key).is_ok() {
                            text_keys += 1;
                        }
                        
                        // Print first 10 entries
                        if i < 10 {
                            if let Ok(key_str) = std::str::from_utf8(key) {
                                println!("  Entry {}: key='{}' ({} bytes), value={} bytes",
                                    i + 1, key_str, key.len(), value.len());
                            } else if key_len == 20 {
                                println!("  Entry {}: key=0x{} (address?), value={} bytes",
                                    i + 1, hex::encode(key), value.len());
                            } else {
                                println!("  Entry {}: key=[{} bytes binary], value={} bytes",
                                    i + 1, key.len(), value.len());
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Error reading entry: {}", e);
                        break;
                    }
                }
            }
            
            // Summary
            println!("\n📈 Key Pattern Summary (from {} samples):\n", sample_size);
            println!("  Text keys (table names?):     {}", text_keys);
            println!("  20-byte keys (addresses?):    {}", address_like_keys);
            println!("  32-byte keys (hashes?):       {}", hash_like_keys);
            println!("  Short keys (< 20 bytes):      {}", short_keys);
            println!("  Long keys (> 32 bytes):       {}", long_keys);
            
            // Interpretation
            println!("\n💡 Interpretation:\n");
            if text_keys > sample_size / 2 {
                println!("  ✓ Likely contains table metadata/names");
                println!("  → This is the MDBX internal database");
            } else if address_like_keys > sample_size / 2 {
                println!("  ✓ Contains 20-byte keys (Ethereum addresses!)");
                println!("  → Account data might be stored here");
                println!("  → BSC-Erigon may use unnamed database for accounts");
            } else if hash_like_keys > sample_size / 2 {
                println!("  ✓ Contains 32-byte keys (hashes)");
                println!("  → Could be storage data or code hashes");
            } else {
                println!("  ? Mixed key types - need deeper analysis");
            }
            
            // Try to decode as account data
            if address_like_keys > 0 {
                println!("\n🧪 Attempting to decode as PlainAccount:");
                let iter = db.iter(&txn)?;
                
                for (i, result) in iter.enumerate() {
                    if i >= 3 { break; }
                    
                    match result {
                        Ok((key, value)) if key.len() == 20 => {
                            println!("\n  Address: 0x{}", hex::encode(key));
                            println!("  Value length: {} bytes", value.len());
                            println!("  Value (hex): {}", hex::encode(&value[..value.len().min(64)]));
                            
                            // Try RLP decode
                            if let Ok(decoded) = try_decode_account(value) {
                                println!("  ✓ Decoded: {}", decoded);
                            } else {
                                println!("  ✗ Failed to decode as PlainAccount");
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(None) => {
            println!("⚠️  Unnamed database is empty");
        }
        Err(e) => {
            println!("✗ Error opening unnamed database: {}", e);
        }
    }

    Ok(())
}

fn try_decode_account(data: &[u8]) -> Result<String, String> {
    // Simple heuristic decode
    if data.is_empty() {
        return Err("Empty data".to_string());
    }
    
    // Check if it's RLP encoded
    if data[0] >= 0xc0 {
        Ok(format!("RLP list with {} bytes", data.len()))
    } else {
        Err("Not RLP encoded".to_string())
    }
}
