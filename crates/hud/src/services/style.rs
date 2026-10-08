//! Style strings, their canonical form, and the escape sequences a style turns into.

use core::fmt;
use core::str::FromStr;

use super::color::{ParsedColor, downgrade, parse_color, push_codes};
use crate::model::{Attribute, ColorSystem, Style, StyleError};

/// A parsed style and its canonical text, used to decide whether two tags are the same style.
pub(crate) struct ParsedStyle {
    pub(crate) style: Style,
    pub(crate) canonical: String,
}

fn attribute_for(word: &str) -> Option<Attribute> {
    Some(match word {
        "bold" | "b" => Attribute::Bold,
        "dim" | "d" => Attribute::Dim,
        "italic" | "i" => Attribute::Italic,
        "underline" | "u" => Attribute::Underline,
        "blink" => Attribute::Blink,
        "blink2" => Attribute::Blink2,
        "reverse" | "r" => Attribute::Reverse,
        "conceal" | "c" => Attribute::Conceal,
        "strike" | "s" => Attribute::Strike,
        "underline2" | "uu" => Attribute::Underline2,
        "frame" => Attribute::Frame,
        "encircle" => Attribute::Encircle,
        "overline" | "o" => Attribute::Overline,
        _ => return None,
    })
}

fn canonical_form(style: &Style, fg: Option<&str>, bg: Option<&str>) -> String {
    let mut words: Vec<String> = Vec::new();
    for attribute in Attribute::ALL {
        match style.get(attribute) {
            Some(true) => words.push(attribute.word().to_string()),
            Some(false) => words.push(format!("not {}", attribute.word())),
            None => {}
        }
    }
    if let Some(fg) = fg {
        words.push(fg.to_string());
    }
    if let Some(bg) = bg {
        words.push("on".to_string());
        words.push(bg.to_string());
    }
    if let Some(url) = style.link_url() {
        words.push("link".to_string());
        words.push(url.to_string());
    }
    if words.is_empty() {
        "none".to_string()
    } else {
        words.join(" ")
    }
}

/// Parses a style string such as `bold red on #223344`, `not italic` or `link https://x.org`.
pub(crate) fn parse(definition: &str) -> Result<ParsedStyle, StyleError> {
    let mut style = Style::new();
    if definition.trim() == "none" || definition.is_empty() {
        return Ok(ParsedStyle {
            style,
            canonical: "none".to_string(),
        });
    }
    let mut fg_word: Option<String> = None;
    let mut bg_word: Option<String> = None;
    let mut words = definition.split_whitespace();
    while let Some(original) = words.next() {
        let word = original.to_lowercase();
        match word.as_str() {
            "on" => {
                let next = words
                    .next()
                    .ok_or_else(|| StyleError::new("color expected after 'on'"))?;
                let ParsedColor { color, word } = parse_color(next)?;
                style.bg = Some(color);
                bg_word = Some(word);
            }
            "not" => {
                let next = words
                    .next()
                    .ok_or_else(|| StyleError::new("attribute expected after 'not'"))?;
                let attribute = attribute_for(&next.to_lowercase())
                    .ok_or_else(|| StyleError::new(format!("unknown attribute '{next}'")))?;
                style = style.attribute(attribute, false);
            }
            "link" => {
                let url = words
                    .next()
                    .ok_or_else(|| StyleError::new("URL expected after 'link'"))?;
                style.link = Some(url.to_string());
            }
            other => {
                if let Some(attribute) = attribute_for(other) {
                    style = style.attribute(attribute, true);
                } else {
                    let ParsedColor { color, word } = parse_color(original).map_err(|_| {
                        StyleError::new(format!(
                            "unable to parse '{original}' in style '{definition}'"
                        ))
                    })?;
                    style.fg = Some(color);
                    fg_word = Some(word);
                }
            }
        }
    }
    let canonical = canonical_form(&style, fg_word.as_deref(), bg_word.as_deref());
    Ok(ParsedStyle { style, canonical })
}

impl Style {
    /// Parses a style string: attribute words (`bold`, `dim`, `italic`, `underline`, `strike`,
    /// `reverse`, `blink` and their one-letter forms), a foreground color, `on` and a
    /// background color, `not` and an attribute to switch it off, `link` and a URL.
    ///
    /// Colors are names (`red`, `bright_blue`, `grey50`), `default`, `color(208)`, `#ff8800`
    /// or `rgb(255,136,0)`.
    ///
    /// ```
    /// use hud::{Color, Style};
    ///
    /// let style = Style::parse("bold #ff8800 on blue")?;
    /// assert_eq!(style, Style::new().bold().color(Color::rgb(255, 136, 0)).bgcolor(Color::BLUE));
    /// assert!(Style::parse("bold sparkly").is_err());
    /// # Ok::<(), hud::StyleError>(())
    /// ```
    pub fn parse(definition: &str) -> Result<Style, StyleError> {
        parse(definition).map(|parsed| parsed.style)
    }
}

