use std::{fs, io::Write, path::Path};

use hud_bench_rich_rs_adapters::render_case;
use rich_rs::{OverflowMethod, Text, cell_len, chop_cells};
use serde_json::{Value, json};

const BENCH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");

fn jsonl(path: &str) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().expect("usage: width-runner <outdir> [--truncate-via chop]");
    let via_chop = args.next().as_deref() == Some("--truncate-via");
    let out = Path::new(&out);
    fs::create_dir_all(out.join("tables")).unwrap();

    let mut width = fs::File::create(out.join("width.jsonl")).unwrap();
    let mut fold = fs::File::create(out.join("fold.jsonl")).unwrap();
    let mut truncate = fs::File::create(out.join("truncate.jsonl")).unwrap();
    for item in jsonl(&format!("{BENCH}/cases/width_corpus.jsonl")) {
        let id = item["id"].as_str().unwrap();
        let text = item["text"].as_str().unwrap();
        writeln!(width, "{}", json!({"id": id, "width": cell_len(text)})).unwrap();
        for w in 1..=40usize {
            let lines = chop_cells(text, w);
            let cut = if via_chop {
                lines.first().cloned().unwrap_or_default()
            } else {
                Text::plain(text).truncate(w, OverflowMethod::Crop, false).plain_text().to_string()
            };
            writeln!(fold, "{}", json!({"id": id, "w": w, "lines": lines})).unwrap();
            writeln!(truncate, "{}", json!({"id": id, "w": w, "text": cut})).unwrap();
        }
    }

    for case in jsonl(&format!("{BENCH}/cases/table_unicode.jsonl")) {
        let id = case["id"].as_str().unwrap();
        let bytes = render_case(&case).unwrap_or_else(|e| format!("<error: {e}>").into_bytes());
        fs::write(out.join("tables").join(format!("{id}.ansi")), bytes).unwrap();
    }
}
