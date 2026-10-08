//! Terminal cell width of characters, clusters and strings.

use crate::props::{Gcb, props};
use crate::segment::clusters;

/// Width in terminal cells (0, 1 or 2) of a single code point, ignoring its context.
///
/// ```
/// assert_eq!(hud_width::char_width('a'), 1);
/// assert_eq!(hud_width::char_width('你'), 2);
/// assert_eq!(hud_width::char_width('\u{301}'), 0);
/// ```
pub fn char_width(c: char) -> usize {
    usize::from(props(c).width())
}

/// Width in terminal cells of one grapheme cluster, as produced by [`clusters`].
///
/// A cluster is as wide as its base: combining marks and joiners add nothing, an emoji
/// ZWJ sequence or modifier sequence is one emoji wide, a flag is two cells, and a narrow
/// emoji base followed by `U+FE0F` (emoji presentation) is two cells.
///
/// ```
/// use hud_width::cell_width_of_cluster;
/// assert_eq!(cell_width_of_cluster("e\u{301}"), 1);
/// assert_eq!(cell_width_of_cluster("👨‍👩‍👧‍👦"), 2);
/// assert_eq!(cell_width_of_cluster("1\u{FE0F}\u{20E3}"), 2);
/// ```
pub fn cell_width_of_cluster(cluster: &str) -> usize {
    let mut chars = cluster.chars().peekable();
    let Some(first) = chars.next() else {
        return 0;
    };
    let first_props = props(first);
    if first_props.gcb() == Gcb::RegionalIndicator {
        if let Some(&second) = chars.peek() {
            if props(second).gcb() == Gcb::RegionalIndicator {
                return 2;
            }
        }
    }
    let mut total = usize::from(first_props.width());
    if total == 1 && first_props.has_emoji_variation() && chars.peek() == Some(&'\u{FE0F}') {
        total = 2;
    }
    let mut joined = false;
    for c in chars {
        if c == '\u{200D}' {
            joined = true;
            continue;
        }
        let p = props(c);
        if joined && p.is_extended_pictographic() {
            joined = false;
            continue;
        }
        joined = false;
        total += usize::from(p.width());
    }
    total
}

/// Width in terminal cells of a whole string: the sum of the widths of its clusters.
///
/// Control characters are zero cells wide.
///
/// ```
/// use hud_width::cell_width;
/// assert_eq!(cell_width("hello"), 5);
/// assert_eq!(cell_width("你好"), 4);
/// assert_eq!(cell_width("🇧🇷"), 2);
/// ```
pub fn cell_width(text: &str) -> usize {
    if text.is_ascii() {
        return text.bytes().filter(|b| (0x20..0x7F).contains(b)).count();
    }
    clusters(text).map(cell_width_of_cluster).sum()
}
