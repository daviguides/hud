use std::{fs, path::Path};

use hud_bench_rich_rs_adapters::render_case;
use serde_json::Value;

fn main() {
    let mut args = std::env::args().skip(1);
    let cases = args.next().expect("usage: cases-runner <cases.jsonl> <outdir>");
    let out = args.next().expect("usage: cases-runner <cases.jsonl> <outdir>");
    let out = Path::new(&out);
    fs::create_dir_all(out).unwrap();
    let mut errors = Vec::new();
    for line in fs::read_to_string(cases).unwrap().lines().filter(|l| !l.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap();
        let bytes = match std::panic::catch_unwind(|| render_case(&case)) {
            Ok(Ok(b)) => b,
            Ok(Err(e)) => {
                errors.push(serde_json::json!({"id": id, "error": e}));
                format!("<error: {e}>").into_bytes()
            }
            Err(_) => {
                errors.push(serde_json::json!({"id": id, "error": "panic"}));
                b"<error: panic>".to_vec()
            }
        };
        fs::write(out.join(format!("{id}.ansi")), bytes).unwrap();
    }
    fs::write(out.join("errors.json"), serde_json::to_string_pretty(&errors).unwrap()).unwrap();
    eprintln!("{} cases with errors", errors.len());
}
