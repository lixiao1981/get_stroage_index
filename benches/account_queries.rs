//! Benchmark for account query performance
//!
//! Run with: cargo bench
//!
//! Note: Requires a real Erigon database to benchmark against.
//! Set environment variable ERIGON_DB_PATH to your database path.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use alloy_primitives::{Address, U256};
use std::str::FromStr;

fn account_query_benchmark(c: &mut Criterion) {
    // Skip benchmarks if no database path is provided
    let db_path = match std::env::var("ERIGON_DB_PATH") {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Skipping benchmarks: ERIGON_DB_PATH not set");
            eprintln!("Set it to run benchmarks: export ERIGON_DB_PATH=/path/to/chaindata");
            return;
        }
    };

    // Initialize reader
    let reader = match get_stroage_index::StateReader::open(&db_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            return;
        }
    };

    // Test addresses (mix of EOA and contracts)
    let addresses = vec![
        Address::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap(), // Vitalik
        Address::from_str("0x0000000000000000000000000000000000000000").unwrap(), // Zero
        Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(), // WETH
    ];

    // Benchmark single account queries
    let mut group = c.benchmark_group("single_account_query");
    for (i, addr) in addresses.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::from_parameter(i),
            addr,
            |b, addr| {
                b.iter(|| {
                    let _ = reader.get_account(black_box(*addr));
                });
            },
        );
    }
    group.finish();

    // Benchmark balance queries
    c.bench_function("get_balance", |b| {
        b.iter(|| {
            let _ = reader.get_balance(black_box(addresses[0]));
        });
    });

    // Benchmark nonce queries
    c.bench_function("get_nonce", |b| {
        b.iter(|| {
            let _ = reader.get_nonce(black_box(addresses[0]));
        });
    });

    // Benchmark contract check
    c.bench_function("is_contract", |b| {
        b.iter(|| {
            let _ = reader.is_contract(black_box(addresses[2]));
        });
    });

    // Benchmark batch iteration
    if let Ok(mut iter) = reader.iter_accounts(100) {
        c.bench_function("batch_iterate_100", |b| {
            b.iter(|| {
                iter.reset();
                let _ = iter.next_batch();
            });
        });
    }

    // Benchmark storage read (if WETH contract exists)
    c.bench_function("storage_read", |b| {
        let weth = addresses[2];
        let slot = U256::ZERO;
        b.iter(|| {
            let _ = reader.get_storage(black_box(weth), black_box(slot));
        });
    });
}

criterion_group!(benches, account_query_benchmark);
criterion_main!(benches);
