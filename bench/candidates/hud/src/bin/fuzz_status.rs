// fuzz_status [--seed S] [--count N]   (bench/scripts/fuzz.py drives it)
// Zero-panic check for Spinner and Status: N seeded random inputs per feature, plus the properties
// that must hold on every input. Same protocol as fuzz.rs (one JSON line per feature, exit 1 on
// any panic or violated property); a separate binary because fuzz.rs belongs to the main line.
use std::io::Write;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use hud::{Console, ColorSystem, Node, Renderable, Spinner, Status, Style, Text, cell_width};
use serde_json::json;

type Case = fn(&mut Rng, &str) -> Option<String>;

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

const TIMES: &[f64] = &[
    0.0,
    0.001,
    0.08,
    0.5,
    1.0,
    7.77,
    1000.0,
    -5.0,
    1e12,
    1e300,
    f64::MAX,
    f64::MIN_POSITIVE,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NAN,
];
const SPEEDS: &[f64] = &[
    0.0,
    1.0,
    -1.0,
    0.5,
    3.0,
    1e9,
    1e-9,
    f64::MAX,
    f64::INFINITY,
    f64::NAN,
];
const STYLES: &[&str] = &["", "bold", "red", "bold red on blue", "dim italic", "reverse"];

type Clock = Arc<Mutex<f64>>;

fn clock_fn(clock: &Clock) -> impl Fn() -> f64 + Send + Sync + 'static {
    let clock = Arc::clone(clock);
    move || *clock.lock().unwrap()
}

fn set(clock: &Clock, at: f64) {
    *clock.lock().unwrap() = at;
}

fn time(rng: &mut Rng) -> f64 {
    if rng.chance(60) {
        rng.below(2_000_000) as f64 / 1000.0
    } else {
        TIMES[rng.below(TIMES.len())]
    }
}

fn style(rng: &mut Rng) -> Style {
    Style::parse(rng.pick(STYLES)).unwrap_or_default()
}

fn name(rng: &mut Rng) -> String {
    if rng.chance(75) {
        let names: Vec<&str> = Spinner::names().collect();
        names[rng.below(names.len())].to_string()
    } else {
        random_string(rng)
    }
}

/// What a message is once read as markup and printed with no style.
fn plain_of(markup: &str) -> String {
    Text::from_markup(markup).unwrap_or_else(|_| Text::new(markup)).plain().to_string()
}

fn too_wide(console: &Console, spinner: &dyn Renderable) -> Option<String> {
    let width = console.width();
    for line in console.render_to_plain(spinner).lines() {
        if cell_width(line) > width {
            return Some(format!("a line of {} cells on a console {width} wide: {line:?}", cell_width(line)));
        }
    }
    None
}

fn spinner_case(rng: &mut Rng, input: &str) -> Option<String> {
    let clock = Clock::default();
    let spinner = Spinner::new(&name(rng))
        .text(input)
        .style(style(rng))
        .speed(SPEEDS[rng.below(SPEEDS.len())])
        .clock(clock_fn(&clock));
    let expected = Node::text(plain_of(input));
    let console = console(rng);
    for _ in 0..1 + rng.below(6) {
        set(&clock, time(rng));
        let first = console.render_to_string(&spinner);
        if first != console.render_to_string(&spinner) {
            return Some(format!("two prints at the same time differ: {first:?}"));
        }
        if let Some(problem) = too_wide(&console, &spinner) {
            return Some(problem);
        }
        if spinner.node() != expected {
            return Some(format!("the document depends on the clock: {:?}", spinner.node()));
        }
    }
    None
}

fn spinner_update_case(rng: &mut Rng, input: &str) -> Option<String> {
    let clock = Clock::default();
    let spinner = Spinner::new(&name(rng)).text(input).clock(clock_fn(&clock));
    let mut expected = plain_of(input);
    let console = console(rng);
    for _ in 0..rng.below(8) {
        let mut update = spinner.update();
        if rng.chance(70) {
            let text = random_string(rng);
            if !text.is_empty() {
                expected = plain_of(&text);
            }
            update = update.text(text);
        }
        if rng.chance(40) {
            update = update.style(style(rng));
        }
        if rng.chance(50) {
            update = update.speed(SPEEDS[rng.below(SPEEDS.len())]);
        }
        drop(update);
        set(&clock, time(rng));
        if let Some(problem) = too_wide(&console, &spinner) {
            return Some(problem);
        }
    }
    if spinner.node() != Node::text(expected.clone()) {
        return Some(format!(
            "after the updates the message is {:?}, not {expected:?}",
            spinner.node()
        ));
    }
    None
}

fn status_case(rng: &mut Rng, input: &str) -> Option<String> {
    let clock = Clock::default();
    let capture = Capture::default();
    let interactive = rng.chance(75);
    let status = Status::builder()
        .text(input)
        .console(console(rng))
        .spinner(name(rng))
        .spinner_style(style(rng))
        .speed(SPEEDS[rng.below(SPEEDS.len())])
        .interactive(interactive)
        .auto_refresh(false)
        .clock(clock_fn(&clock))
        .writer(capture.clone())
        .build();
    let text = |capture: &Capture| String::from_utf8_lossy(&capture.0.lock().unwrap()).into_owned();
    let mut started = false;
    for _ in 0..rng.below(9) {
        set(&clock, time(rng));
        match rng.below(6) {
            0 => {
                status.start();
                started = true;
            }
            1 | 2 => status.refresh(),
            3 => {
                let mut update = status.update();
                if rng.chance(70) {
                    update = update.text(random_string(rng));
                }
                if rng.chance(40) {
                    update = update.spinner(name(rng));
                }
                if rng.chance(40) {
                    update = update.spinner_style(style(rng));
                }
                if rng.chance(40) {
                    update = update.speed(SPEEDS[rng.below(SPEEDS.len())]);
                }
                drop(update);
            }
            4 => status.stop(),
            _ => drop(status.clone()),
        }
    }
    status.stop();
    status.stop();
    let written = text(&capture);
    if !interactive || !started {
        if written.is_empty() {
            return None;
        }
        return Some(format!(
            "a status that is not interactive or never started wrote {written:?}"
        ));
    }
    if written.matches("\x1b[?25l").count() != 1 || written.matches("\x1b[?25h").count() != 1 {
        return Some(format!("the cursor is not hidden once and shown once: {written:?}"));
    }
    let (row, least) = cursor_rows(&written);
    if least < 0 {
        return Some(format!("the cursor went above the display (row {least}): {written:?}"));
    }
    if row != 0 {
        return Some(format!(
            "a transient status left the cursor {row} lines below where it started: {written:?}"
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
        ("spinner", spinner_case),
        ("spinner_update", spinner_update_case),
        ("status", status_case),
    ];
    let mut ok = true;
    for (feature, case) in cases {
        ok &= run(feature, seed, count, case);
    }
    std::process::exit(i32::from(!ok));
}
