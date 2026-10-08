//! Per-code-point properties read from the generated two-stage table.

use crate::tables::{BLOCK_SHIFT, STAGE1, STAGE2};

/// `Grapheme_Cluster_Break` property values (UAX #29).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Gcb {
    Other,
    Cr,
    Lf,
    Control,
    Extend,
    Zwj,
    RegionalIndicator,
    Prepend,
    SpacingMark,
    L,
    V,
    T,
    Lv,
    Lvt,
}

/// `Indic_Conjunct_Break` property values (UAX #44, used by rule GB9c).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Incb {
    None,
    Linker,
    Consonant,
    Extend,
}

/// Packed properties of one code point.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Props(u16);

const BLOCK_MASK: usize = (1 << BLOCK_SHIFT) - 1;

pub(crate) fn props(c: char) -> Props {
    let cp = c as usize;
    let block = usize::from(STAGE1[cp >> BLOCK_SHIFT]);
    Props(STAGE2[(block << BLOCK_SHIFT) | (cp & BLOCK_MASK)])
}

impl Props {
    pub(crate) fn gcb(self) -> Gcb {
        match self.0 & 0xF {
            1 => Gcb::Cr,
            2 => Gcb::Lf,
            3 => Gcb::Control,
            4 => Gcb::Extend,
            5 => Gcb::Zwj,
            6 => Gcb::RegionalIndicator,
            7 => Gcb::Prepend,
            8 => Gcb::SpacingMark,
            9 => Gcb::L,
            10 => Gcb::V,
            11 => Gcb::T,
            12 => Gcb::Lv,
            13 => Gcb::Lvt,
            _ => Gcb::Other,
        }
    }

    pub(crate) fn incb(self) -> Incb {
        match (self.0 >> 4) & 0x3 {
            1 => Incb::Linker,
            2 => Incb::Consonant,
            3 => Incb::Extend,
            _ => Incb::None,
        }
    }

    pub(crate) fn is_extended_pictographic(self) -> bool {
        self.0 & (1 << 6) != 0
    }

    pub(crate) fn width(self) -> u8 {
        ((self.0 >> 7) & 0x3) as u8
    }

    pub(crate) fn has_emoji_variation(self) -> bool {
        self.0 & (1 << 9) != 0
    }
}
