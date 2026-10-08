//! A columns spec to styled runs: items arranged into as many columns as fit, laid out as the
//! borderless grid Rich builds for it (padding collapsed between columns, none at the edges).

use hud_width::cell_width;

use super::frame::{adjust_line, run, split_lines};
use super::layout::{
    Measurement, arrange_total, measure_padded, measure_renderable, ratio_distribute,
};
use super::table::annotation;
use crate::model::{
    Align, Capabilities, Columns, Justify, Measure, Overflow, Pad, Renderable, Segment, Style, Text,
};

/// Renders `item` into lines `width` cells wide, padded with plain spaces.
fn item_lines(
    item: &dyn Renderable,
    width: usize,
    caps: Option<&Capabilities>,
) -> Vec<Vec<Segment>> {
    if width == 0 {
        return Vec::new();
    }
    let segments = match caps {
        Some(caps) => item.render_with(width, caps),
        None => item.render(width),
    };
    split_lines(segments, width, Some(&Style::new()))
}

/// A renderable held to at most `width` cells: Rich's `Constrain`.
struct Constrained<'a> {
    inner: &'a dyn Renderable,
    width: usize,
}

impl Renderable for Constrained<'_> {
    fn render(&self, width: usize) -> Vec<Segment> {
        self.inner.render(width.min(self.width))
    }

    fn render_with(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        self.inner.render_with(width.min(self.width), caps)
    }

    fn measure(&self, _max_width: usize) -> Measure {
        measure_renderable(self.inner, self.width as i64).measure()
    }
}

/// What one cell holds: the item, held to a width when the columns are equal, and placed in the
/// cell when they are aligned.
struct Cell<'a> {
    item: &'a dyn Renderable,
    constrain: Option<usize>,
    align: Option<Align>,
}

impl Cell<'_> {
    fn with_target<R>(&self, with: impl FnOnce(&dyn Renderable) -> R) -> R {
        match self.constrain {
            Some(width) => with(&Constrained {
                inner: self.item,
                width,
            }),
            None => with(self.item),
        }
    }

    /// What the cell asks for within `available` cells, before padding.
    fn measure(&self, available: i64) -> Measurement {
        self.with_target(|target| measure_renderable(target, available))
    }

    /// The cell's content in lines exactly `inner` cells wide.
    fn lines(&self, inner: usize, caps: Option<&Capabilities>) -> Vec<Vec<Segment>> {
        self.with_target(|target| match self.align {
            None => item_lines(target, inner, caps),
            Some(align) => aligned_lines(target, align, inner, caps),
        })
    }
}

/// Rich's `Align`: the item rendered at the width it asks for, its lines levelled to the widest,
/// then placed in `inner` cells.
fn aligned_lines(
    target: &dyn Renderable,
    align: Align,
    inner: usize,
    caps: Option<&Capabilities>,
) -> Vec<Vec<Segment>> {
    let asked = measure_renderable(target, inner as i64).max.max(0) as usize;
    let child = Constrained {
        inner: target,
        width: asked,
    };
    let segments = match caps {
        Some(caps) => child.render_with(inner, caps),
        None => child.render(inner),
    };
    let mut lines = split_lines(segments, usize::MAX, None);
    let width = lines
        .iter()
        .map(|line| line.iter().map(|s| cell_width(&s.text)).sum::<usize>())
        .max()
        .unwrap_or(0);
    let plain = Style::new();
    for line in &mut lines {
        adjust_line(line, width, &plain, true);
    }
    let excess = inner as i64 - width as i64;
    if excess > 0 {
        let excess = excess as usize;
        for line in &mut lines {
            match align {
                Align::Left => line.push(run(" ".repeat(excess), &plain)),
                Align::Center => {
                    let left = excess / 2;
                    if left > 0 {
                        line.insert(0, run(" ".repeat(left), &plain));
                    }
                    line.push(run(" ".repeat(excess - left), &plain));
                }
                Align::Right => line.insert(0, run(" ".repeat(excess), &plain)),
            }
        }
    }
    for line in &mut lines {
        adjust_line(line, inner, &plain, true);
    }
    lines
}

