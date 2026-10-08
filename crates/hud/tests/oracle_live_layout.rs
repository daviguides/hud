#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Differential tests against Python Rich for Columns, Layout, Live and Progress on Live: vectors
//! rendered by the pinned Rich (`bench/scripts/gen_oracle_vectors_live_layout.py`) must come out
//! byte for byte the same. The nodes are built by the bench adapter, so the oracle and the corpus
//! gate exercise the same construction code.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

#[allow(dead_code)]
#[path = "../../../bench/candidates/hud/src/bin/live_layout_runner.rs"]
mod runner;

fn vectors(name: &str) -> Vec<Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// The vectors that differ: the case and what Rich wrote against what hud wrote.
fn mismatches(name: &str) -> (usize, Vec<(Value, String, String)>) {
    let rows = vectors(name);
    let bad = rows
        .iter()
        .filter_map(|row| {
            let case = &row["case"];
            let want = row["out"].as_str().unwrap();
            let got = runner::render(case).unwrap_or_else(|| panic!("{} is not supported", case["id"]));
            (got != want).then(|| (case.clone(), want.to_string(), got))
        })
        .collect();
    (rows.len(), bad)
}

fn assert_all_match(name: &str) {
    let (total, bad) = mismatches(name);
    assert!(total >= 1000, "{name} has only {total} vectors");
    assert!(
        bad.is_empty(),
        "{} of {total} vectors differ, first:\n{}",
        bad.len(),
        bad.iter()
            .take(3)
            .map(|(case, want, got)| format!("{}\n  want {want:?}\n  got  {got:?}", case["id"]))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A flag or a combining mark in the case, which D-003 and D-024 say Rich measures differently.
fn explained(case: &Value) -> bool {
    case.to_string()
        .chars()
        .any(|c| c == '\u{301}' || ('\u{1F1E6}'..='\u{1F1FF}').contains(&c))
}

/// D-024: a wide character where Rich ellipsizes a word that exactly fits, so every differing
/// line has more ellipses in Rich's output than in hud's.
fn explained_by_wide_characters(case: &Value, want: &str, got: &str) -> bool {
    let wide = case
        .to_string()
        .chars()
        .any(|c| hud::cell_width(c.encode_utf8(&mut [0; 4])) > 1);
    wide && want
        .lines()
        .zip(got.lines())
        .filter(|(a, b)| a != b)
        .all(|(a, b)| ellipses(&without_escapes(a)) > ellipses(&without_escapes(b)))
}

fn ellipses(line: &str) -> usize {
    line.matches('…').count()
}

fn without_escapes(line: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in line.chars() {
        match (in_escape, c) {
            (false, '\u{1b}') => in_escape = true,
            (true, 'm') => in_escape = false,
            (false, _) => out.push(c),
            (true, _) => {}
        }
    }
    out
}

fn assert_only_explained_differences(name: &str) {
    let (total, bad) = mismatches(name);
    let unexplained: Vec<_> = bad
        .iter()
        .filter(|(case, want, got)| !explained(case) && !explained_by_wide_characters(case, want, got))
        .collect();
    eprintln!(
        "{name}: {total} vectors, {} differ ({} neither a flag or combining mark nor a wide character)",
        bad.len(),
        unexplained.len()
    );
    assert!(
        unexplained.is_empty(),
        "{} unexplained, first:\n{}",
        unexplained.len(),
        unexplained
            .iter()
            .take(4)
            .map(|(case, want, got)| format!("{}\n  want {want:?}\n  got  {got:?}", case["id"]))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn ascii_columns_match_rich() {
    assert_all_match("columns_ascii.jsonl");
}

#[test]
fn ascii_layouts_match_rich() {
    assert_all_match("layout_ascii.jsonl");
}

#[test]
fn ascii_live_displays_match_rich() {
    assert_all_match("live_ascii.jsonl");
}

#[test]
fn ascii_progress_on_live_matches_rich() {
    assert_all_match("progress_live_ascii.jsonl");
}

#[test]
fn unicode_columns_differ_from_rich_only_where_the_deviations_say_so() {
    assert_only_explained_differences("columns_unicode.jsonl");
}

#[test]
fn unicode_layouts_differ_from_rich_only_where_the_deviations_say_so() {
    assert_only_explained_differences("layout_unicode.jsonl");
}

#[test]
fn unicode_live_displays_differ_from_rich_only_where_the_deviations_say_so() {
    assert_only_explained_differences("live_unicode.jsonl");
}

#[test]
fn unicode_progress_on_live_differs_from_rich_only_where_the_deviations_say_so() {
    assert_only_explained_differences("progress_live_unicode.jsonl");
}
