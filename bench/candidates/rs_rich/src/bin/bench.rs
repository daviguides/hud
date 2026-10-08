// bench --workload S2|S3|S4 [--input FILE] [--emit | --warmup W --iterations N]  (spec/speed.md)
use std::io::Write;
use std::time::Instant;

use rich::{BarColumn, Console, Justify, Progress, ProgressColumn, Table, TextColumn};

fn console() -> Console {
    let mut builder = Console::builder().highlight(false);
    if std::env::var_os("FORCE_COLOR").is_some() {
        builder = builder.force_terminal(true);
    }
    builder.build()
}

fn s2(rows: &[Vec<String>]) {
    let mut table = Table::new().title("Results").header_style("bold cyan");
    table.add_column_justify("Id", Justify::Right);
    table.add_column("Name");
    table.add_column("Status");
    table.add_column_justify("Value", Justify::Right);
    table.add_column("Note");
    for r in rows {
        let color = match r[2].as_str() {
            "ok" => "green",
            "warn" => "yellow",
            "fail" => "red",
            _ => "dim",
        };
        table.add_row(&[&r[0], &r[1], &format!("[{color}]{}[/]", r[2]), &r[3], &r[4]]);
    }
    console().print(&table);
}

fn s3() {
    let progress = Progress::new().columns(vec![
        ProgressColumn::TextFormat(TextColumn::new("{task.description}")),
        ProgressColumn::BarWith(BarColumn::new().bar_width(Some(30))),
        ProgressColumn::Percentage,
        ProgressColumn::MofN,
    ]);
    let live = progress.start(console(), std::io::stdout(), 1.0);
    let ids: Vec<_> = (0..8)
        .map(|i| live.add_task(format!("task {i}"), 12_500.0, 0.0))
        .collect();
    for k in 0..100_000usize {
        live.advance(ids[k % 8], 1.0);
        if k % 100 == 99 {
            live.refresh();
        }
    }
    live.stop();
}

fn s4(lines: &[String]) {
    let console = console();
    for line in lines {
        console.print_str(line);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args[i + 1].clone())
    };
    let workload = get("--workload").expect("--workload");
    let input = get("--input");
    let emit = args.iter().any(|a| a == "--emit");
    let warmup: usize = get("--warmup").map_or(5, |v| v.parse().unwrap());
    let iterations: usize = get("--iterations").map_or(30, |v| v.parse().unwrap());

    let text = input
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_default();
    let rows: Vec<Vec<String>> = if workload == "S2" {
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split('\t').map(str::to_string).collect())
            .collect()
    } else {
        Vec::new()
    };
    let lines: Vec<String> = if workload == "S4" {
        text.lines().map(str::to_string).collect()
    } else {
        Vec::new()
    };
    let run = || match workload.as_str() {
        "S2" => s2(&rows),
        "S3" => s3(),
        "S4" => s4(&lines),
        other => panic!("unknown workload {other}"),
    };

    if emit {
        run();
        std::io::stdout().flush().unwrap();
        return;
    }
    for _ in 0..warmup {
        run();
        std::io::stdout().flush().unwrap();
    }
    for i in 0..iterations {
        let t0 = Instant::now();
        run();
        std::io::stdout().flush().unwrap();
        let ns = t0.elapsed().as_nanos();
        eprintln!("{{\"iter\": {i}, \"ns\": {ns}}}");
    }
}
