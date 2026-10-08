use std::io::{BufWriter, Write};
use std::time::Instant;

use hud_bench_rich_rust_adapters::Frame;
use rich_rust::prelude::*;

struct Args {
    workload: String,
    input: Option<String>,
    emit: bool,
    warmup: usize,
    iterations: usize,
    detect_size: bool,
    buffered: bool,
}

fn parse() -> Args {
    let mut a = Args { workload: String::new(), input: None, emit: false, warmup: 5, iterations: 30, detect_size: false, buffered: false };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--workload" => a.workload = it.next().unwrap(),
            "--input" => a.input = it.next(),
            "--emit" => a.emit = true,
            "--warmup" => a.warmup = it.next().unwrap().parse().unwrap(),
            "--iterations" => a.iterations = it.next().unwrap().parse().unwrap(),
            "--detect-size" => a.detect_size = true,
            "--buffered" => a.buffered = true,
            other => panic!("unknown flag {other}"),
        }
    }
    a
}

fn console(a: &Args) -> Console {
    let mut builder = Console::builder().highlight(false);
    if !a.detect_size {
        let columns = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(100);
        builder = builder.width(columns).height(24);
    }
    if a.buffered {
        builder = builder.file(Box::new(BufWriter::with_capacity(1 << 16, std::io::stdout())));
    }
    builder.build()
}

fn s2(a: &Args, rows: &[Vec<String>]) {
    let console = console(a);
    let mut table = Table::new()
        .title("Results")
        .header_style(Style::parse("bold cyan").unwrap_or_default())
        .with_column(Column::new("Id").justify(JustifyMethod::Right))
        .with_column(Column::new("Name"))
        .with_column(Column::new("Status"))
        .with_column(Column::new("Value").justify(JustifyMethod::Right))
        .with_column(Column::new("Note"));
    for r in rows {
        let color = match r[2].as_str() {
            "ok" => "green",
            "warn" => "yellow",
            "fail" => "red",
            _ => "dim",
        };
        table.add_row_markup([r[0].clone(), r[1].clone(), format!("[{color}]{}[/]", r[2]), r[3].clone(), r[4].clone()]);
    }
    console.print_renderable(&table);
}

fn s3(a: &Args) {
    let console = console(a).shared();
    let live = Live::new(console.clone());
    let (bar_complete, bar_back, bar_pulse) =
        (console.get_style("bar.complete"), console.get_style("bar.back"), console.get_style("bar.pulse"));
    let mut done = [0u64; 8];
    let frame = |done: &[u64; 8]| Frame {
        rows: done.iter().enumerate().map(|(i, d)| (format!("task {i}"), 12_500, *d)).collect(),
        bar_width: 30,
        bar_complete: bar_complete.clone(),
        bar_back: bar_back.clone(),
        bar_pulse: bar_pulse.clone(),
    };
    let live = live.renderable(frame(&done));
    live.start(true).unwrap();
    for k in 0..100_000usize {
        done[k % 8] += 1;
        if k % 100 == 99 {
            live.update(frame(&done), true);
        }
    }
    live.stop().unwrap();
}

fn s4(a: &Args, lines: &[String]) {
    let console = console(a);
    for line in lines {
        console.print(line);
    }
}

fn main() {
    let a = parse();
    let text = a.input.as_ref().map(|p| std::fs::read_to_string(p).unwrap());
    let rows: Vec<Vec<String>> = match (a.workload.as_str(), &text) {
        ("S2", Some(t)) => t.lines().filter(|l| !l.trim().is_empty()).map(|l| l.split('\t').map(str::to_string).collect()).collect(),
        _ => Vec::new(),
    };
    let lines: Vec<String> = match (a.workload.as_str(), &text) {
        ("S4", Some(t)) => t.lines().map(str::to_string).collect(),
        _ => Vec::new(),
    };
    let run = |a: &Args| match a.workload.as_str() {
        "S2" => s2(a, &rows),
        "S3" => s3(a),
        "S4" => s4(a, &lines),
        other => panic!("workload {other}"),
    };
    if a.emit {
        run(&a);
        std::io::stdout().flush().unwrap();
        return;
    }
    for _ in 0..a.warmup {
        run(&a);
        std::io::stdout().flush().unwrap();
    }
    for i in 0..a.iterations {
        let t0 = Instant::now();
        run(&a);
        std::io::stdout().flush().unwrap();
        eprintln!("{{\"iter\": {i}, \"ns\": {}}}", t0.elapsed().as_nanos());
    }
}