/// The order the items take in the grid, `None` marking an empty cell at the end of the last row.
fn arrangement(items: usize, column_count: usize, column_first: bool) -> Vec<Option<usize>> {
    let mut order: Vec<Option<usize>> = if column_first {
        let mut lengths = vec![items / column_count; column_count];
        for length in lengths.iter_mut().take(items % column_count) {
            *length += 1;
        }
        let rows = items.div_ceil(column_count);
        let mut cells = vec![vec![None; column_count]; rows];
        let (mut row, mut column) = (0, 0);
        for index in 0..items {
            cells[row][column] = Some(index);
            lengths[column] -= 1;
            if lengths[column] != 0 {
                row += 1;
            } else {
                column += 1;
                row = 0;
            }
        }
        cells
            .into_iter()
            .flatten()
            .take_while(Option::is_some)
            .collect()
    } else {
        (0..items).map(Some).collect()
    };
    if items % column_count != 0 {
        order.extend(std::iter::repeat_n(
            None,
            column_count - items % column_count,
        ));
    }
    order
}

/// How many columns fit, as Rich finds it: the most columns whose widest items, with the padding
/// between them, stay within `max_width`.
fn fit_columns(widths: &[i64], width_padding: i64, max_width: i64, column_first: bool) -> usize {
    let items = widths.len();
    let mut column_count = items;
    while column_count > 1 {
        let mut columns: Vec<i64> = Vec::new();
        let mut column = 0;
        let mut too_wide = false;
        for position in arrangement(items, column_count, column_first) {
            let width = position.map_or(0, |index| widths[index]);
            if columns.len() <= column {
                columns.resize(column + 1, 0);
            }
            columns[column] = columns[column].max(width);
            let total = columns.iter().sum::<i64>() + width_padding * (columns.len() as i64 - 1);
            if total > max_width {
                column_count = columns.len() - 1;
                too_wide = true;
                break;
            }
            column = (column + 1) % column_count;
        }
        if !too_wide {
            break;
        }
    }
    column_count.max(1)
}

/// The padding of a cell as the grid applies it: collapsed against the neighbour, none at the
/// edges of the grid.
fn cell_padding(
    pad: Pad,
    first_column: bool,
    last_column: bool,
    first_row: bool,
    last_row: bool,
) -> Pad {
    let Pad {
        mut top,
        mut right,
        mut bottom,
        mut left,
    } = pad;
    if !first_column {
        left = left.saturating_sub(right);
    }
    if !last_row {
        bottom = top.saturating_sub(bottom);
    }
    if first_column {
        left = 0;
    }
    if last_column {
        right = 0;
    }
    if first_row {
        top = 0;
    }
    if last_row {
        bottom = 0;
    }
    Pad {
        top,
        right,
        bottom,
        left,
    }
}

/// The padding the grid reserves for a column when it measures it (Rich's own rule, which is not
/// the one it renders with when the left and right padding differ).
fn column_padding_width(pad: Pad, column: usize, count: usize) -> i64 {
    let (mut left, mut right) = (0i64, pad.right as i64);
    if column == 0 {
        left = 0;
    }
    if column + 1 == count {
        right = 0;
    }
    left + right
}

/// A cell's lines with its padding around them: blank lines above and below, spaces at the sides.
fn padded(lines: Vec<Vec<Segment>>, width: usize, pad: Pad) -> Vec<Vec<Segment>> {
    let null = Style::new();
    let mut out = Vec::with_capacity(lines.len() + pad.top + pad.bottom);
    let blank = || vec![run(" ".repeat(width), &null)];
    out.extend((0..pad.top).map(|_| blank()));
    for mut line in lines {
        if pad.left > 0 {
            line.insert(0, run(" ".repeat(pad.left), &null));
        }
        if pad.right > 0 {
            line.push(run(" ".repeat(pad.right), &null));
        }
        out.push(line);
    }
    out.extend((0..pad.bottom).map(|_| blank()));
    out
}

