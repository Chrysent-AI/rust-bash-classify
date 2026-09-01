// SPDX-FileCopyrightText: 2026 Chrysent LLC
// SPDX-License-Identifier: MIT

//! Bulk-classify a corpus of JSON-encoded bash command strings (one per line)
//! using the library API, printing only aggregate timings and tallies.
//!
//! Usage: cargo run --release --example bulk_classify -- /path/to/corpus.jsonl
//!
//! The corpus is never echoed; only counts are printed. Lines that are not
//! valid JSON strings are counted as decode errors.

use std::collections::BTreeMap;
use std::io::BufRead;
use std::time::Instant;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/allbash-commands".to_string());
    let file = std::fs::File::open(&path).unwrap_or_else(|e| {
        eprintln!("cannot open {path}: {e}");
        std::process::exit(2);
    });

    let db = bash_classify::database::load_database(None).unwrap_or_else(|e| {
        eprintln!("cannot load database: {e}");
        std::process::exit(2);
    });

    let mut classes: BTreeMap<String, u64> = BTreeMap::new();
    let mut risks: BTreeMap<String, u64> = BTreeMap::new();
    let (mut total, mut classified, mut decode_errors, mut empty, mut errors) =
        (0u64, 0u64, 0u64, 0u64, 0u64);

    let start = Instant::now();
    for line in std::io::BufReader::new(file).lines() {
        let line = line.unwrap_or_default();
        total += 1;
        let expr: String = match serde_json::from_str(line.trim_end_matches(',')) {
            Ok(v) => v,
            Err(_) => {
                decode_errors += 1;
                continue;
            }
        };
        if expr.trim().is_empty() {
            empty += 1;
            continue;
        }
        match bash_classify::classifier::classify_expression_with_database(&expr, &db) {
            Ok(result) => {
                classes
                    .entry(format!("{:?}", result.classification))
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                risks
                    .entry(format!("{:?}", result.risk))
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                classified += 1;
            }
            Err(_) => errors += 1,
        }
    }
    let elapsed = start.elapsed();

    println!(
        "{}",
        serde_json::json!({
            "total": total,
            "classified": classified,
            "decode_errors": decode_errors,
            "empty": empty,
            "errors": errors,
            "wall_s": elapsed.as_secs_f64(),
            "classes": classes,
            "risks": risks,
        })
    );
}
