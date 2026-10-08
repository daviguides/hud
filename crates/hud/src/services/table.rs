//! A table spec to styled runs: cell text laid out in its column, the borders drawn around.

use super::frame::{
    BOTTOM, FOOT, HEAD, HEAD_ROW, MID, ROW, TOP, adjust_line, box_rows, rule, run, split_lines,
};
use super::layout::{PADDING, column_widths};
use super::render::render_text;
use crate::model::{Column, Justify, Overflow, Segment, Style, Table, Text};

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
        && text.overflow != Overflow::Ignore
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
    split_lines(render_text(text, inner), inner, &Style::new())
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
    let header_styles: Vec<Style> = table
        .columns
        .iter()
        .map(|column| table.header_style.combine(&column.header_style))
        .collect();
    for row_index in 0..=body_rows {
        let first = row_index == 0;
        let last = row_index == body_rows;
        let mut cells: Vec<Vec<Vec<Segment>>> = Vec::with_capacity(count);
        for (column_index, (column, (column_texts, &column_width))) in table
            .columns
            .iter()
            .zip(texts.iter().zip(&widths))
            .enumerate()
        {
            let base = if first {
                &header_styles[column_index]
            } else {
                &column.style
            };
            cells.push(cell_lines(&column_texts[row_index], column_width, base));
        }
        let height = cells.iter().map(Vec::len).max().unwrap_or(0);
        for (column_index, ((column, &column_width), lines)) in table
            .columns
            .iter()
            .zip(&widths)
            .zip(cells.iter_mut())
            .enumerate()
        {
            let missing = height - lines.len();
            if missing == 0 {
                continue;
            }
            let base = if first {
                &header_styles[column_index]
            } else {
                &column.style
            };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::render::to_plain;

    fn plain(table: &Table, width: usize) -> String {
        to_plain(&render_table(table, width))
    }

    #[test]
    fn a_small_table_is_boxed_and_aligned() {
        let table = Table::new()
            .column("A")
            .column(Column::new("B").justify(Justify::Right))
            .row(["x", "yy"]);
        assert_eq!(
            plain(&table, 40),
            "┏━━━┳━━━━┓\n┃ A ┃  B ┃\n┡━━━╇━━━━┩\n│ x │ yy │\n└───┴────┘\n"
        );
    }

    #[test]
    fn a_table_without_columns_is_a_blank_line() {
        assert_eq!(plain(&Table::new().title("t"), 40), "\n");
    }

    #[test]
    fn a_longer_row_adds_columns_and_a_shorter_one_is_completed() {
        let table = Table::new().column("A").row(["x", "y"]).row(["z"]);
        let text = plain(&table, 40);
        assert_eq!(text.lines().count(), 6);
        assert!(text.contains("│ x │ y │"));
        assert!(text.contains("│ z │   │"));
    }

    #[test]
    fn overflow_ignore_leaves_the_cell_unjustified() {
        let table = Table::new()
            .column(
                Column::new("A")
                    .justify(Justify::Right)
                    .overflow(Overflow::Ignore),
            )
            .row(["x"])
            .row(["abc"]);
        assert!(plain(&table, 40).contains("│ x   │"));
    }

    #[test]
    fn a_column_squeezed_below_its_padding_has_no_lines_of_its_own() {
        let table = Table::new()
            .column("alpha beta")
            .column("gamma delta")
            .column("omega")
            .row(["one two", "three four", "five"]);
        let text = plain(&table, 9);
        assert!(text.lines().all(|line| line.chars().count() >= 9));
    }
}