impl FromStr for Style {
    type Err = StyleError;

    fn from_str(definition: &str) -> Result<Style, StyleError> {
        Style::parse(definition)
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fg = self.fg.map(|c| c.to_string());
        let bg = self.bg.map(|c| c.to_string());
        f.write_str(&canonical_form(self, fg.as_deref(), bg.as_deref()))
    }
}

/// The SGR parameters (`1;3;38;5;244`) `style` needs on a stream that shows `color_system`
/// colors and, when `attributes` is set, text attributes. Empty when nothing is to be emitted.
pub(crate) fn sgr_codes(style: &Style, color_system: ColorSystem, attributes: bool) -> String {
    let mut out = String::new();
    if attributes {
        for attribute in Attribute::ALL {
            if style.get(attribute) == Some(true) {
                if !out.is_empty() {
                    out.push(';');
                }
                out.push_str(attribute.sgr());
            }
        }
    }
    if color_system != ColorSystem::None {
        if let Some(color) = style.fg {
            push_codes(&mut out, downgrade(color, color_system), true);
        }
        if let Some(color) = style.bg {
            push_codes(&mut out, downgrade(color, color_system), false);
        }
    }
    out
}

/// Appends `text` to `out` wrapped in the escape sequences of `style`.
pub(crate) fn emit(
    out: &mut String,
    text: &str,
    style: &Style,
    color_system: ColorSystem,
    attributes: bool,
) {
    if text.is_empty() {
        return;
    }
    let link = if attributes { style.link_url() } else { None };
    if let Some(url) = link {
        out.push_str("\x1b]8;;");
        out.push_str(url);
        out.push_str("\x1b\\");
    }
    let codes = sgr_codes(style, color_system, attributes);
    if codes.is_empty() {
        out.push_str(text);
    } else {
        out.push_str("\x1b[");
        out.push_str(&codes);
        out.push('m');
        out.push_str(text);
        out.push_str("\x1b[0m");
    }
    if link.is_some() {
        out.push_str("\x1b]8;;\x1b\\");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Color;

    #[test]
    fn parse_builds_the_same_style_as_the_builder() {
        let parsed = Style::parse("Bold ITALIC red on #223344").unwrap();
        let built = Style::new()
            .bold()
            .italic()
            .color(Color::RED)
            .bgcolor(Color::rgb(0x22, 0x33, 0x44));
        assert_eq!(parsed, built);
        assert_eq!(Style::parse("").unwrap(), Style::new());
        assert_eq!(Style::parse("none").unwrap(), Style::new());
    }

    #[test]
    fn errors_name_the_problem() {
        for bad in ["on", "not", "link", "not red", "bold on sparkly", "sparkly"] {
            assert!(Style::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn canonical_form_is_stable_across_spellings() {
        let a = parse("b i red").unwrap().canonical;
        let b = parse("red italic bold").unwrap().canonical;
        assert_eq!(a, "bold italic red");
        assert_eq!(a, b);
        assert_eq!(
            parse("not bold on blue").unwrap().canonical,
            "not bold on blue"
        );
    }

    #[test]
    fn codes_follow_attribute_order_then_colors() {
        let style = Style::parse("italic bold #808080 on rgb(250,240,10)").unwrap();
        assert_eq!(
            sgr_codes(&style, ColorSystem::EightBit, true),
            "1;3;38;5;244;48;5;226"
        );
        assert_eq!(sgr_codes(&style, ColorSystem::None, true), "1;3");
        assert_eq!(sgr_codes(&style, ColorSystem::None, false), "");
    }

    #[test]
    fn off_attributes_emit_nothing() {
        let style = Style::new().bold().attribute(Attribute::Bold, false);
        assert_eq!(sgr_codes(&style, ColorSystem::TrueColor, true), "");
    }

    #[test]
    fn emit_wraps_with_reset_and_links() {
        let mut out = String::new();
        emit(
            &mut out,
            "x",
            &Style::new().bold().link("https://x.org"),
            ColorSystem::None,
            true,
        );
        assert_eq!(
            out,
            "\x1b]8;;https://x.org\x1b\\\x1b[1mx\x1b[0m\x1b]8;;\x1b\\"
        );
    }
}
