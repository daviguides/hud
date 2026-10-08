// fuzz_live_layout [--seed S] [--count N]   (bench/scripts/fuzz.py drives it)
// Zero-panic check for Columns, Layout and Live: N seeded random inputs per feature, plus the
// properties that must hold on every input. Same protocol as fuzz.rs (one JSON line per feature,
// exit 1 on any panic or violated property); a separate binary because fuzz.rs belongs to the
// main line.
use std::io::Write;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use hud::{
    Align, Body, BoxStyle, ColorSystem, Column, Columns, Console, Layout, Live, Padding, Panel,
    Renderable, Table, Tree, VerticalOverflow, cell_width,
};
use serde_json::json;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
}

const FRAGMENTS: &[&str] = &[
    "[",
    "]",
    "[/]",
    "[bold]",
    "[/bold]",
    "[red on blue]",
    "\\[",
    "[b",
    "bold",
    " ",
    "  ",
    "\t",
    "\n",
    "x",
    "word",
    "alpha beta gamma",
    "supercalifragilisticexpialidocious",
    "0",
    "255",
];
const UNICODE: &[&str] = &[
    "你好",
    "😀",
    "👨\u{200d}👩\u{200d}👧",
    "🇧🇷",
    "e\u{301}",
    "한국어",
    "क्ष",
];

fn random_string(rng: &mut Rng) -> String {
    let mut out = String::new();
    for _ in 0..rng.below(14) {
        match rng.below(10) {
            0..=5 => out.push_str(rng.pick(FRAGMENTS)),
            6 | 7 => out.push_str(rng.pick(UNICODE)),
            8 => out.extend((0..rng.below(30)).map(|_| 'a')),
            _ => out.push_str(&rng.below(100_000).to_string()),
        }
    }
    out
}

fn console(rng: &mut Rng) -> Console {
    let system = [
        ColorSystem::None,
        ColorSystem::Standard,
        ColorSystem::EightBit,
        ColorSystem::TrueColor,
    ][rng.below(4)];
    Console::builder()
        .width([1, 2, 3, 8, 20, 80, 200][rng.below(7)])
        .height([1, 2, 3, 5, 12, 24, 60][rng.below(7)])
        .color_system(system)
        .attributes(rng.chance(70))
        .build()
}

const BOXES: [BoxStyle; 8] = [
    BoxStyle::Ascii,
    BoxStyle::Rounded,
    BoxStyle::Simple,
    BoxStyle::Heavy,
    BoxStyle::Double,
    BoxStyle::Minimal,
    BoxStyle::Square,
    BoxStyle::HeavyHead,
];

fn random_table(rng: &mut Rng) -> Table {
    let mut table = Table::new().box_style(BOXES[rng.below(8)]);
    for _ in 0..rng.below(4) {
        table.add_column(Column::new(random_string(rng)).no_wrap(rng.chance(25)));
    }
    for _ in 0..rng.below(5) {
        let cells = rng.below(5);
        table.add_row((0..cells).map(|_| random_string(rng)));
    }
    table
}

fn random_tree(rng: &mut Rng, depth: usize) -> Tree {
    let mut tree = Tree::new(random_string(rng));
    if depth < 3 {
        for _ in 0..rng.below(3) {
            tree = tree.child(random_tree(rng, depth + 1));
        }
    }
    tree
}

fn random_panel(rng: &mut Rng, depth: usize) -> Panel {
    let mut panel = Panel::new(random_body(rng, depth + 1))
        .box_style(BOXES[rng.below(8)])
        .expand(rng.chance(50))
        .padding(Padding::from((rng.below(3), rng.below(4))));
    if rng.chance(50) {
        panel = panel.title(random_string(rng));
    }
    panel
}

fn random_body(rng: &mut Rng, depth: usize) -> Body {
    match rng.below(if depth >= 2 { 2 } else { 5 }) {
        0 | 1 => Body::from(random_string(rng)),
        2 => Body::from(random_table(rng)),
        3 => Body::from(random_tree(rng, 2)),
        _ => Body::from(random_panel(rng, depth)),
    }
}

fn random_padding(rng: &mut Rng) -> Padding {
    match rng.below(3) {
        0 => Padding::from(rng.below(3)),
        1 => Padding::from((rng.below(3), rng.below(4))),
        _ => Padding::from((rng.below(3), rng.below(4), rng.below(3), rng.below(4))),
    }
}

