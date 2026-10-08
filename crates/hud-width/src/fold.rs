//! Folding, truncating and padding on cluster boundaries.

use core::fmt;

use crate::segment::clusters;
use crate::width::{cell_width, cell_width_of_cluster};

/// Iterator over the lines of a greedy hard fold, created by [`fold`].
#[derive(Clone, Debug)]
pub struct Fold<'a> {
    rest: &'a str,
    width: usize,
    emitted: bool,
}

/// Folds `text` into lines at most `width` cells wide, cutting only between clusters.
///
/// A cluster wider than `width` goes alone on its line, zero-width clusters stay on the
/// current line, and concatenating the lines gives back the input. Empty input yields one
/// empty line.
///
/// ```
/// let lines: Vec<&str> = hud_width::fold("ab你cd", 3).collect();
/// assert_eq!(lines, ["ab", "你c", "d"]);
/// ```
pub fn fold(text: &str, width: usize) -> Fold<'_> {
    Fold {
        rest: text,
        width,
        emitted: false,
    }
}

impl<'a> Iterator for Fold<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() {
            if self.emitted {
                return None;
            }
            self.emitted = true;
            return Some("");
        }
        self.emitted = true;
        let mut end = 0;
        let mut used = 0;
        for cluster in clusters(self.rest) {
            let w = cell_width_of_cluster(cluster);
            if end > 0 && used + w > self.width {
                break;
            }
            end += cluster.len();
            used += w;
        }
        let (line, rest) = self.rest.split_at(end);
        self.rest = rest;
        Some(line)
    }
}

/// Returns the longest prefix of `text` that fits in `width` cells without splitting a
/// cluster. This is the first line of [`fold`], so a first cluster wider than `width`
/// is returned whole.
///
/// ```
/// assert_eq!(hud_width::truncate("你好世界", 5), "你好");
/// assert_eq!(hud_width::truncate("🇧🇷🇺🇸", 3), "🇧🇷");
/// ```
pub fn truncate(text: &str, width: usize) -> &str {
    fold(text, width).next().unwrap_or("")
}

/// Horizontal placement of text inside a padded field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// Text at the start, fill after it.
    Left,
    /// Text in the middle, the extra cell (if any) goes after it.
    Center,
    /// Text at the end, fill before it.
    Right,
}

/// A value that displays `text` padded with spaces to a cell width, created by [`pad`].
#[derive(Clone, Copy, Debug)]
pub struct Pad<'a> {
    text: &'a str,
    width: usize,
    align: Align,
}

/// Pads `text` with spaces up to `width` cells. Text already at least that wide is shown as is.
///
/// ```
/// use hud_width::{Align, pad};
/// assert_eq!(pad("你", 4, Align::Right).to_string(), "  你");
/// assert_eq!(pad("ab", 5, Align::Center).to_string(), " ab  ");
/// ```
pub fn pad(text: &str, width: usize, align: Align) -> Pad<'_> {
    Pad { text, width, align }
}

impl fmt::Display for Pad<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fill = self.width.saturating_sub(cell_width(self.text));
        let before = match self.align {
            Align::Left => 0,
            Align::Center => fill / 2,
            Align::Right => fill,
        };
        for _ in 0..before {
            f.write_str(" ")?;
        }
        f.write_str(self.text)?;
        for _ in 0..fill - before {
            f.write_str(" ")?;
        }
        Ok(())
    }
}
