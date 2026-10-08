// fuzz [--seed S] [--count N]   (bench/scripts/fuzz.py drives it)
// Zero-panic check: N seeded random inputs per feature (style parse, markup parse and render,
// width and segmentation, capability resolution, table, panel, tree, progress, error, padding, text operations, progress updates and color parsing), plus a few properties that must hold on every
// input. Prints one JSON line per feature and exits 1 on any panic or violated property.
use std::io::Write;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use hud::{
    Align, BarColumn, Body, BoxStyle, Color, Column, ColorSystem, Console, EnvSnapshot, ErrorReport, Justify,
    MofNCompleteColumn, Overflow, Pad, Padding, Panel, Progress, Renderable, SpinnerColumn, StreamInfo,
    Style, Table, TaskProgressColumn, Text, TextColumn, TimeElapsedColumn, TimeRemainingColumn,
    Tree, cell_len, cell_width, clusters, escape, fold, pad, resolve, truncate,
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
    "[", "]", "[/]", "[/", "\\", "\\\\", "\\[", "[bold]", "[/bold]", "[red on blue]", "[#ff8800]",
    "[color(300)]", "[rgb(1,2,3)]", "[link=https://x.org]", "[/link]", "[@x]", "[b", "bold", "on",
    "not", "link", "color(", "rgb(", "#", "=", " ", "  ", "\t", "\n", "\r\n", "default", "grey50",
    "x", "word", "supercalifragilisticexpialidocious", "0", "255", "256", "-1",
];
const UNICODE: &[&str] = &[
    "你好", "😀", "👨\u{200d}👩\u{200d}👧", "🇧🇷", "e\u{301}", "\u{200d}", "\u{feff}", "\u{0}", "\u{7}", "\u{1b}",
    "\u{85}", "\u{2028}", "ｆｕｌｌ", "한국어", "क्ष", "\u{fe0f}", "\u{1f3fd}", "\u{10ffff}", "\u{e000}",
];

fn random_char(rng: &mut Rng) -> char {
    loop {
        let code = match rng.below(4) {
            0 => rng.below(0x80) as u32,
            1 => rng.below(0x800) as u32,
            2 => rng.below(0x10000) as u32,
            _ => rng.below(0x11_0000) as u32,
        };
        if let Some(c) = char::from_u32(code) {
            return c;
        }
    }
}

fn random_string(rng: &mut Rng) -> String {
    let mut out = String::new();
    for _ in 0..rng.below(24) {
        match rng.below(10) {
            0..=4 => out.push_str(rng.pick(FRAGMENTS)),
            5 | 6 => out.push_str(rng.pick(UNICODE)),
            7 => out.push(random_char(rng)),
            8 => out.extend((0..rng.below(40)).map(|_| 'a')),
            _ => out.push_str(&rng.below(100_000).to_string()),
        }
    }
    out
}

fn system(rng: &mut Rng) -> ColorSystem {
    [ColorSystem::None, ColorSystem::Standard, ColorSystem::EightBit, ColorSystem::TrueColor][rng.below(4)]
}

fn console(rng: &mut Rng) -> Console {
    Console::builder()
        .width([0, 1, 2, 3, 8, 20, 80, 200, u16::MAX][rng.below(9)])
        .color_system(system(rng))
        .attributes(rng.chance(70))
        .build()
}

fn text_options(rng: &mut Rng, text: Text) -> Text {
    text.justify([Justify::Default, Justify::Left, Justify::Center, Justify::Right, Justify::Full][rng.below(5)])
        .overflow([Overflow::Fold, Overflow::Crop, Overflow::Ellipsis, Overflow::Ignore][rng.below(4)])
        .no_wrap(rng.chance(20))
        .tab_size(rng.below(20))
        .end(rng.pick(&["\n", "", " | ", "\n\n"]))
}

type Case = fn(&mut Rng, &str) -> Option<String>;