fn random_columns(rng: &mut Rng) -> Columns {
    let mut columns = Columns::new((0..rng.below(14)).map(|_| random_body(rng, 0)))
        .padding(random_padding(rng))
        .expand(rng.chance(30))
        .equal(rng.chance(30))
        .column_first(rng.chance(40))
        .right_to_left(rng.chance(30));
    if rng.chance(25) {
        columns = columns.width(rng.below(40));
    }
    if rng.chance(30) {
        columns = columns.align([Align::Left, Align::Center, Align::Right][rng.below(3)]);
    }
    if rng.chance(30) {
        columns = columns.title(random_string(rng));
    }
    columns
}

/// A layout; with `flex` every child shares what is left by ratio alone, so the regions tile the
/// terminal exactly.
fn random_layout(rng: &mut Rng, depth: usize, named: &mut usize, flex: bool) -> Layout {
    let mut layout = if depth < 3 && rng.chance(60) {
        let children: Vec<Layout> = (0..1 + rng.below(4))
            .map(|_| random_layout(rng, depth + 1, named, flex))
            .collect();
        if rng.chance(50) {
            Layout::row(children)
        } else {
            Layout::column(children)
        }
    } else if !flex && rng.chance(15) {
        Layout::empty()
    } else {
        Layout::new(random_body(rng, 0))
    };
    if flex {
        return layout.ratio(1 + rng.below(4));
    }
    if rng.chance(25) {
        layout = layout.size(rng.below(40));
    }
    if rng.chance(40) {
        layout = layout.ratio(rng.below(5));
    }
    if rng.chance(20) {
        layout = layout.minimum_size(rng.below(8));
    }
    if rng.chance(15) {
        layout = layout.visible(false);
    }
    if rng.chance(30) {
        layout = layout.name(format!("n{}", *named));
        *named += 1;
    }
    layout
}

type Case = fn(&mut Rng, &str) -> Option<String>;

fn check_width(console: &Console, renderable: &dyn Renderable, what: &str) -> Option<String> {
    let width = usize::from(console.capabilities().width);
    let plain = console.render_to_plain(&renderable);
    for line in plain.split('\n') {
        if cell_width(line) > width {
            return Some(format!(
                "a printed {what} line is {} cells wide at width {width}: {line:?}",
                cell_width(line)
            ));
        }
    }
    let ansi = console.render_to_string(&renderable);
    if !console.capabilities().emits_escapes() && ansi != plain {
        return Some(format!(
            "a console that shows nothing wrote escape sequences for a {what}"
        ));
    }
    if console.render_to_string(&renderable) != ansi {
        return Some(format!("rendering a {what} twice gave different bytes"));
    }
    None
}

fn columns_case(rng: &mut Rng, _: &str) -> Option<String> {
    let columns = random_columns(rng);
    let console = console(rng);
    if let Some(problem) = check_width(&console, &columns, "columns") {
        return Some(problem);
    }
    let _ = columns.measure(usize::from(console.capabilities().width));
    // On a wide console the grid is as wide as its widest row and every row is padded to it.
    let wide = Console::builder().width(2000).plain().build();
    let plain = wide.render_to_plain(&columns);
    let widths: std::collections::BTreeSet<usize> = plain
        .strip_suffix('\n')
        .unwrap_or(&plain)
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(cell_width)
        .collect();
    (widths.len() > 1).then(|| format!("columns lines of different widths {widths:?}: {plain:?}"))
}

fn layout_case(rng: &mut Rng, _: &str) -> Option<String> {
    let mut named = 0;
    let mut layout = random_layout(rng, 0, &mut named, false);
    let console = console(rng);
    if let Some(problem) = check_width(&console, &layout, "layout") {
        return Some(problem);
    }
    let height = usize::from(console.capabilities().height);
    let plain = console.render_to_plain(&layout);
    let lines = plain.matches('\n').count();
    if lines != height {
        return Some(format!(
            "a layout printed {lines} lines on a console {height} rows tall: {plain:?}"
        ));
    }
    // With no fixed sizes the regions tile the terminal: every line is as wide as the console.
    let flex = random_layout(rng, 0, &mut 0, true);
    let width = usize::from(console.capabilities().width);
    let plain = console.render_to_plain(&flex);
    for line in plain.strip_suffix('\n').unwrap_or(&plain).split('\n') {
        if cell_width(line) != width {
            return Some(format!(
                "a layout of flexible children left a line {} cells wide on a console {width} wide: {plain:?}",
                cell_width(line)
            ));
        }
    }
    // Updating a named child never panics and the layout is still as tall as the terminal.
    if named > 0 {
        let name = format!("n{}", rng.below(named));
        if let Some(found) = layout.get_mut(&name) {
            found.update(random_string(rng));
        }
        let after = console.render_to_plain(&layout);
        if after.matches('\n').count() != height {
            return Some("a layout changed height after an update".to_string());
        }
    }
    None
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Where the cursor is, in lines below the first line of the display, over the whole stream:
/// a newline moves it down and `ESC [ 1 A` up. Returns the final row and the least row reached.
fn cursor_rows(text: &str) -> (i64, i64) {
    let (mut row, mut least) = (0i64, 0i64);
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'\n' {
            row += 1;
        } else if bytes[at..].starts_with(b"\x1b[1A") {
            row -= 1;
            least = least.min(row);
            at += 3;
        }
        at += 1;
    }
    (row, least)
}

