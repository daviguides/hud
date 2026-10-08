use core::ops::Range;

use super::style::Style;

/// A styled stretch of a [`Text`], as byte offsets into its plain string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// First byte of the span.
    pub start: usize,
    /// One past the last byte of the span.
    pub end: usize,
    /// The style applied over `start..end`.
    pub style: Style,
}

/// How a line is placed inside the width it is printed at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    /// Left aligned without padding.
    #[default]
    Default,
    /// Left aligned, padded on the right to the full width.
    Left,
    /// Centered.
    Center,
    /// Right aligned.
    Right,
    /// Spaces between words stretched so every line but the last fills the width.
    Full,
}

/// What happens to a line that is too wide for the width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    /// Words longer than a line are folded onto more lines.
    #[default]
    Fold,
    /// The excess is cut off.
    Crop,
    /// The excess is cut off and replaced by an ellipsis.
    Ellipsis,
    /// The line is left as it is.
    Ignore,
}

/// Text with styled spans, ready to be wrapped, justified and printed.
///
/// ```
/// use hud::{Style, Text};
///
/// let mut text = Text::new("Deploy ok");
/// text.stylize(0..6, Style::new().bold());
/// assert_eq!(text.plain(), "Deploy ok");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    pub(crate) plain: String,
    pub(crate) style: Style,
    pub(crate) spans: Vec<Span>,
    pub(crate) justify: Justify,
    pub(crate) overflow: Overflow,
    pub(crate) no_wrap: bool,
    pub(crate) tab_size: usize,
    pub(crate) end: String,
}

/// Removes the control characters that would corrupt a terminal line: bell, backspace,
/// vertical tab, form feed and carriage return.
pub(crate) fn strip_controls(text: String) -> String {
    if text.bytes().any(|b| matches!(b, 7 | 8 | 11 | 12 | 13)) {
        text.chars()
            .filter(|c| !matches!(*c, '\u{7}' | '\u{8}' | '\u{b}' | '\u{c}' | '\r'))
            .collect()
    } else {
        text
    }
}

impl Default for Text {
    fn default() -> Text {
        Text::new("")
    }
}

impl Text {
    /// Plain text with no style.
    pub fn new(plain: impl Into<String>) -> Text {
        Text {
            plain: strip_controls(plain.into()),
            style: Style::new(),
            spans: Vec::new(),
            justify: Justify::Default,
            overflow: Overflow::Fold,
            no_wrap: false,
            tab_size: 8,
            end: "\n".to_string(),
        }
    }

    /// Text where `style` covers every character.
    pub fn styled(plain: impl Into<String>, style: Style) -> Text {
        let mut text = Text::new(plain);
        text.style = style;
        text
    }

    /// The text without styles.
    pub fn plain(&self) -> &str {
        &self.plain
    }

    /// The style that covers the whole text.
    pub fn style(&self) -> &Style {
        &self.style
    }

    /// The styled spans, in the order they were added.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Appends `text` styled with `style`.
    pub fn append(&mut self, text: &str, style: Style) {
        let start = self.plain.len();
        self.plain.push_str(&strip_controls(text.to_string()));
        if !style.is_null() && !text.is_empty() {
            self.spans.push(Span {
                start,
                end: self.plain.len(),
                style,
            });
        }
    }

    /// Applies `style` over a byte range. The range is clamped to the text and moved outward to
    /// the nearest character boundaries; an empty range does nothing.
    pub fn stylize(&mut self, range: Range<usize>, style: Style) {
        let len = self.plain.len();
        let mut start = range.start.min(len);
        let mut end = range.end.min(len);
        while !self.plain.is_char_boundary(start) {
            start -= 1;
        }
        while !self.plain.is_char_boundary(end) {
            end += 1;
        }
        if start < end && !style.is_null() {
            self.spans.push(Span { start, end, style });
        }
    }

    /// Sets how lines are placed inside the print width.
    #[must_use]
    pub fn justify(mut self, justify: Justify) -> Text {
        self.justify = justify;
        self
    }

    /// Sets what happens to lines that are too wide.
    #[must_use]
    pub fn overflow(mut self, overflow: Overflow) -> Text {
        self.overflow = overflow;
        self
    }

    /// Turns word wrapping off: each line is cut by the overflow setting instead.
    #[must_use]
    pub fn no_wrap(mut self, no_wrap: bool) -> Text {
        self.no_wrap = no_wrap;
        self
    }

    /// Sets the distance between tab stops (default 8).
    #[must_use]
    pub fn tab_size(mut self, tab_size: usize) -> Text {
        self.tab_size = tab_size;
        self
    }

    /// Sets the text printed after the last line (default a newline).
    #[must_use]
    pub fn end(mut self, end: impl Into<String>) -> Text {
        self.end = end.into();
        self
    }
}