fn style_case(_: &mut Rng, input: &str) -> Option<String> {
    let style = Style::parse(input).ok()?;
    match Style::parse(&style.to_string()) {
        Ok(again) if again == style => None,
        other => Some(format!("{:?} displays as {:?} and parses back to {other:?}", style, style.to_string())),
    }
}

fn markup_case(rng: &mut Rng, input: &str) -> Option<String> {
    let escaped_ok = !input.ends_with('\\');
    if escaped_ok {
        match Text::from_markup(&escape(input)) {
            Ok(text) if text.plain() == strip_controls(input) => {}
            other => return Some(format!("escape does not round-trip: {other:?}")),
        }
    }
    let Ok(text) = Text::from_markup(input) else {
        return None;
    };
    let text = text_options(rng, text);
    let console = console(rng);
    let plain = console.render_to_plain(&text);
    let width = usize::from(console.capabilities().width);
    let ansi = console.render_to_string(&text);
    if width > 0 {
        for line in plain.split('\n') {
            if cell_width(line) > width {
                return Some(format!("a printed line is {} cells wide at width {width}: {line:?}", cell_width(line)));
            }
        }
    }
    if !console.capabilities().emits_escapes() && ansi != plain {
        return Some("a console that shows nothing wrote escape sequences".to_string());
    }
    None
}

fn random_table(rng: &mut Rng, input: &str) -> Table {
    let boxes = [
        BoxStyle::Ascii,
        BoxStyle::Rounded,
        BoxStyle::Simple,
        BoxStyle::Heavy,
        BoxStyle::Double,
        BoxStyle::Minimal,
        BoxStyle::Square,
        BoxStyle::HeavyHead,
    ];
    let columns = 1 + rng.below(6);
    let mut table = Table::new()
        .box_style(boxes[rng.below(8)])
        .show_lines(rng.chance(40));
    if rng.chance(60) {
        table = table.title(input);
    }
    if rng.chance(40) {
        table = table.caption(random_string(rng));
    }
    for _ in 0..columns {
        let mut column = Column::new(random_string(rng))
            .justify([Justify::Left, Justify::Center, Justify::Right, Justify::Default][rng.below(4)])
            .overflow([Overflow::Fold, Overflow::Crop, Overflow::Ellipsis, Overflow::Ignore][rng.below(4)])
            .no_wrap(rng.chance(25));
        if rng.chance(15) {
            column = column.width(rng.below(30));
        }
        if rng.chance(15) {
            column = column.min_width(rng.below(20));
        }
        if rng.chance(15) {
            column = column.max_width(rng.below(30));
        }
        table.add_column(column);
    }
    for _ in 0..rng.below(9) {
        let cells = rng.below(columns + 2);
        table.add_row((0..cells).map(|_| random_string(rng)));
    }
    table
}

