//! The bench corpus: 500 width strings against the Rich reference, and fold/truncate
//! invariants over every string and every width from 1 to 40.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use hud_width::{cell_width, cell_width_of_cluster, clusters, fold, truncate};

fn bench(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench")
        .join(path)
}

fn lines(path: &str) -> Vec<serde_json::Value> {
    fs::read_to_string(bench(path))
        .expect("bench file")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("json line"))
        .collect()
}

/// Corpus strings where hud deliberately differs from the Rich reference.
/// Each is listed in DEVIATIONS.md (ZWJ between non-pictographic characters).
const DEVIATIONS: [&str; 4] = ["w-0348", "w-0356", "w-0358", "w-0360"];

#[test]
fn width_agrees_with_the_reference_except_for_listed_deviations() {
    let reference: HashMap<String, u64> = lines("golden/width/width_ref.jsonl")
        .into_iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap().to_string(),
                r["ref"].as_u64().unwrap(),
            )
        })
        .collect();
    let corpus = lines("cases/width_corpus.jsonl");
    assert_eq!(corpus.len(), 500);
    let mut disagreements = Vec::new();
    for row in &corpus {
        let id = row["id"].as_str().unwrap();
        let text = row["text"].as_str().unwrap();
        if cell_width(text) as u64 != reference[id] {
            disagreements.push(id.to_string());
        }
    }
    disagreements.sort();
    assert_eq!(
        disagreements,
        DEVIATIONS,
        "agreement {}/500",
        500 - disagreements.len()
    );
}

#[test]
fn fold_and_truncate_never_split_a_cluster() {
    let corpus = lines("cases/width_corpus.jsonl");
    let mut cases = 0;
    for row in &corpus {
        let text = row["text"].as_str().unwrap();
        let boundaries: Vec<usize> = clusters(text)
            .scan(0, |pos, c| {
                *pos += c.len();
                Some(*pos)
            })
            .collect();
        for width in 1..=40 {
            let lines: Vec<&str> = fold(text, width).collect();
            assert_eq!(lines.concat(), text, "concat, width {width}");
            let mut pos = 0;
            for line in &lines {
                pos += line.len();
                assert!(
                    pos == text.len() || boundaries.contains(&pos),
                    "cut inside a cluster at {pos}"
                );
                let cells = cell_width(line);
                let single_cluster = clusters(line).count() <= 1;
                assert!(
                    cells <= width || single_cluster,
                    "line {line:?} is {cells} cells for width {width}"
                );
            }
            let cut = truncate(text, width);
            assert_eq!(Some(&cut), lines.first());
            cases += 1;
        }
    }
    assert_eq!(cases, 20_000);
}

#[test]
fn cell_width_is_the_sum_of_cluster_widths() {
    for row in lines("cases/width_corpus.jsonl") {
        let text = row["text"].as_str().unwrap();
        let sum: usize = clusters(text).map(cell_width_of_cluster).sum();
        assert_eq!(cell_width(text), sum);
    }
}
