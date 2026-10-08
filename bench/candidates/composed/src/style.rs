//! Glue: Rich-style strings, color downgrade and the bridge to owo-colors.
use owo_colors::{AnsiColors, DynColors, Style, XtermColors};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Depth {
    #[default]
    None,
    Std,
    Idx,
    True,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Col {
    Default,
    Std(u8),
    Idx(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct St {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub reverse: bool,
    pub fg: Option<Col>,
    pub bg: Option<Col>,
}

const NAMES: [&str; 8] = [
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
];
const STD16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (170, 0, 0),
    (0, 170, 0),
    (170, 85, 0),
    (0, 0, 170),
    (170, 0, 170),
    (0, 170, 170),
    (170, 170, 170),
    (85, 85, 85),
    (255, 85, 85),
    (85, 255, 85),
    (255, 255, 85),
    (85, 85, 255),
    (255, 85, 255),
    (85, 255, 255),
    (255, 255, 255),
];
const XTERM16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (128, 0, 0),
    (0, 128, 0),
    (128, 128, 0),
    (0, 0, 128),
    (128, 0, 128),
    (0, 128, 128),
    (192, 192, 192),
    (128, 128, 128),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (0, 0, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];
const ANSI: [AnsiColors; 16] = [
    AnsiColors::Black,
    AnsiColors::Red,
    AnsiColors::Green,
    AnsiColors::Yellow,
    AnsiColors::Blue,
    AnsiColors::Magenta,
    AnsiColors::Cyan,
    AnsiColors::White,
    AnsiColors::BrightBlack,
    AnsiColors::BrightRed,
    AnsiColors::BrightGreen,
    AnsiColors::BrightYellow,
    AnsiColors::BrightBlue,
    AnsiColors::BrightMagenta,
    AnsiColors::BrightCyan,
    AnsiColors::BrightWhite,
];

fn parse_color(word: &str) -> Option<Col> {
    if word == "default" {
        return Some(Col::Default);
    }
    if let Some(i) = NAMES.iter().position(|n| *n == word) {
        return Some(Col::Std(i as u8));
    }
    if let Some(i) = word
        .strip_prefix("bright_")
        .and_then(|w| NAMES.iter().position(|n| *n == w))
    {
        return Some(Col::Std(8 + i as u8));
    }
    if word == "grey50" {
        return Some(Col::Idx(244));
    }
    if let Some(hex) = word.strip_prefix('#').filter(|h| h.len() == 6) {
        let v = u32::from_str_radix(hex, 16).ok()?;
        return Some(Col::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8));
    }
    if let Some(n) = word
        .strip_prefix("color(")
        .and_then(|w| w.strip_suffix(')'))
    {
        let n: u8 = n.parse().ok()?;
        return Some(if n < 16 { Col::Std(n) } else { Col::Idx(n) });
    }
    let rgb = word.strip_prefix("rgb(")?.strip_suffix(')')?;
    let p: Vec<u8> = rgb
        .split(',')
        .filter_map(|v| v.trim().parse().ok())
        .collect();
    (p.len() == 3).then(|| Col::Rgb(p[0], p[1], p[2]))
}

pub fn parse_style(s: &str) -> St {
    let mut st = St::default();
    let mut on = false;
    for w in s.split_whitespace() {
        match w {
            "bold" | "b" => st.bold = true,
            "dim" | "d" => st.dim = true,
            "italic" | "i" => st.italic = true,
            "underline" | "u" => st.underline = true,
            "strike" | "s" => st.strike = true,
            "reverse" | "r" => st.reverse = true,
            "on" => on = true,
            _ => {
                if let Some(c) = parse_color(w) {
                    if on {
                        st.bg = Some(c)
                    } else {
                        st.fg = Some(c)
                    }
                    on = false;
                }
            }
        }
    }
    st
}

impl St {
    pub fn over(self, o: St) -> St {
        St {
            bold: self.bold || o.bold,
            dim: self.dim || o.dim,
            italic: self.italic || o.italic,
            underline: self.underline || o.underline,
            strike: self.strike || o.strike,
            reverse: self.reverse || o.reverse,
            fg: o.fg.or(self.fg),
            bg: o.bg.or(self.bg),
        }
    }
    pub fn is_plain(&self) -> bool {
        *self == St::default()
    }
}

fn rgb_of(c: Col) -> (u8, u8, u8) {
    match c {
        Col::Rgb(r, g, b) => (r, g, b),
        Col::Std(i) => XTERM16[i as usize],
        Col::Idx(n) if n < 16 => XTERM16[n as usize],
        Col::Idx(n) if n >= 232 => (8 + 10 * (n - 232), 8 + 10 * (n - 232), 8 + 10 * (n - 232)),
        Col::Idx(n) => {
            let lv = [0u8, 95, 135, 175, 215, 255];
            let k = (n - 16) as usize;
            (lv[k / 36], lv[k / 6 % 6], lv[k % 6])
        }
        Col::Default => (0, 0, 0),
    }
}

fn nearest_std(c: (u8, u8, u8)) -> u8 {
    let d = |p: (u8, u8, u8)| {
        let (r1, g1, b1) = (c.0 as i32, c.1 as i32, c.2 as i32);
        let (r2, g2, b2) = (p.0 as i32, p.1 as i32, p.2 as i32);
        let mean = (r1 + r2) / 2;
        let (r, g, b) = (r1 - r2, g1 - g2, b1 - b2);
        (((512 + mean) * r * r) >> 8) + 4 * g * g + (((767 - mean) * b * b) >> 8)
    };
    (0..16).min_by_key(|i| d(STD16[*i])).unwrap() as u8
}

fn rgb_to_idx(r: u8, g: u8, b: u8) -> u8 {
    let (rf, gf, bf) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let (hi, lo) = (rf.max(gf).max(bf), rf.min(gf).min(bf));
    let l = (hi + lo) / 2.0;
    let s = if hi == lo {
        0.0
    } else if l <= 0.5 {
        (hi - lo) / (hi + lo)
    } else {
        (hi - lo) / (2.0 - hi - lo)
    };
    if s < 0.15 {
        let gray = (l * 25.0).round_ties_even() as u8;
        return match gray {
            0 => 16,
            25 => 231,
            g => 231 + g,
        };
    }
    let six = |v: u8| {
        if v < 95 {
            v as f64 / 95.0
        } else {
            1.0 + (v as f64 - 95.0) / 40.0
        }
    };
    16 + 36 * six(r).round_ties_even() as u8
        + 6 * six(g).round_ties_even() as u8
        + six(b).round_ties_even() as u8
}

pub fn downgrade(c: Col, d: Depth) -> Col {
    match (c, d) {
        (Col::Default, _) | (_, Depth::True) | (Col::Std(_), _) => c,
        (Col::Rgb(r, g, b), Depth::Idx) => Col::Idx(rgb_to_idx(r, g, b)),
        (Col::Idx(_), Depth::Idx) => c,
        (_, Depth::Std) => Col::Std(nearest_std(rgb_of(c))),
        (_, Depth::None) => c,
    }
}

fn dyn_color(c: Col, d: Depth) -> DynColors {
    match downgrade(c, d) {
        Col::Default => DynColors::Ansi(AnsiColors::Default),
        Col::Std(i) => DynColors::Ansi(ANSI[i as usize]),
        Col::Idx(n) => DynColors::Xterm(XtermColors::from(n)),
        Col::Rgb(r, g, b) => DynColors::Rgb(r, g, b),
    }
}

pub fn owo_style(st: &St, d: Depth) -> Style {
    let mut s = Style::new();
    if st.bold {
        s = s.bold();
    }
    if st.dim {
        s = s.dimmed();
    }
    if st.italic {
        s = s.italic();
    }
    if st.underline {
        s = s.underline();
    }
    if st.strike {
        s = s.strikethrough();
    }
    if st.reverse {
        s = s.reversed();
    }
    if let Some(c) = st.fg {
        s = s.color(dyn_color(c, d));
    }
    if let Some(c) = st.bg {
        s = s.on_color(dyn_color(c, d));
    }
    s
}

pub fn paint(text: &str, st: &St, d: Depth) -> String {
    if d == Depth::None || st.is_plain() || text.is_empty() {
        return text.to_string();
    }
    owo_style(st, d).style(text).to_string()
}