fn frame_text(rng: &mut Rng) -> String {
    random_string(rng).replace('\u{1b}', "")
}

fn random_frame(rng: &mut Rng) -> Body {
    match rng.below(4) {
        0 | 1 => Body::from(frame_text(rng)),
        2 => Body::from(random_table(rng)),
        _ => Body::from(Panel::new(frame_text(rng)).expand(rng.chance(50))),
    }
}

fn live_case(rng: &mut Rng, _: &str) -> Option<String> {
    let console = console(rng);
    let interactive = rng.chance(70);
    let transient = rng.chance(40);
    let overflow = [
        VerticalOverflow::Ellipsis,
        VerticalOverflow::Crop,
        VerticalOverflow::Visible,
    ][rng.below(3)];
    let capture = Capture::default();
    let mut builder = Live::builder()
        .console(console)
        .interactive(interactive)
        .transient(transient)
        .vertical_overflow(overflow)
        .auto_refresh(false)
        .writer(capture.clone());
    if rng.chance(80) {
        builder = builder.renderable(random_frame(rng));
    }
    let live = builder.build();
    let text = |capture: &Capture| String::from_utf8_lossy(&capture.0.lock().unwrap()).into_owned();
    live.start();
    for _ in 0..rng.below(6) {
        match rng.below(3) {
            0 => live.update(random_frame(rng)),
            1 => live.refresh(),
            _ => {
                live.update(random_frame(rng));
                live.refresh();
            }
        }
    }
    if !interactive && !text(&capture).is_empty() {
        return Some(format!(
            "a stream that is not interactive was written to before the end: {:?}",
            text(&capture)
        ));
    }
    live.stop();
    live.stop();
    let written = text(&capture);
    if !interactive {
        if transient && !written.is_empty() {
            return Some(format!(
                "a transient display on a stream that is not interactive wrote {written:?}"
            ));
        }
        return None;
    }
    if written.matches("\x1b[?25l").count() != 1 || written.matches("\x1b[?25h").count() != 1 {
        return Some(format!(
            "the cursor is not hidden once and shown once: {written:?}"
        ));
    }
    let (row, least) = cursor_rows(&written);
    if least < 0 {
        return Some(format!(
            "the cursor went above the display (row {least}): {written:?}"
        ));
    }
    if transient && row != 0 {
        return Some(format!(
            "a transient display left the cursor {row} lines below where it started: {written:?}"
        ));
    }
    if !transient && row < 0 {
        return Some(format!(
            "the display ended above where it started: {written:?}"
        ));
    }
    None
}

fn run(feature: &str, seed: u64, count: usize, case: Case) -> bool {
    let mut rng = Rng(seed ^ feature.len() as u64 * 0x1234_5678_9ABC_DEF1);
    let (mut panics, mut violations) = (0usize, 0usize);
    let (mut first_panic, mut first_violation): (Option<String>, Option<String>) = (None, None);
    for _ in 0..count {
        let input = random_string(&mut rng);
        let mut case_rng = Rng(rng.next());
        match panic::catch_unwind(AssertUnwindSafe(|| case(&mut case_rng, &input))) {
            Err(_) => {
                panics += 1;
                first_panic.get_or_insert_with(|| format!("panic on {input:?}"));
            }
            Ok(Some(problem)) => {
                violations += 1;
                first_violation.get_or_insert_with(|| format!("{problem} (input {input:?})"));
            }
            Ok(None) => {}
        }
    }
    println!(
        "{}",
        json!({"feature": feature, "inputs": count, "panics": panics, "violations": violations, "first_panic": first_panic, "first_violation": first_violation})
    );
    panics == 0 && violations == 0
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args[i + 1].clone())
    };
    let seed: u64 = get("--seed").map_or(20_261_008, |v| v.parse().unwrap());
    let count: usize = get("--count").map_or(5000, |v| v.parse().unwrap());
    panic::set_hook(Box::new(|_| {}));
    let cases: [(&str, Case); 3] = [
        ("columns", columns_case),
        ("layout", layout_case),
        ("live", live_case),
    ];
    let mut ok = true;
    for (feature, case) in cases {
        ok &= run(feature, seed, count, case);
    }
    std::process::exit(i32::from(!ok));
}
