//! bench --workload S2|S3|S4 --input FILE [--emit | --warmup W --iterations N]
use std::fs;
use std::time::Instant;

use richrs::prelude::*;
use richrs::segment::{Control, ControlType};
use richrs::table::Row;

fn console() -> Console {
    let mut console = Console::new();
    console.set_width(100);
    console.options_mut().highlight = false;
    console.options_mut().emoji = false;
    console
}

fn s2(rows: &[Vec<String>]) -> Result<()> {
    let mut console = console();
    let header = Style::parse("bold cyan")?;
    let mut table = Table::new().title("Results");
    for (name, justify) in [
        ("Id", Justify::Right),
        ("Name", Justify::Left),
        ("Status", Justify::Left),
        ("Value", Justify::Right),
        ("Note", Justify::Left),
    ] {
        table.add_column(Column::new(name).header_style(header.clone()).justify(justify));
    }
    for r in rows {
        let color = match r[2].as_str() {
            "ok" => "green",
            "warn" => "yellow",
            "fail" => "red",
            _ => "dim",
        };
        table.add_row(Row::new([
            Text::from(r[0].as_str()),
            Text::from(r[1].as_str()),
            Text::styled(r[2].as_str(), Style::parse(color)?),
            Text::from(r[3].as_str()),
            Text::from(r[4].as_str()),
        ]));
    }
    console.write_segments(&table.render(console.width()))?;
    console.flush()
}

fn s3() -> Result<()> {
    let mut console = console();
    let mut progress = Progress::new().bar(ProgressBar::new().width(30));
    let ids: Vec<_> = (0..8)
        .map(|i| progress.add_task(format!("task {i}"), Some(12_500), true))
        .collect();
    for k in 0..100_000usize {
        progress.advance(ids[k % 8], 1)?;
        if k % 100 == 99 {
            if k > 99 {
                let up = Control::with_params(ControlType::CursorUp, vec![8]);
                console.write_segment(&Segment::control(up))?;
            }
            console.write_segments(&progress.render(console.width()))?;
            console.flush()?;
        }
    }
    Ok(())
}

fn s4(lines: &[String]) -> Result<()> {
    let mut console = console();
    for line in lines {
        console.print(line)?;
    }
    Ok(())
}

fn run(workload: &str, rows: &[Vec<String>], lines: &[String]) -> Result<()> {
    match workload {
        "S2" => s2(rows),
        "S3" => s3(),
        _ => s4(lines),
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    let workload = arg("--workload").expect("--workload").as_str();
    let text = arg("--input").map(|p| fs::read_to_string(p).expect("input")).unwrap_or_default();
    let rows: Vec<Vec<String>> = if workload == "S2" {
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split('\t').map(str::to_owned).collect())
            .collect()
    } else {
        Vec::new()
    };
    let lines: Vec<String> = if workload == "S4" { text.lines().map(str::to_owned).collect() } else { Vec::new() };
    if args.iter().any(|a| a == "--emit") {
        return run(workload, &rows, &lines);
    }
    let warmup: usize = arg("--warmup").and_then(|v| v.parse().ok()).unwrap_or(5);
    let iterations: usize = arg("--iterations").and_then(|v| v.parse().ok()).unwrap_or(30);
    for _ in 0..warmup {
        run(workload, &rows, &lines)?;
    }
    for i in 0..iterations {
        let start = Instant::now();
        run(workload, &rows, &lines)?;
        eprintln!("{{\"iter\": {i}, \"ns\": {}}}", start.elapsed().as_nanos());
    }
    Ok(())
}
