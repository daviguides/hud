//! Color parsing, downgrade to a shallower color system, and escape parameters.

use crate::model::palette::{EIGHT_BIT, NAMES, STANDARD};
use crate::model::{Color, ColorSystem, StyleError};

/// A parsed color together with the lowercase word it was written as, which style
/// normalization needs to compare tags.
pub(crate) struct ParsedColor {
    pub(crate) color: Color,
    pub(crate) word: String,
}

fn invalid(word: &str) -> StyleError {
    StyleError::new(format!("unable to parse '{word}' as color"))
}

/// Parses a color word: `default`, a name, `color(n)`, `#rrggbb` or `rgb(r,g,b)`.
pub(crate) fn parse_color(input: &str) -> Result<ParsedColor, StyleError> {
    let word = input.trim().to_lowercase();
    let color = parse_word(&word).ok_or_else(|| invalid(input))?;
    Ok(ParsedColor { color, word })
}

fn parse_word(word: &str) -> Option<Color> {
    if word == "default" {
        return Some(Color::Default);
    }
    if let Ok(index) = NAMES.binary_search_by(|(name, _)| (*name).cmp(word)) {
        return Some(Color::indexed(NAMES[index].1));
    }
    if let Some(hex) = word.strip_prefix('#') {
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        return Some(Color::TrueColor(channel(0)?, channel(2)?, channel(4)?));
    }
    if let Some(number) = word
        .strip_prefix("color(")
        .and_then(|r| r.strip_suffix(')'))
    {
        if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        return number.parse::<u8>().ok().map(Color::indexed);
    }
    if let Some(inner) = word.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
        if inner.is_empty()
            || !inner
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b',' || b.is_ascii_whitespace())
        {
            return None;
        }
        let mut channels = [0u8; 3];
        let mut parts = inner.split(',');
        for slot in &mut channels {
            let part = parts.next()?.trim();
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            *slot = part.parse::<u8>().ok()?;
        }
        if parts.next().is_some() {
            return None;
        }
        return Some(Color::TrueColor(channels[0], channels[1], channels[2]));
    }
    None
}

fn hls_saturation_lightness(red: u8, green: u8, blue: u8) -> (f64, f64) {
    let (r, g, b) = (
        f64::from(red) / 255.0,
        f64::from(green) / 255.0,
        f64::from(blue) / 255.0,
    );
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (min + max) / 2.0;
    if min == max {
        return (0.0, lightness);
    }
    let span = max - min;
    let saturation = if lightness <= 0.5 {
        span / (max + min)
    } else {
        span / (2.0 - max - min)
    };
    (saturation, lightness)
}

fn cube_step(channel: u8) -> f64 {
    let value = f64::from(channel);
    if channel < 95 {
        value / 95.0
    } else {
        1.0 + (value - 95.0) / 40.0
    }
}

fn truecolor_to_eight_bit(red: u8, green: u8, blue: u8) -> u8 {
    let (saturation, lightness) = hls_saturation_lightness(red, green, blue);
    if saturation < 0.15 {
        let gray = (lightness * 25.0).round_ties_even() as i32;
        return match gray {
            0 => 16,
            25 => 231,
            _ => (231 + gray) as u8,
        };
    }
    let index = |channel: u8| cube_step(channel).round_ties_even() as u32;
    (16 + 36 * index(red) + 6 * index(green) + index(blue)) as u8
}

fn nearest_standard(red: u8, green: u8, blue: u8) -> u8 {
    let (r1, g1, b1) = (i64::from(red), i64::from(green), i64::from(blue));
    let mut best = 0u8;
    let mut best_distance = i64::MAX;
    for (index, &(r2, g2, b2)) in STANDARD.iter().enumerate() {
        let (r2, g2, b2) = (i64::from(r2), i64::from(g2), i64::from(b2));
        let red_mean = (r1 + r2) / 2;
        let (dr, dg, db) = (r1 - r2, g1 - g2, b1 - b2);
        let distance =
            (((512 + red_mean) * dr * dr) >> 8) + 4 * dg * dg + (((767 - red_mean) * db * db) >> 8);
        if distance < best_distance {
            best_distance = distance;
            best = index as u8;
        }
    }
    best
}

