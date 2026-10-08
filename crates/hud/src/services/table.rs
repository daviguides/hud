//! A table spec to styled runs: cell text laid out in its column, the borders drawn around.

use hud_width::cell_width;

use super::layout::{PADDING, column_widths};
use super::measure::set_cell_size;
use super::render::render_text;
use crate::model::{BoxStyle, Column, Justify, Segment, Style, Table, Text};

/// The eight rows of a box: top, header, under the header, body, between rows, above the
/// footer, footer and bottom, each as left, horizontal (or filler), divider and right.
fn box_rows(style: BoxStyle) -> [[char; 4]; 8] {
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

const TOP: usize = 0;
const HEAD: usize = 1;
const HEAD_ROW: usize = 2;
const MID: usize = 3;
const ROW: usize = 4;
const FOOT: usize = 6;
const BOTTOM: usize = 7;

/// A horizontal rule: `parts` are left, filler, divider and right, over columns of `widths`.
fn rule(parts: [char; 4], widths: &[usize]) -> String {
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

fn run(text: impl Into<String>, style: &Style) -> Segment {
    Segment {
        text: text.into(),
        style: style.clone(),
    }
}

/// Pads or crops one line of runs to exactly `width` cells; padding takes `style`.
fn adjust_line(line: &mut Vec<Segment>, width: usize, style: &Style) {
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

/// Text that is printable ASCII: one cell per byte, nothing to wrap or expand.
fn is_plain_ascii(text: &str) -> bool {
    text.bytes().all(|b| (0x20..0x7f).contains(&b))
}

/// The lines of `text` laid out in `inner` cells, each exactly `inner` cells wide.
fn content_lines(text: &Text, inner: usize) -> Vec<Vec<Segment>> {
    let plain = text.plain();
    if text.spans.is_empty()
        && is_plain_ascii(plain)
        && plain.len() <= inner
        && matches!(
            text.justify,
            Justify::Left | Justify::Center | Justify::Right
        )
    {
        let placed = match text.justify {
            Justify::Left => {
                let mut line = String::with_capacity(inner);
                line.push_str(plain);
                line.extend(core::iter::repeat_n(' ', inner - plain.len()));
                line
            }
            justify => {
                let trimmed = plain.trim_end();
                let free = inner - trimmed.len();
                let left = if justify == Justify::Center {
                    free / 2
                } else {
                    free
                };
                let mut line = String::with_capacity(inner);
                line.extend(core::iter::repeat_n(' ', left));
                line.push_str(trimmed);
                line.extend(core::iter::repeat_n(' ', free - left));
                line
            }
        };
        return vec![vec![run(placed, &text.style)]];
    }
    let mut lines = Vec::new();
    let mut line: Vec<Segment> = Vec::new();
    let null = Style::new();
    for segment in render_text(text, inner) {
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
                adjust_line(&mut line, inner, &null);
                lines.push(core::mem::take(&mut line));
            }
            rest = tail;
        }
    }
    if !line.is_empty() {
        adjust_line(&mut line, inner, &null);
        lines.push(line);
    }
    lines
}

/// The lines of one cell: its text with a cell of padding on each side, in `base` over the whole
/// width, each exactly `width` cells wide.
fn cell_lines(text: &Text, width: usize, base: &Style) -> Vec<Vec<Segment>> {
    let pad = PADDING as usize;
    let inner = width.saturating_sub(2 * pad);
    if inner == 0 {
        // A console renders nothing into a width below one, so the cell has no lines of its own.
        return Vec::new();
    }
    let content = content_lines(text, inner);
    let null = Style::new();
    content
        .into_iter()
        .map(|line| {
            let mut full = Vec::with_capacity(line.len() + 2);
            full.push(run(" ".repeat(pad), &null));
            full.extend(line);
            full.push(run(" ".repeat(pad), &null));
            if !base.is_null() {
                for segment in &mut full {
                    segment.style = base.combine(&segment.style);
                }
            }
            adjust_line(&mut full, width, base);
            full
        })
        .collect()
}

fn markup_text(markup: &str) -> Text {
    Text::from_markup(markup).unwrap_or_else(|_| Text::new(markup))
}

fn cell_text(markup: &str, column: &Column) -> Text {
    let mut text = markup_text(markup);
    text.justify = column.justify;
    text.overflow = column.overflow;
    text.no_wrap = column.no_wrap;
    text
}

/// A title or caption centered over `width` cells.
fn annotation(markup: &str, style: Style, width: usize) -> Vec<Segment> {
    let mut text = markup_text(markup);
    text.style = style;
    text.justify = Justify::Center;
    render_text(&text, width)
}

/// Renders `table` for a console `width` cells wide: title, rows with their borders, caption.
pub(crate) fn render_table(table: &Table, width: usize) -> Vec<Segment> {
    let null = Style::new();
    let newline = || run("\n", &Style::new());
    if table.columns.is_empty() {
        return vec![newline()];
    }
    let count = table.columns.len();
    let texts: Vec<Vec<Text>> = table
        .columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let mut cells = Vec::with_capacity(table.rows.len() + 1);
            cells.push(cell_text(&column.header, column));
            for row in &table.rows {
                let markup = row.get(index).map_or("", String::as_str);
                cells.push(cell_text(markup, column));
            }
            cells
        })
        .collect();

    let extra = 2 + count - 1;
    let available = width as i64 - extra as i64;
    let widths = column_widths(&table.columns, &texts, available);
    let table_width: usize = widths.iter().sum::<usize>() + extra;
    let rows = box_rows(table.box_style);

    let mut out = Vec::new();
    if let Some(title) = table.title.as_deref().filter(|t| !t.is_empty()) {
        out.extend(annotation(title, Style::new().italic(), table_width));
    }

    out.push(run(rule(rows[TOP], &widths), &null));
    out.push(newline());
    let body_rows = table.rows.len();
    let header_style = Style::new().bold();
    for row_index in 0..=body_rows {
        let first = row_index == 0;
        let last = row_index == body_rows;
        let mut cells: Vec<Vec<Vec<Segment>>> = Vec::with_capacity(count);
        for (column, (column_texts, &column_width)) in
            table.columns.iter().zip(texts.iter().zip(&widths))
        {
            let base = if first { &header_style } else { &column.style };
            cells.push(cell_lines(&column_texts[row_index], column_width, base));
        }
        let height = cells.iter().map(Vec::len).max().unwrap_or(0);
        for ((column, &column_width), lines) in
            table.columns.iter().zip(&widths).zip(cells.iter_mut())
        {
            let missing = height - lines.len();
            if missing == 0 {
                continue;
            }
            let base = if first { &header_style } else { &column.style };
            let blanks = (0..missing).map(|_| vec![run(" ".repeat(column_width), base)]);
            if first {
                let mut padded: Vec<Vec<Segment>> = blanks.collect();
                padded.append(lines);
                *lines = padded;
            } else {
                lines.extend(blanks);
            }
        }
        let height = if height == 0 {
            for (&column_width, lines) in widths.iter().zip(cells.iter_mut()) {
                lines.push(vec![run(" ".repeat(column_width), &null)]);
            }
            1
        } else {
            height
        };
        let edge = if first {
            HEAD
        } else if last {
            FOOT
        } else {
            MID
        };
        let (left, divider, right) = (rows[edge][0], rows[edge][2], rows[edge][3]);
        for line_no in 0..height {
            out.push(run(left, &null));
            for (index, lines) in cells.iter_mut().enumerate() {
                out.append(&mut lines[line_no]);
                if index + 1 < count {
                    out.push(run(divider, &null));
                }
            }
            out.push(run(right, &null));
            out.push(newline());
        }
        if first {
            out.push(run(rule(rows[HEAD_ROW], &widths), &null));
            out.push(newline());
        } else if table.show_lines && !last {
            out.push(run(rule(rows[ROW], &widths), &null));
            out.push(newline());
        }
    }
    out.push(run(rule(rows[BOTTOM], &widths), &null));
    out.push(newline());

    if let Some(caption) = table.caption.as_deref().filter(|c| !c.is_empty()) {
        out.extend(annotation(
            caption,
            Style::new().italic().dim(),
            table_width,
        ));
    }
    out
}