pub(crate) fn render_columns(
    columns: &Columns,
    max_width: usize,
    caps: Option<&Capabilities>,
) -> Vec<Segment> {
    // A grid cell lays text out left-justified with an ellipsis at the edge, unless the text asks
    // for something else, as a Rich table cell does with the options of its column.
    let cell_texts: Vec<Option<Text>> = columns
        .items
        .iter()
        .map(|body| {
            body.1.as_ref().map(|text| {
                let mut text = (**text).clone();
                if text.justify == Justify::Default {
                    text.justify = Justify::Left;
                }
                if text.overflow == Overflow::Fold {
                    text.overflow = Overflow::Ellipsis;
                }
                text
            })
        })
        .collect();
    let items: Vec<&dyn Renderable> = columns
        .items
        .iter()
        .zip(&cell_texts)
        .map(|(body, text)| match text {
            Some(text) => text as &dyn Renderable,
            None => &*body.0,
        })
        .collect();
    if items.is_empty() {
        return Vec::new();
    }
    let max_width_i = max_width as i64;
    let pad = columns.padding;
    let width_padding = pad.left.max(pad.right) as i64;
    let mut asked: Vec<i64> = items
        .iter()
        .map(|item| measure_renderable(*item, max_width_i).max)
        .collect();
    let equal = asked.iter().copied().max().unwrap_or(0);
    if columns.equal {
        asked.iter_mut().for_each(|width| *width = equal);
    }
    let column_count = match columns.width {
        Some(width) => (max_width / (width + width_padding as usize).max(1)).max(1),
        None => fit_columns(&asked, width_padding, max_width_i, columns.column_first),
    };

    let empty = Text::new("");
    let constrain = columns.equal.then_some(asked[0].max(0) as usize);
    let order = arrangement(items.len(), column_count, columns.column_first);
    let cells: Vec<Cell> = order
        .iter()
        .map(|position| Cell {
            item: position.map_or(&empty as &dyn Renderable, |index| items[index]),
            constrain: position.and(constrain),
            align: position.and(columns.align),
        })
        .collect();
    let mut rows: Vec<Vec<&Cell>> = cells
        .chunks(column_count)
        .map(|row| row.iter().collect())
        .collect();
    if columns.right_to_left {
        rows.iter_mut().for_each(|row| row.reverse());
    }
    let row_count = rows.len();
    let any_padding = pad != Pad::default();

    let wrapable: Vec<bool> = vec![columns.width.is_none(); column_count];
    let (mut widths, table_width) = arrange_total(
        &wrapable,
        |column, available| {
            if let Some(fixed) = columns.width {
                let width = fixed as i64 + column_padding_width(pad, column, column_count);
                return Measurement {
                    min: width,
                    max: width,
                }
                .with_maximum(available);
            }
            let extra = cell_padding(pad, column == 0, column + 1 == column_count, false, false);
            let extra = (extra.left + extra.right) as i64;
            let mut min: Option<i64> = None;
            let mut max: Option<i64> = None;
            for row in &rows {
                let measured = measure_padded(|| row[column].measure(available), available, extra);
                min = Some(min.map_or(measured.min, |m| m.max(measured.min)));
                max = Some(max.map_or(measured.max, |m| m.max(measured.max)));
            }
            Measurement {
                min: min.unwrap_or(1),
                max: max.unwrap_or(available),
            }
            .with_maximum(available)
        },
        max_width_i,
    );
    if columns.expand && table_width < max_width_i {
        let ratios: Vec<i64> = widths.iter().map(|&width| width as i64).collect();
        let extra = ratio_distribute(max_width_i - table_width, &ratios);
        for (width, add) in widths.iter_mut().zip(extra) {
            *width = (*width as i64 + add).max(0) as usize;
        }
    }
    let total: usize = widths.iter().sum();

    let mut out = Vec::new();
    if let Some(title) = columns.title.as_deref().filter(|t| !t.is_empty()) {
        out.extend(annotation(title, Style::new().italic(), total));
    }
    let null = Style::new();
    for (row_index, row) in rows.iter().enumerate() {
        let mut rendered: Vec<Vec<Vec<Segment>>> = Vec::with_capacity(column_count);
        for (column, cell) in row.iter().enumerate() {
            let width = widths[column];
            let cell_pad = cell_padding(
                pad,
                column == 0,
                column + 1 == column_count,
                row_index == 0,
                row_index + 1 == row_count,
            );
            let inner = width.saturating_sub(cell_pad.left + cell_pad.right);
            let lines = if inner == 0 && any_padding {
                Vec::new()
            } else {
                cell.lines(inner, caps)
            };
            let lines = if any_padding {
                padded(lines, width, cell_pad)
            } else {
                lines
            };
            rendered.push(lines);
        }
        let height = rendered.iter().map(Vec::len).max().unwrap_or(0).max(1);
        for (column, lines) in rendered.iter_mut().enumerate() {
            while lines.len() < height {
                lines.push(vec![run(" ".repeat(widths[column]), &null)]);
            }
        }
        for line_no in 0..height {
            for lines in rendered.iter_mut() {
                out.append(&mut lines[line_no]);
            }
            out.push(run("\n", &null));
        }
    }
    out
}
