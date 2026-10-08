// bench --workload S2|S3|S4 [--input FILE] [--emit | --warmup W --iterations N]  (spec/speed.md)
// hud claims S4 in v0.2; S2 needs Table (v0.3) and S3 needs Progress (v0.5), so they exit 2.
use std::io::Write;
use std::time::Instant;

use hud::Console;

fn s4(lines: &[String]) {
    let console = Console::stdout();
    for line in lines {
        console.print(line.as_str());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].clone());
    let workload = get("--workload").expect("--workload");
    if workload != "S4" {
        eprintln!("hud does not implement {workload} yet");
        std::process::exit(2);
    }
    let emit = args.iter().any(|a| a == "--emit");
    let warmup: usize = get("--warmup").map_or(5, |v| v.parse().unwrap());
    let iterations: usize = get("--iterations").map_or(30, |v| v.parse().unwrap());
    let text = std::fs::read_to_string(get("--input").expect("--input")).unwrap();
    let lines: Vec<String> = text.lines().map(str::to_string).collect();

    if emit {
        s4(&lines);
        return;
    }
    for _ in 0..warmup {
        s4(&lines);
    }
    for i in 0..iterations {
        let t0 = Instant::now();
        s4(&lines);
        std::io::stdout().flush().unwrap();
        let ns = t0.elapsed().as_nanos();
        eprintln!("{{\"iter\": {i}, \"ns\": {ns}}}");
    }
}
