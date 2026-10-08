//! width-runner <outdir>: width.jsonl, truncate.jsonl and tables/ from richrs' public API.
//! No fold.jsonl: richrs has no public fold/wrap primitive (Text::truncate is the only cut).
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use richrs::measure::cell_len;
use richrs::text::Text;
use serde_json::{Value, json};

fn main() {
    let out = Path::new(&std::env::args().nth(1).expect("outdir")).to_path_buf();
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cases");
    fs::create_dir_all(&out).expect("outdir");
    let corpus = fs::read_to_string(cases.join("width_corpus.jsonl")).expect("corpus");
    let mut widths = BufWriter::new(File::create(out.join("width.jsonl")).expect("width"));
    let mut truncs = BufWriter::new(File::create(out.join("truncate.jsonl")).expect("truncate"));
    for line in corpus.lines() {
        let item: Value = serde_json::from_str(line).expect("json");
        let (id, text) = (item["id"].as_str().expect("id"), item["text"].as_str().expect("text"));
        writeln!(widths, "{}", json!({"id": id, "width": cell_len(text)})).expect("write");
        for w in 1..=40usize {
            let cut = Text::from(text).truncate(w, None);
            writeln!(truncs, "{}", json!({"id": id, "w": w, "text": cut.plain()})).expect("write");
        }
    }
    bench_richrs::run_cases(&cases.join("table_unicode.jsonl"), &out.join("tables"));
}
