// width-runner <outdir>: width.jsonl, fold.jsonl, truncate.jsonl (+ truncate_cells_fn.jsonl variant) and tables/
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use rich::cells::{cell_len, chop_cells, truncate};
use serde_json::{Value, json};

fn main() {
    let outdir = PathBuf::from(std::env::args().nth(1).expect("outdir"));
    fs::create_dir_all(&outdir).unwrap();
    let bench = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus = fs::read_to_string(bench.join("cases/width_corpus.jsonl")).unwrap();

    let mut width = fs::File::create(outdir.join("width.jsonl")).unwrap();
    let mut fold = fs::File::create(outdir.join("fold.jsonl")).unwrap();
    let mut trunc = fs::File::create(outdir.join("truncate.jsonl")).unwrap();
    let mut trunc_fn = fs::File::create(outdir.join("truncate_cells_fn.jsonl")).unwrap();
    for line in corpus.lines().filter(|l| !l.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap();
        let text = case["text"].as_str().unwrap();
        writeln!(width, "{}", json!({"id": id, "width": cell_len(text)})).unwrap();
        for w in 1..=40usize {
            let lines = chop_cells(text, w);
            let first = lines.first().cloned().unwrap_or_default();
            writeln!(fold, "{}", json!({"id": id, "w": w, "lines": lines})).unwrap();
            writeln!(trunc, "{}", json!({"id": id, "w": w, "text": first})).unwrap();
            writeln!(
                trunc_fn,
                "{}",
                json!({"id": id, "w": w, "text": truncate(text, w)})
            )
            .unwrap();
        }
    }

    let runner = std::env::current_exe()
        .unwrap()
        .with_file_name("cases_runner");
    let status = Command::new(runner)
        .arg(bench.join("cases/table_unicode.jsonl"))
        .arg(outdir.join("tables"))
        .status()
        .unwrap();
    assert!(status.success());
}
