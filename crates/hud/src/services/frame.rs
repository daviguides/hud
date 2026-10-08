//! Pieces shared by the widgets that draw boxes: the eight Rich box rows, rules, runs, and
//! lines cut to an exact width.

use hud_width::cell_width;

use super::measure::set_cell_size;
use crate::model::{BoxStyle, Segment, Style};

/// The eight rows of a box: top, header, under the header, body, between rows, above the
/// footer, footer and bottom, each as left, horizontal (or filler), divider and right.
pub(crate) fn box_rows(style: BoxStyle) -> [[char; 4]; 8] {
    let rows: [&str; 8] = match style {
        BoxStyle::Ascii => [
            "+--+", "| ||", "|-+|", "| ||", "|-+|", "|-+|", "| ||", "+--+",
        ],
        BoxStyle::Rounded => [
            "╭─┬╮",
            "│ ││",
            "├─┼┤",
            "│ ││",
            "├─┼┤",
            "├─┼┤",
            "│ ││",
            "╰─┴╯",
        ],
        BoxStyle::Simple => [
            "    ", "    ", " ── ", "    ", "    ", " ── ", "    ", "    ",
        ],
        BoxStyle::Heavy => [
            "┏━┳┓",
            "┃ ┃┃",
            "┣━╋┫",
            "┃ ┃┃",
            "┣━╋┫",
            "┣━╋┫",
            "┃ ┃┃",
            "┗━┻┛",
        ],
        BoxStyle::Double => [
            "╔═╦╗",
            "║ ║║",
            "╠═╬╣",
            "║ ║║",
            "╠═╬╣",
            "╠═╬╣",
            "║ ║║",
            "╚═╩╝",
        ],
        BoxStyle::Minimal => [
            "  ╷ ",
            "  │ ",
            "╶─┼╴",
            "  │ ",
            "╶─┼╴",
            "╶─┼╴",
            "  │ ",
            "  ╵ ",
        ],
        BoxStyle::Square => [
            "┌─┬┐",
            "│ ││",
            "├─┼┤",
            "│ ││",
            "├─┼┤",
            "├─┼┤",
            "│ ││",
            "└─┴┘",
        ],
        BoxStyle::HeavyHead => [
            "┏━┳┓",
            "┃ ┃┃",
            "┡━╇┩",
            "│ ││",
            "├─┼┤",
            "├─┼┤",
            "│ ││",
            "└─┴┘",
        ],
    };
    let mut out = [[' '; 4]; 8];
    for (row, text) in out.iter_mut().zip(rows) {
        for (slot, c) in row.iter_mut().zip(text.chars()) {
            *slot = c;
        }
    }
    out
}

pub(crate) const TOP: usize = 0;
pub(crate) const HEAD: usize = 1;
pub(crate) const HEAD_ROW: usize = 2;
pub(crate) const MID: usize = 3;
pub(crate) const ROW: usize = 4;
pub(crate) const FOOT: usize = 6;
pub(crate) const BOTTOM: usize = 7;

/// A horizontal rule: `parts` are left, filler, divider and right, over columns of `widths`.
pub(crate) fn rule(parts: [char; 4], widths: &[usize]) -> String {
    let mut out = String::new();
    out.push(parts[0]);
    for (index, &width) in widths.iter().enumerate() {
        out.extend(core::iter::repeat_n(parts[1], width));
        if index + 1 < widths.len() {
            out.push(parts[2]);
        }
    }
    out.push(parts[3]);
    out
}

pub(crate) fn run(text: impl Into<String>, style: &Style) -> Segment {
    Segment {
        text: text.into(),
        style: style.clone(),
    }
}

/// Pads or crops one line of runs to exactly `width` cells; padding takes `style`.
pub(crate) fn adjust_line(line: &mut Vec<Segment>, width: usize, style: &Style) {
    let length: usize = line.iter().map(|s| cell_width(&s.text)).sum();
    if length < width {
        line.push(run(" ".repeat(width - length), style));
    } else if length > width {
        let mut used = 0;
        let mut cropped = Vec::with_capacity(line.len());
        for segment in line.drain(..) {
            let cells = cell_width(&segment.text);
            if used + cells < width {
                used += cells;
                cropped.push(segment);
            } else {
                cropped.push(Segment {
                    text: set_cell_size(&segment.text, width - used),
                    style: segment.style,
                });
                break;
            }
        }
        *line = cropped;
    }
}

/// Splits runs at newlines into lines of runs, each adjusted to exactly `width` cells: shorter
/// lines are padded with `style`, longer ones cropped. A final line with no newline counts.
pub(crate) fn split_lines(segments: Vec<Segment>, width: usize, style: &Style) -> Vec<Vec<Segment>> {
    let mut lines = Vec::new();
    let mut line: Vec<Segment> = Vec::new();
    for segment in segments {
        if !segment.text.contains('\n') {
            line.push(segment);
            continue;
        }
        let mut rest = segment.text.as_str();
        while !rest.is_empty() {
            let (piece, newline, tail) = match rest.split_once('\n') {
                Some((piece, tail)) => (piece, true, tail),
                None => (rest, false, ""),
            };
            if !piece.is_empty() {
                line.push(run(piece, &segment.style));
            }
            if newline {
                adjust_line(&mut line, width, style);
                lines.push(core::mem::take(&mut line));
            }
            rest = tail;
        }
    }
    if !line.is_empty() {
        adjust_line(&mut line, width, style);
        lines.push(line);
    }
    lines
}
