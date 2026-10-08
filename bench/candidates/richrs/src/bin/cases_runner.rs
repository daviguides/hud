//! cases-runner <cases.jsonl> <outdir>
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    bench_richrs::run_cases(Path::new(&args[1]), Path::new(&args[2]));
}
