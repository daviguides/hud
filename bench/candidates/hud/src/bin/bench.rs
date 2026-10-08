// bench --workload S2|S3|S4 [--input FILE] [--emit | --warmup W --iterations N]  (spec/speed.md)
// hud claims S4 (v0.2) and S2 (v0.3); S3 needs Progress (v0.5), so it exits 2.
use std::io::Write;
use std::time::Instant;

use hud::{Color, Column, Console, Justify, Style, Table};

fn s4(lines: &[String]) {
    let console = Console::stdout();
    for line in lines {
        console.print(line.as_str());
    }
}

fn s2(rows: &[Vec<String>]) {
    let mut table = Table::new()
        .title("Results")
        .header_style(Style::new().bold().color(Color::CYAN))
        .column(Column::new("Id").justify(Justify::Right))
        .column("Name")
        .column("Status")
        .column(Column::new("Value").justify(Justify::Right))
        .column("Note");
    for row in rows {
        let color = if row[2] == "skip" { "dim" } else { match row[2].as_str() { "ok" => "green", "warn" => "yellow", _ => "red" } };
        table.add_row([
            row[0].as_str(),
            row[1].as_str(),
            &format!("[{color}]{}[/]", row[2]),
            row[3].as_str(),
            row[4].as_str(),
        ]);
    }
    Console::stdout().print(&table);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].clone());
    let workload = get("--workload").expect("--workload");
    if workload != "S4" && workload != "S2" {
        eprintln!("hud does not implement {workload} yet");
        std::process::exit(2);
    }
    let emit = args.iter().any(|a| a == "--emit");
    let warmup: usize = get("--warmup").map_or(5, |v| v.parse().unwrap());
    let iterations: usize = get("--iterations").map_or(30, |v| v.parse().unwrap());
    let text = std::fs::read_to_string(get("--input").expect("--input")).unwrap();
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let rows: Vec<Vec<String>> = if workload == "S2" {
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split('\t').map(str::to_string).collect())
            .collect()
    } else {
        Vec::new()
    };
    let run = |lines: &[String], rows: &[Vec<String>]| {
        if workload == "S2" {
            s2(rows);
        } else {
            s4(lines);
        }
    };

    if emit {
        run(&lines, &rows);
        return;
    }
    for _ in 0..warmup {
        run(&lines, &rows);
    }
    for i in 0..iterations {
        let t0 = Instant::now();
        run(&lines, &rows);
        std::io::stdout().flush().unwrap();
        let ns = t0.elapsed().as_nanos();
        eprintln!("{{\"iter\": {i}, \"ns\": {ns}}}");
    }
}