/// Converts `color` to what `system` can show: truecolor goes to the 256 palette or to the 16
/// standard colors, the 256 palette goes to the 16 standard colors, everything else stays.
pub(crate) fn downgrade(color: Color, system: ColorSystem) -> Color {
    match (color, system) {
        (Color::TrueColor(r, g, b), ColorSystem::EightBit) => {
            Color::EightBit(truecolor_to_eight_bit(r, g, b))
        }
        (Color::TrueColor(r, g, b), ColorSystem::Standard) => {
            Color::Standard(nearest_standard(r, g, b))
        }
        (Color::EightBit(n), ColorSystem::Standard) => {
            let (r, g, b) = EIGHT_BIT[usize::from(n)];
            Color::Standard(nearest_standard(r, g, b))
        }
        _ => color,
    }
}

/// Appends the SGR parameters of `color`, with a `;` first when `out` already has some.
pub(crate) fn push_codes(out: &mut String, color: Color, foreground: bool) {
    use core::fmt::Write;

    if !out.is_empty() {
        out.push(';');
    }
    let (base, bright, extended) = if foreground {
        (30u16, 90u16, 38u16)
    } else {
        (40, 100, 48)
    };
    // Writing into a String cannot fail.
    let _ = match color {
        Color::Default => write!(out, "{}", base + 9),
        Color::Standard(n) if n < 8 => write!(out, "{}", base + u16::from(n)),
        Color::Standard(n) => write!(out, "{}", bright + u16::from(n) - 8),
        Color::EightBit(n) => write!(out, "{extended};5;{n}"),
        Color::TrueColor(r, g, b) => write!(out, "{extended};2;{r};{g};{b}"),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_hex_index_and_rgb_parse() {
        let parse = |s: &str| parse_color(s).map(|p| p.color).ok();
        assert_eq!(parse("red"), Some(Color::Standard(1)));
        assert_eq!(parse("Bright_Blue"), Some(Color::Standard(12)));
        assert_eq!(parse("grey50"), Some(Color::EightBit(244)));
        assert_eq!(parse("#FF8800"), Some(Color::TrueColor(255, 136, 0)));
        assert_eq!(parse("color(5)"), Some(Color::Standard(5)));
        assert_eq!(parse("color(208)"), Some(Color::EightBit(208)));
        assert_eq!(
            parse("rgb(12, 200,99)"),
            Some(Color::TrueColor(12, 200, 99))
        );
        assert_eq!(parse("default"), Some(Color::Default));
        for bad in [
            "#fff",
            "color(256)",
            "rgb(1,2)",
            "rgb(1,2,3,4)",
            "rgb(300,0,0)",
            "nope",
            "",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn downgrade_follows_the_palettes() {
        assert_eq!(
            downgrade(Color::TrueColor(128, 128, 128), ColorSystem::EightBit),
            Color::EightBit(244)
        );
        assert_eq!(
            downgrade(Color::TrueColor(0, 0, 0), ColorSystem::EightBit),
            Color::EightBit(16)
        );
        assert_eq!(
            downgrade(Color::TrueColor(255, 255, 255), ColorSystem::EightBit),
            Color::EightBit(231)
        );
        assert_eq!(
            downgrade(Color::Standard(3), ColorSystem::EightBit),
            Color::Standard(3)
        );
        assert_eq!(
            downgrade(Color::EightBit(100), ColorSystem::TrueColor),
            Color::EightBit(100)
        );
    }

    #[test]
    fn codes_use_the_bright_range_for_colors_8_to_15() {
        let mut out = String::new();
        push_codes(&mut out, Color::Standard(9), true);
        push_codes(&mut out, Color::Standard(2), false);
        push_codes(&mut out, Color::Default, true);
        assert_eq!(out, "91;42;39");
    }
}
