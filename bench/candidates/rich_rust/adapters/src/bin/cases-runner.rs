use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use hud_bench_rich_rust_adapters::render_case;
use serde_json::Value;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (cases, outdir) = (&args[1], PathBuf::from(&args[2]));
    fs::create_dir_all(&outdir).unwrap();
    let mut log = String::new();
    for line in fs::read_to_string(cases).unwrap().lines().filter(|l| !l.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap();
        match catch_unwind(AssertUnwindSafe(|| render_case(&case))) {
            Ok(Ok(bytes)) => fs::write(outdir.join(format!("{id}.ansi")), bytes).unwrap(),
            Ok(Err(reason)) => {
                fs::write(outdir.join(format!("{id}.unsupported")), b"").unwrap();
                log.push_str(&format!("{id}\tunsupported\t{reason}\n"));
            }
            Err(_) => log.push_str(&format!("{id}\tpanic\n")),
        }
    }
    fs::write(outdir.join("runner.log"), log).unwrap();
}
