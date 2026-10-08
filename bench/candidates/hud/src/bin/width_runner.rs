// width-runner <outdir>: width.jsonl, fold.jsonl, truncate.jsonl for the width corpus.
// Tables are not written: hud has no Table until v0.3, and a missing file means unsupported.
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use hud_width::{cell_width, fold, truncate};
use serde_json::{Value, json};

fn main() {
    let outdir = PathBuf::from(std::env::args().nth(1).expect("outdir"));
    fs::create_dir_all(&outdir).unwrap();
    let bench = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus = fs::read_to_string(bench.join("cases/width_corpus.jsonl")).unwrap();

    let mut width = fs::File::create(outdir.join("width.jsonl")).unwrap();
    let mut fold_out = fs::File::create(outdir.join("fold.jsonl")).unwrap();
    let mut trunc = fs::File::create(outdir.join("truncate.jsonl")).unwrap();
    for line in corpus.lines().filter(|l| !l.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap();
        let text = case["text"].as_str().unwrap();
        writeln!(width, "{}", json!({"id": id, "width": cell_width(text)})).unwrap();
        for w in 1..=40usize {
            let lines: Vec<&str> = fold(text, w).collect();
            writeln!(fold_out, "{}", json!({"id": id, "w": w, "lines": lines})).unwrap();
            writeln!(trunc, "{}", json!({"id": id, "w": w, "text": truncate(text, w)})).unwrap();
        }
    }
}
