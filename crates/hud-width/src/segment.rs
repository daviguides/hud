//! Extended grapheme cluster segmentation (UAX #29, Unicode 17, rules GB1 to GB999
//! including GB9c for Indic conjuncts).

use crate::props::{Gcb, Incb, Props, props};

/// Running state needed to decide whether a boundary exists before the next character.
#[derive(Clone, Copy)]
struct State {
    prev: Props,
    /// Length of the run of regional indicators that ends at the previous character.
    ri_run: u32,
    /// 0: none, 1: `ExtPict Extend*` seen, 2: `ExtPict Extend* ZWJ` seen (rule GB11).
    emoji: u8,
    /// 0: none, 1: `Consonant [Extend Linker]*` seen, 2: the same with a linker (rule GB9c).
    indic: u8,
}

impl State {
    fn start(first: Props) -> Self {
        let mut state = State {
            prev: first,
            ri_run: 0,
            emoji: 0,
            indic: 0,
        };
        state.update(first);
        state
    }

    fn breaks_before(&self, next: Props) -> bool {
        use Gcb::{
            Control, Cr, Extend, L, Lf, Lv, Lvt, Prepend, RegionalIndicator, SpacingMark, T, V, Zwj,
        };
        let prev = self.prev.gcb();
        let next_gcb = next.gcb();
        if prev == Cr && next_gcb == Lf {
            return false;
        }
        if matches!(prev, Control | Cr | Lf) || matches!(next_gcb, Control | Cr | Lf) {
            return true;
        }
        if prev == L && matches!(next_gcb, L | V | Lv | Lvt) {
            return false;
        }
        if matches!(prev, Lv | V) && matches!(next_gcb, V | T) {
            return false;
        }
        if matches!(prev, Lvt | T) && next_gcb == T {
            return false;
        }
        if matches!(next_gcb, Extend | Zwj | SpacingMark) {
            return false;
        }
        if prev == Prepend {
            return false;
        }
        if self.indic == 2 && next.incb() == Incb::Consonant {
            return false;
        }
        if self.emoji == 2 && next.is_extended_pictographic() {
            return false;
        }
        if prev == RegionalIndicator && next_gcb == RegionalIndicator && self.ri_run % 2 == 1 {
            return false;
        }
        true
    }

    fn update(&mut self, next: Props) {
        let gcb = next.gcb();
        self.ri_run = if gcb == Gcb::RegionalIndicator {
            self.ri_run + 1
        } else {
            0
        };
        self.emoji = if next.is_extended_pictographic() || (self.emoji == 1 && gcb == Gcb::Extend) {
            1
        } else if self.emoji == 1 && gcb == Gcb::Zwj {
            2
        } else {
            0
        };
        self.indic = match next.incb() {
            Incb::Consonant => 1,
            Incb::Linker if self.indic >= 1 => 2,
            Incb::Extend if self.indic >= 1 => self.indic,
            _ => 0,
        };
        self.prev = next;
    }
}

/// Iterator over the extended grapheme clusters of a string.
///
/// Created by [`clusters`]. Every item is a non-empty `&str` and concatenating
/// all items gives back the input exactly.
#[derive(Clone, Debug)]
pub struct Clusters<'a> {
    rest: &'a str,
}

/// Splits `text` into extended grapheme clusters (UAX #29, Unicode 17).
///
/// ```
/// let parts: Vec<&str> = hud_width::clusters("e\u{301}a🇧🇷").collect();
/// assert_eq!(parts, ["e\u{301}", "a", "🇧🇷"]);
/// ```
pub fn clusters(text: &str) -> Clusters<'_> {
    Clusters { rest: text }
}

impl<'a> Iterator for Clusters<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let mut chars = self.rest.char_indices();
        let (_, first) = chars.next()?;
        let mut state = State::start(props(first));
        let mut end = self.rest.len();
        for (index, c) in chars {
            let p = props(c);
            if state.breaks_before(p) {
                end = index;
                break;
            }
            state.update(p);
        }
        let (cluster, rest) = self.rest.split_at(end);
        self.rest = rest;
        Some(cluster)
    }
}