fn table_case(rng: &mut Rng, input: &str) -> Option<String> {
    let table = random_table(rng, input);
    let console = console(rng);
    let width = usize::from(console.capabilities().width);
    let plain = console.render_to_plain(&table);
    for line in plain.split('\n') {
        if cell_width(line) > width {
            return Some(format!("a printed line is {} cells wide at width {width}: {line:?}", cell_width(line)));
        }
    }
    let wide = Console::builder().width(60_000).plain().build();
    let plain = wide.render_to_plain(&table);
    let widths: std::collections::BTreeSet<usize> = plain
        .strip_suffix('\n')
        .unwrap_or(&plain)
        .split('\n')
        .map(cell_width)
        .collect();
    (widths.len() > 1).then(|| format!("lines of different widths {widths:?} in a table that fits: {plain:?}"))
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

const STYLES: &[&str] = &["", "red", "bold green on blue", "#ff8800", "dim", "underline2", "not bold"];

fn random_style(rng: &mut Rng) -> Style {
    Style::parse(rng.pick(STYLES)).unwrap_or_default()
}

fn random_tree(rng: &mut Rng, depth: usize, nodes: &mut usize) -> Tree {
    *nodes += 1;
    let mut tree = Tree::new(random_string(rng));
    if rng.chance(25) {
        tree = tree.guide_style(random_style(rng));
    }
    if depth < 4 {
        for _ in 0..rng.below(4) {
            tree = tree.child(random_tree(rng, depth + 1, nodes));
        }
    }
    tree
}

fn random_body(rng: &mut Rng, depth: usize) -> Body {
    match rng.below(if depth >= 2 { 3 } else { 6 }) {
        0 | 1 => Body::from(random_string(rng)),
        2 => {
            let text = Text::new(random_string(rng));
            Body::from(text_options(rng, text))
        }
        3 => Body::from(random_table(rng, "t")),
        4 => Body::from(random_tree(rng, 2, &mut 0)),
        _ => Body::from(random_panel(rng, depth + 1)),
    }
}

fn random_panel(rng: &mut Rng, depth: usize) -> Panel {
    let mut panel = Panel::new(random_body(rng, depth))
        .box_style(BOXES[rng.below(8)])
        .expand(rng.chance(50))
        .padding(match rng.below(3) {
            0 => Pad::from(rng.below(3)),
            1 => Pad::from((rng.below(3), rng.below(4))),
            _ => Pad::from((rng.below(3), rng.below(4), rng.below(3), rng.below(4))),
        })
        .title_align([Align::Left, Align::Center, Align::Right][rng.below(3)])
        .subtitle_align([Align::Left, Align::Center, Align::Right][rng.below(3)])
        .border_style(random_style(rng));
    if rng.chance(60) {
        panel = panel.title(random_string(rng));
    }
    if rng.chance(40) {
        panel = panel.subtitle(random_string(rng));
    }
    panel
}

fn panel_case(rng: &mut Rng, _: &str) -> Option<String> {
    let panel = random_panel(rng, 0);
    let console = console(rng);
    let width = usize::from(console.capabilities().width);
    let plain = console.render_to_plain(&panel);
    for line in plain.split('\n') {
        if cell_width(line) > width {
            return Some(format!("a printed line is {} cells wide at width {width}: {line:?}", cell_width(line)));
        }
    }
    let ansi = console.render_to_string(&panel);
    if !console.capabilities().emits_escapes() && ansi != plain {
        return Some("a console that shows nothing wrote escape sequences".to_string());
    }
    let wide = Console::builder().width(2000).plain().build();
    let plain = wide.render_to_plain(&panel);
    let widths: std::collections::BTreeSet<usize> = plain
        .strip_suffix('\n')
        .unwrap_or(&plain)
        .split('\n')
        .map(cell_width)
        .collect();
    let _ = panel.measure(width);
    (widths.len() > 1).then(|| format!("panel lines of different widths {widths:?}: {plain:?}"))
}

#[derive(Debug)]
struct Chain {
    message: String,
    source: Option<Box<Chain>>,
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Chain {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|c| c as &(dyn std::error::Error + 'static))
    }
}

fn error_case(rng: &mut Rng, input: &str) -> Option<String> {
    let mut report = ErrorReport::new(input);
    let causes: Vec<String> = (0..rng.below(5)).map(|_| random_string(rng)).collect();
    for cause in &causes {
        report = report.cause(cause.clone());
    }
    let hint = rng.chance(50).then(|| random_string(rng));
    if let Some(hint) = &hint {
        report = report.hint(hint.clone());
    }
    let mut chain: Option<Box<Chain>> = None;
    for cause in causes.iter().rev() {
        chain = Some(Box::new(Chain { message: cause.clone(), source: chain }));
    }
    let top = Chain { message: input.to_string(), source: chain };
    let mut by_hand = ErrorReport::new(input);
    for cause in &causes {
        by_hand = by_hand.cause(cause.clone());
    }
    if ErrorReport::from_error(&top) != by_hand {
        return Some("from_error is not the report built from the same chain by hand".to_string());
    }
    let console = console(rng);
    let width = usize::from(console.capabilities().width);
    let plain = console.render_to_plain(&report);
    for line in plain.split('\n') {
        if cell_width(line) > width {
            return Some(format!("a printed line is {} cells wide at width {width}: {line:?}", cell_width(line)));
        }
    }
    let ansi = console.render_to_string(&report);
    if !console.capabilities().emits_escapes() && ansi != plain {
        return Some("a console that shows nothing wrote escape sequences".to_string());
    }
    let wide = Console::builder().width(2000).plain().build();
    let plain = wide.render_to_plain(&report);
    let widths: std::collections::BTreeSet<usize> = plain
        .strip_suffix('\n')
        .unwrap_or(&plain)
        .split('\n')
        .map(cell_width)
        .collect();
    let _ = report.measure(width);
    if widths.len() > 1 {
        return Some(format!("error report lines of different widths {widths:?}: {plain:?}"));
    }
    let least = 3 + if causes.is_empty() { 0 } else { 2 + causes.len() } + if hint.is_some() { 2 } else { 0 };
    let lines = plain.matches('\n').count();
    (lines < least).then(|| format!("{lines} lines printed for a report that needs at least {least}: {plain:?}"))
}

fn tree_case(rng: &mut Rng, _: &str) -> Option<String> {
    let mut nodes = 0;
    let tree = random_tree(rng, 0, &mut nodes);
    let console = console(rng);
    let width = usize::from(console.capabilities().width);
    let plain = console.render_to_plain(&tree);
    for line in plain.split('\n') {
        if cell_width(line) > width {
            return Some(format!("a printed line is {} cells wide at width {width}: {line:?}", cell_width(line)));
        }
    }
    let ansi = console.render_to_string(&tree);
    if !console.capabilities().emits_escapes() && ansi != plain {
        return Some("a console that shows nothing wrote escape sequences".to_string());
    }
    let lines = plain.matches('\n').count();
    let _ = tree.measure(width);
    (width > 16 && lines < nodes).then(|| format!("{nodes} nodes printed on {lines} lines: {plain:?}"))
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

impl Capture {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

const TOTALS: &[u64] = &[0, 1, 2, 9, 10, 99, 100, 12_500, 1_000_000, u64::MAX];
const CLOCKS: &[f64] = &[0.0, 0.5, 1.0, 29.0, 31.0, 3600.0, 90_000.0, 1e12, -5.0, f64::NAN, f64::INFINITY];

/// A frame's bytes without what is written to put it in place: the cursor hide of the first
/// draw and the erase of the previous frame (`CR`, then `ESC [ 2 K` and, for each earlier line,
/// `ESC [ 1 A ESC [ 2 K`).
fn frame_of(written: &str) -> &str {
    let mut rest = written.strip_prefix("\x1b[?25l").unwrap_or(written);
    if let Some(erased) = rest.strip_prefix("\r\x1b[2K") {
        rest = erased;
        while let Some(erased) = rest.strip_prefix("\x1b[1A\x1b[2K") {
            rest = erased;
        }
    }
    rest
}

fn progress_case(rng: &mut Rng, input: &str) -> Option<String> {
    let console = console(rng);
    let width = usize::from(console.capabilities().width);
    let height = usize::from(console.capabilities().height);
    let clock = Arc::new(Mutex::new(0.0_f64));
    let capture = Capture::default();
    let interactive = rng.chance(50);
    let clock_handle = Arc::clone(&clock);
    let mut builder = Progress::builder()
        .console(console.clone())
        .clock(move || *clock_handle.lock().unwrap())
        .writer(capture.clone())
        .interactive(interactive)
        .auto_refresh(false)
        .transient(rng.chance(10));
    let usual = rng.chance(40);
    if usual {
        builder = builder
            .column(TextColumn::new("{task.description}"))
            .column(BarColumn::new().bar_width(1 + rng.below(40)))
            .column(TaskProgressColumn::new())
            .column(MofNCompleteColumn::new());
    }
    for _ in 0..if usual { 0 } else { rng.below(6) } {
        builder = match rng.below(7) {
            0 => builder.column(TextColumn::new(if rng.chance(50) { input.to_string() } else { random_string(rng) })),
            1 => builder.column(match rng.below(3) {
                0 => BarColumn::new().full_width(),
                1 => BarColumn::new().bar_width(rng.below(60)),
                _ => BarColumn::new(),
            }),
            2 => builder.column(TaskProgressColumn::new()),
            3 => builder.column(MofNCompleteColumn::new().separator(random_string(rng))),
            4 => builder.column(TimeElapsedColumn::new()),
            5 => builder.column(TimeRemainingColumn::new().compact(rng.chance(50)).elapsed_when_finished(rng.chance(50))),
            _ => builder.column(SpinnerColumn::new().speed([0.0, 1.0, 2.5, -1.0][rng.below(4)]).finished_text(random_string(rng))),
        };
    }
    let progress = builder.build();
    let mut tasks = Vec::new();
    for _ in 0..rng.below(5) {
        let description = if usual {
            ["fetch", "download crates", "verify", "x", "a long description that does not fit"][rng.below(5)].to_string()
        } else if rng.chance(50) {
            input.to_string()
        } else {
            random_string(rng)
        };
        tasks.push(progress.add_task(description, TOTALS[rng.below(TOTALS.len())]));
    }
    for _ in 0..rng.below(24) {
        *clock.lock().unwrap() = CLOCKS[rng.below(CLOCKS.len())];
        if tasks.is_empty() {
            break;
        }
        let task = &tasks[rng.below(tasks.len())];
        match rng.below(7) {
            0 => task.advance([0, 1, 5, 1_000, u64::MAX][rng.below(5)]),
            1 => task.set_completed(TOTALS[rng.below(TOTALS.len())]),
            2 => task.set_total(TOTALS[rng.below(TOTALS.len())]),
            3 => task.set_description(random_string(rng)),
            4 => task.finish(),
            5 => progress.refresh(),
            _ => {
                let _ = console.render_to_string(&progress);
            }
        }
    }
    let plain = console.render_to_plain(&progress);
    for line in plain.split('\n') {
        if cell_width(line) > width {
            return Some(format!("a printed line is {} cells wide at width {width}: {line:?}", cell_width(line)));
        }
    }
    let ansi = console.render_to_string(&progress);
    if !console.capabilities().emits_escapes() && ansi != plain {
        return Some("a console that shows nothing wrote escape sequences".to_string());
    }
    // The live path, assembled from cached pieces when it can be, writes the bytes of a render.
    let before = capture.text().len();
    progress.refresh();
    let written = capture.text();
    let drawn = &written[before..];
    if !interactive {
        if !drawn.is_empty() {
            return Some(format!("a stream that is not interactive drew a frame: {drawn:?}"));
        }
    } else {
        // A frame taller than the terminal is cut on purpose, with an ellipsis line.
        let want = console.render_to_string(&progress);
        let want = want.strip_suffix('\n').unwrap_or(&want);
        if want.matches('\n').count() + 1 < height && frame_of(drawn) != want {
            return Some(format!("the live frame differs from a render at width {width}: want {want:?} got {:?}", frame_of(drawn)));
        }
    }
    progress.finish();
    progress.finish();
    None
}

fn strip_controls(text: &str) -> String {
    text.chars().filter(|c| !matches!(*c, '\u{7}' | '\u{8}' | '\u{b}' | '\u{c}' | '\r')).collect()
}

fn width_case(rng: &mut Rng, input: &str) -> Option<String> {
    let joined: String = clusters(input).collect();
    if joined != input {
        return Some("clusters do not add up to the input".to_string());
    }
    let limit = rng.below(30);
    let lines: Vec<&str> = fold(input, limit).collect();
    if lines.concat() != input {
        return Some(format!("fold at {limit} loses text"));
    }
    for line in &lines {
        if cell_width(line) > limit && clusters(line).count() > 1 {
            return Some(format!("fold at {limit} made a line of {} cells with several clusters", cell_width(line)));
        }
    }
    let cut = truncate(input, limit);
    if !input.starts_with(cut) {
        return Some("truncate is not a prefix".to_string());
    }
    let _ = pad(input, limit, hud::width::Align::Center).to_string();
    None
}

fn capability_case(rng: &mut Rng, _: &str) -> Option<String> {
    const VALUES: &[&str] = &["", "0", "1", "no", "truecolor", "24bit", "xterm", "xterm-256color", "dumb", "-5", "99999999999", "x"];
    let value = |rng: &mut Rng| rng.chance(50).then(|| rng.pick(VALUES).to_string());
    let env = EnvSnapshot {
        no_color: value(rng),
        force_color: value(rng),
        clicolor: value(rng),
        clicolor_force: value(rng),
        colorterm: value(rng),
        term: value(rng),
        columns: value(rng),
        lines: value(rng),
    };
    let stream = StreamInfo {
        is_tty: rng.chance(50),
        size: rng.chance(60).then(|| (rng.below(300) as u16, rng.below(100) as u16)),
    };
    let caps = resolve(&env, stream);
    (caps.width == 0 || caps.height == 0).then(|| format!("resolved a zero size from {env:?} {stream:?}"))
}

fn random_pad(rng: &mut Rng) -> Pad {
    match rng.below(3) {
        0 => Pad::from(rng.below(4)),
        1 => Pad::from((rng.below(3), rng.below(5))),
        _ => Pad::from((rng.below(3), rng.below(4), rng.below(3), rng.below(5))),
    }
}

fn padding_case(rng: &mut Rng, _: &str) -> Option<String> {
    let expand = rng.chance(50);
    let padding = Padding::new(random_body(rng, 0), random_pad(rng))
        .style(random_style(rng))
        .expand(expand);
    let console = console(rng);
    let width = usize::from(console.capabilities().width);
    let plain = console.render_to_plain(&padding);
    for line in plain.split('\n') {
        if cell_width(line) > width {
            return Some(format!("a printed line is {} cells wide at width {width}: {line:?}", cell_width(line)));
        }
    }
    let ansi = console.render_to_string(&padding);
    if !console.capabilities().emits_escapes() && ansi != plain {
        return Some("a console that shows nothing wrote escape sequences".to_string());
    }
    let wide = Console::builder().width(2000).plain().build();
    let lines = wide.render_to_plain(&padding);
    let widths: std::collections::BTreeSet<usize> = lines
        .strip_suffix('\n')
        .unwrap_or(&lines)
        .split('\n')
        .map(cell_width)
        .collect();
    let _ = padding.measure(width);
    (expand && widths.len() > 1).then(|| format!("padding lines of different widths {widths:?}: {lines:?}"))
}

fn bound(rng: &mut Rng, len: usize) -> usize {
    [0, 1, 2, len / 2, len, len + 1, usize::MAX][rng.below(7)]
}

fn textops_case(rng: &mut Rng, input: &str) -> Option<String> {
    let mut text = text_options(rng, Text::new(input.to_string()));
    let len = text.plain().len();
    for _ in 0..rng.below(6) {
        let style = random_style(rng);
        match rng.below(5) {
            0 => text.stylize(style, ..),
            1 => text.stylize(style, bound(rng, len)..),
            2 => text.stylize(style, ..bound(rng, len)),
            3 => text.stylize(style, bound(rng, len)..=bound(rng, len)),
            _ => text.stylize(style, bound(rng, len)..bound(rng, len)),
        }
    }
    for span in text.spans() {
        let plain = text.plain();
        if span.start >= span.end || span.end > plain.len() || !plain.is_char_boundary(span.start) || !plain.is_char_boundary(span.end) {
            return Some(format!("span {}..{} is not a valid range of {:?}", span.start, span.end, plain));
        }
    }
    let max = [0, 1, 2, 5, 20][rng.below(5)];
    let overflow = [Overflow::Fold, Overflow::Crop, Overflow::Ellipsis][rng.below(3)];
    let mut cut = text.clone();
    cut.truncate(max, overflow, rng.chance(50));
    let measured = cell_len(cut.plain());
    if measured > max && max > 0 {
        return Some(format!("truncate({max}) left {measured} cells: {:?}", cut.plain()));
    }
    for span in cut.spans() {
        if span.end > cut.plain().len() || !cut.plain().is_char_boundary(span.start) || !cut.plain().is_char_boundary(span.end) {
            return Some(format!("truncate left a span {}..{} outside {:?}", span.start, span.end, cut.plain()));
        }
    }
    for width in [1usize, 3, 10, 40] {
        let lines = text.wrap(width);
        if text.plain().is_empty() && lines.len() > 1 {
            return Some("empty text wrapped to several lines".to_string());
        }
    }
    None
}

fn update_case(rng: &mut Rng, input: &str) -> Option<String> {
    let console = console(rng);
    let width = usize::from(console.capabilities().width);
    let clock = Arc::new(Mutex::new(0.0_f64));
    let clock_handle = Arc::clone(&clock);
    let progress = Progress::builder()
        .console(console.clone())
        .clock(move || *clock_handle.lock().unwrap())
        .disable(true)
        .build();
    let mut tasks = Vec::new();
    for step in 0..rng.below(24) + 1 {
        *clock.lock().unwrap() = CLOCKS[rng.below(CLOCKS.len())];
        if tasks.is_empty() || rng.chance(15) {
            tasks.push(progress.add_task(if rng.chance(50) { input.to_string() } else { random_string(rng) }, TOTALS[rng.below(TOTALS.len())]));
            continue;
        }
        let task = &tasks[rng.below(tasks.len())];
        let mut update = progress.update(task);
        if rng.chance(40) {
            update = update.total(TOTALS[rng.below(TOTALS.len())]);
        }
        if rng.chance(40) {
            update = update.completed(TOTALS[rng.below(TOTALS.len())]);
        }
        if rng.chance(40) {
            update = update.advance(TOTALS[rng.below(TOTALS.len())]);
        }
        if rng.chance(30) {
            update = update.description(random_string(rng));
        }
        if rng.chance(30) {
            update = update.visible(rng.chance(60));
        }
        drop(update);
        if step % 5 == 0 {
            let plain = console.render_to_plain(&progress);
            for line in plain.split('\n') {
                if cell_width(line) > width {
                    return Some(format!("a progress line is {} cells wide at width {width}", cell_width(line)));
                }
            }
        }
    }
    None
}

fn colorparse_case(rng: &mut Rng, input: &str) -> Option<String> {
    for candidate in [input.to_string(), random_string(rng), format!("#{}", input), format!("color({input})"), format!("rgb({input})")] {
        if let Ok(color) = Color::parse(&candidate) {
            match Color::parse(&color.to_string()) {
                Ok(again) if again == color => {}
                other => return Some(format!("{candidate:?} parsed to {color:?} whose text {:?} parses to {other:?}", color.to_string())),
            }
        }
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
    println!("{}", json!({"feature": feature, "inputs": count, "panics": panics, "violations": violations, "first_panic": first_panic, "first_violation": first_violation}));
    panics == 0 && violations == 0
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].clone());
    let seed: u64 = get("--seed").map_or(20_261_008, |v| v.parse().unwrap());
    let count: usize = get("--count").map_or(5000, |v| v.parse().unwrap());
    panic::set_hook(Box::new(|_| {}));
    let cases: [(&str, Case); 13] = [
        ("style", style_case),
        ("markup", markup_case),
        ("width", width_case),
        ("capability", capability_case),
        ("table", table_case),
        ("panel", panel_case),
        ("tree", tree_case),
        ("progress", progress_case),
        ("error", error_case),
        ("padding", padding_case),
        ("textops", textops_case),
        ("update", update_case),
        ("colorparse", colorparse_case),
    ];
    let mut ok = true;
    for (feature, case) in cases {
        ok &= run(feature, seed, count, case);
    }
    std::process::exit(i32::from(!ok));
}
