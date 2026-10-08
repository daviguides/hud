use std::fs;
use std::path::PathBuf;

use hud_bench_rich_rust_adapters::render_case;
use rich_rust::cells::{cell_len, chop_cells};
use serde_json::{Value, json};

fn fold(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let (head, tail) = chop_cells(rest, width);
        if head.is_empty() {
            let first = rest.chars().next().unwrap().len_utf8();
            lines.push(rest[..first].to_string());
            rest = &rest[first..];
        } else {
            lines.push(head.to_string());
            rest = tail;
        }
    }
    lines
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let outdir = PathBuf::from(&args[1]);
    let bench = PathBuf::from(&args[2]);
    fs::create_dir_all(outdir.join("tables")).unwrap();
    let (mut widths, mut folds, mut truncs) = (String::new(), String::new(), String::new());
    let corpus = fs::read_to_string(bench.join("cases/width_corpus.jsonl")).unwrap();
    for line in corpus.lines().filter(|l| !l.trim().is_empty()) {
        let item: Value = serde_json::from_str(line).unwrap();
        let (id, text) = (item["id"].as_str().unwrap(), item["text"].as_str().unwrap());
        widths.push_str(&format!("{}\n", json!({"id": id, "width": cell_len(text)})));
        for w in 1..=40usize {
            let lines = fold(text, w);
            let first = lines.first().cloned().unwrap_or_default();
            folds.push_str(&format!("{}\n", json!({"id": id, "w": w, "lines": lines})));
            truncs.push_str(&format!("{}\n", json!({"id": id, "w": w, "text": first})));
        }
    }
    fs::write(outdir.join("width.jsonl"), widths).unwrap();
    fs::write(outdir.join("fold.jsonl"), folds).unwrap();
    fs::write(outdir.join("truncate.jsonl"), truncs).unwrap();
    let tables = fs::read_to_string(bench.join("cases/table_unicode.jsonl")).unwrap();
    for line in tables.lines().filter(|l| !l.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).unwrap();
        if let Ok(bytes) = render_case(&case) {
            fs::write(outdir.join("tables").join(format!("{}.ansi", case["id"].as_str().unwrap())), bytes).unwrap();
        }
    }
}
