// fuzz [--seed S] [--count N]   (bench/scripts/fuzz.py drives it)
// Zero-panic check: N seeded random inputs per feature (style parse, markup parse and render,
// width and segmentation, capability resolution), plus a few properties that must hold on every
// input. Prints one JSON line per feature and exits 1 on any panic or violated property.
use std::panic::{self, AssertUnwindSafe};

use hud::{
    ColorSystem, Console, EnvSnapshot, Justify, Overflow, StreamInfo, Style, Text, cell_width,
    clusters, escape, fold, pad, resolve, truncate,
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
    let cases: [(&str, Case); 4] = [
        ("style", style_case),
        ("markup", markup_case),
        ("width", width_case),
        ("capability", capability_case),
    ];
    let mut ok = true;
    for (feature, case) in cases {
        ok &= run(feature, seed, count, case);
    }
    std::process::exit(i32::from(!ok));
}
