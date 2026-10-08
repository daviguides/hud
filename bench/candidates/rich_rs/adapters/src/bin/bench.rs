use std::{
    fs,
    io::{Write, stderr},
    time::Instant,
};

use rich_rs::{
    BarColumn, Column, Console, JustifyMethod, LiveOptions, MofNCompleteColumn, Progress,
    ProgressColumn, Row, Style, Table, TaskProgressColumn, Text, TextColumn,
};

fn console() -> Console {
    let mut console = Console::new();
    let cols = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(100);
    console.set_size(cols, 24);
    if std::env::var_os("FORCE_COLOR").is_some() {
        console.set_force_terminal(Some(true));
    }
    console
}

fn s2(rows: &[Vec<String>]) {
    let mut console = console();
    let mut table = Table::new()
        .with_title("Results")
        .with_title_style(Style::parse("italic").unwrap_or_default())
        .with_header_style(Style::parse("bold cyan").unwrap_or_default());
    for (name, justify) in [
        ("Id", JustifyMethod::Right),
        ("Name", JustifyMethod::Left),
        ("Status", JustifyMethod::Left),
        ("Value", JustifyMethod::Right),
        ("Note", JustifyMethod::Left),
    ] {
        table.add_column(Column::with_header_str(name).justify(justify));
    }
    let ok = Style::parse("green").unwrap_or_default();
    let warn = Style::parse("yellow").unwrap_or_default();
    let fail = Style::parse("red").unwrap_or_default();
    let skip = Style::parse("dim").unwrap_or_default();
    for r in rows {
        let style = match r[2].as_str() {
            "ok" => ok,
            "warn" => warn,
            "fail" => fail,
            _ => skip,
        };
        table.add_row(Row::new(vec![
            Box::new(Text::plain(&r[0])),
            Box::new(Text::plain(&r[1])),
            Box::new(Text::styled(&r[2], style)),
            Box::new(Text::plain(&r[3])),
            Box::new(Text::plain(&r[4])),
        ]));
    }
    console.print(&table, None, None, None, false, "").unwrap();
}

fn s3() {
    let columns: Vec<Box<dyn ProgressColumn>> = vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::new().with_bar_width(Some(30))),
        Box::new(TaskProgressColumn::new(false)),
        Box::new(MofNCompleteColumn::new()),
    ];
    let live = LiveOptions { auto_refresh: false, ..LiveOptions::default() };
    let mut progress = Progress::with_console(columns, console(), live, false, false);
    progress.start().unwrap();
    let ids: Vec<_> = (0..8)
        .map(|i| progress.add_task(&format!("task {i}"), true, Some(12_500.0), 0.0, true))
        .collect();
    for k in 0..100_000usize {
        progress.advance(ids[k % 8], 1.0);
        if k % 100 == 99 {
            progress.refresh().unwrap();
        }
    }
    progress.stop().unwrap();
}

fn s4(lines: &[String]) {
    let mut console = console();
    for line in lines {
        let text = Text::from_markup(line, false).unwrap();
        console.print(&text, None, None, None, false, "\n").unwrap();
    }
}

enum Data {
    Rows(Vec<Vec<String>>),
    Lines(Vec<String>),
    None,
}

fn run(workload: &str, data: &Data) {
    match (workload, data) {
        ("S2", Data::Rows(rows)) => s2(rows),
        ("S3", _) => s3(),
        ("S4", Data::Lines(lines)) => s4(lines),
        _ => panic!("bad workload"),
    }
    std::io::stdout().flush().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].clone());
    let workload = value("--workload").expect("--workload");
    let data = match (workload.as_str(), value("--input")) {
        ("S2", Some(path)) => Data::Rows(
            fs::read_to_string(path)
                .unwrap()
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.split('\t').map(str::to_string).collect())
                .collect(),
        ),
        ("S4", Some(path)) => {
            Data::Lines(fs::read_to_string(path).unwrap().lines().map(str::to_string).collect())
        }
        _ => Data::None,
    };
    if args.iter().any(|a| a == "--emit") {
        run(&workload, &data);
        return;
    }
    let warmup: usize = value("--warmup").map_or(5, |v| v.parse().unwrap());
    let iterations: usize = value("--iterations").map_or(30, |v| v.parse().unwrap());
    for _ in 0..warmup {
        run(&workload, &data);
    }
    for i in 0..iterations {
        let t0 = Instant::now();
        run(&workload, &data);
        let ns = t0.elapsed().as_nanos();
        writeln!(stderr(), "{{\"iter\": {i}, \"ns\": {ns}}}").unwrap();
    }
}
