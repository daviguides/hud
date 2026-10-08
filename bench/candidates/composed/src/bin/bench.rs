use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, ContentLineStyle, LineStyle, Table,
    TableStyle,
};
use hud_bench_composed::style::Depth;
use hud_bench_composed::text::{parse_markup, render_text};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle, TermLike};
use std::io::{self, Write};
use std::time::Instant;

#[derive(Debug)]
struct StdoutTerm;

impl StdoutTerm {
    fn put(&self, s: &str) -> io::Result<()> {
        io::stdout().lock().write_all(s.as_bytes())
    }
}

impl TermLike for StdoutTerm {
    fn width(&self) -> u16 {
        100
    }
    fn move_cursor_up(&self, n: usize) -> io::Result<()> {
        self.put(&format!("\x1b[{n}A"))
    }
    fn move_cursor_down(&self, n: usize) -> io::Result<()> {
        self.put(&format!("\x1b[{n}B"))
    }
    fn move_cursor_right(&self, n: usize) -> io::Result<()> {
        self.put(&format!("\x1b[{n}C"))
    }
    fn move_cursor_left(&self, n: usize) -> io::Result<()> {
        self.put(&format!("\x1b[{n}D"))
    }
    fn write_line(&self, s: &str) -> io::Result<()> {
        self.put(&format!("{s}\n"))
    }
    fn write_str(&self, s: &str) -> io::Result<()> {
        self.put(s)
    }
    fn clear_line(&self) -> io::Result<()> {
        self.put("\r\x1b[2K")
    }
    fn flush(&self) -> io::Result<()> {
        io::stdout().flush()
    }
}

fn s2(rows: &[Vec<String>]) {
    let style = TableStyle::new()
        .top_border(LineStyle::new('┏', '━', '┳', '┓'))
        .header_lines(ContentLineStyle::new('┃', '┃', '┃'))
        .header_separator(LineStyle::new('┡', '━', '╇', '┩'))
        .content_lines(ContentLineStyle::new('│', '│', '│'))
        .bottom_border(LineStyle::new('└', '─', '┴', '┘'));
    let mut table = Table::new();
    table
        .load_style(style)
        .enforce_styling()
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100);
    table.set_header(
        ["Id", "Name", "Status", "Value", "Note"]
            .map(|h| Cell::new(h).add_attribute(Attribute::Bold).fg(Color::Cyan)),
    );
    for r in rows {
        let status = match r[2].as_str() {
            "ok" => Cell::new(&r[2]).fg(Color::Green),
            "warn" => Cell::new(&r[2]).fg(Color::Yellow),
            "fail" => Cell::new(&r[2]).fg(Color::Red),
            _ => Cell::new(&r[2]).add_attribute(Attribute::Dim),
        };
        table.add_row([
            Cell::new(&r[0]),
            Cell::new(&r[1]),
            status,
            Cell::new(&r[3]),
            Cell::new(&r[4]),
        ]);
    }
    for i in [0, 3] {
        table
            .column_mut(i)
            .unwrap()
            .set_cell_alignment(CellAlignment::Right);
    }
    let lines: Vec<String> = table.lines().collect();
    let out = io::stdout();
    let mut out = out.lock();
    let width = lines[0].chars().count();
    writeln!(out, "\x1b[3m{:^width$}\x1b[0m", "Results").unwrap();
    for line in lines {
        writeln!(out, "{line}").unwrap();
    }
}

fn s3() {
    let multi = MultiProgress::with_draw_target(ProgressDrawTarget::term_like_with_hz(
        Box::new(StdoutTerm),
        1,
    ));
    let style = ProgressStyle::with_template("{prefix:<6} {bar:30} {percent:>3}% {pos}/{len}")
        .unwrap()
        .progress_chars("━╸ ");
    let bars: Vec<ProgressBar> = (0..8)
        .map(|i| {
            multi.add(
                ProgressBar::new(12_500)
                    .with_style(style.clone())
                    .with_prefix(format!("task {i}")),
            )
        })
        .collect();
    for k in 0..100_000 {
        bars[k % 8].inc(1);
        if k % 100 == 99 {
            bars[0].force_draw();
        }
    }
    for bar in &bars {
        bar.finish();
    }
}

fn s4(lines: &[String]) {
    let out = io::stdout();
    let mut out = out.lock();
    for line in lines {
        for l in render_text(&parse_markup(line), 100, Depth::True) {
            writeln!(out, "{l}").unwrap();
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str| {
        args.iter()
            .position(|a| a == k)
            .map(|i| args[i + 1].clone())
    };
    let workload = get("--workload").unwrap();
    let input = get("--input")
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_default();
    let rows: Vec<Vec<String>> = input
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect();
    let lines: Vec<String> = input.lines().map(str::to_string).collect();
    let run = || match workload.as_str() {
        "S2" => s2(&rows),
        "S3" => s3(),
        _ => s4(&lines),
    };
    if args.iter().any(|a| a == "--emit") {
        run();
        io::stdout().flush().unwrap();
        return;
    }
    let warmup: usize = get("--warmup").map_or(5, |v| v.parse().unwrap());
    let iterations: usize = get("--iterations").map_or(30, |v| v.parse().unwrap());
    for _ in 0..warmup {
        run();
        io::stdout().flush().unwrap();
    }
    for i in 0..iterations {
        let t0 = Instant::now();
        run();
        io::stdout().flush().unwrap();
        eprintln!("{{\"iter\": {i}, \"ns\": {}}}", t0.elapsed().as_nanos());
    }
}
