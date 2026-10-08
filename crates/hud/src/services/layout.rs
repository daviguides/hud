//! Column arrangement for tables: how wide each column wants to be, and how the columns give
//! way when the table does not fit.

use hud_width::cell_width;

use crate::model::{Column, Text};

/// Cells of padding the table puts on each side of a cell.
pub(crate) const PADDING: i64 = 1;

/// The narrowest and the widest a piece of content can be printed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Measurement {
    pub(crate) min: i64,
    pub(crate) max: i64,
}

impl Measurement {
    fn normalize(self) -> Measurement {
        let min = self.min.max(0).min(self.max);
        Measurement {
            min: min.max(0),
            max: min.max(self.max).max(0),
        }
    }

    fn with_maximum(self, width: i64) -> Measurement {
        Measurement {
            min: self.min.min(width),
            max: self.max.min(width),
        }
    }

    fn with_minimum(self, width: i64) -> Measurement {
        let width = width.max(0);
        Measurement {
            min: self.min.max(width),
            max: self.max.max(width),
        }
    }
}

fn is_line_break(c: char) -> bool {
    matches!(
        c,
        '\n' | '\r'
            | '\u{b}'
            | '\u{c}'
            | '\u{1c}'
            | '\u{1d}'
            | '\u{1e}'
            | '\u{85}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

fn is_space(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

/// The widest line of `text`, split where Python splits lines.
fn widest_line(text: &str) -> usize {
    let mut widest = 0;
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if is_line_break(c) {
            widest = widest.max(cell_width(&text[start..at]));
            start = at + c.len_utf8();
            if c == '\r' && chars.next_if(|&(_, next)| next == '\n').is_some() {
                start += 1;
            }
        }
    }
    if start < text.len() {
        widest = widest.max(cell_width(&text[start..]));
    }
    widest
}

/// The measurement of the text itself: its longest word to its longest line.
fn measure_text(text: &Text) -> Measurement {
    let plain = text.plain();
    let widest = widest_line(plain);
    let longest_word = plain
        .split(is_space)
        .filter(|word| !word.is_empty())
        .map(cell_width)
        .max();
    Measurement {
        min: longest_word.unwrap_or(widest) as i64,
        max: widest as i64,
    }
}

/// The measurement of a cell as the table prints it: the text plus a cell of padding on each
/// side, never wider than `max_width`.
fn measure_cell(text: &Text, max_width: i64) -> Measurement {
    if max_width < 1 {
        return Measurement { min: 0, max: 0 };
    }
    let extra = 2 * PADDING;
    let padded = if max_width - extra < 1 {
        Measurement {
            min: max_width,
            max: max_width,
        }
    } else {
        let inner = measure_text(text).normalize().with_maximum(max_width);
        let inner = if inner.max < 1 {
            Measurement { min: 0, max: 0 }
        } else {
            inner.normalize()
        };
        Measurement {
            min: inner.min + extra,
            max: inner.max + extra,
        }
        .with_maximum(max_width)
    };
    let padded = padded.normalize().with_maximum(max_width);
    if padded.max < 1 {
        Measurement { min: 0, max: 0 }
    } else {
        padded.normalize()
    }
}

/// The range a column wants, from its header and cells, within `max_width`.
pub(crate) fn measure_column(column: &Column, cells: &[Text], max_width: i64) -> Measurement {
    if max_width < 1 {
        return Measurement { min: 0, max: 0 };
    }
    let padding = 2 * PADDING;
    if let Some(width) = column.width {
        let width = width as i64 + padding;
        return Measurement {
            min: width,
            max: width,
        }
        .with_maximum(max_width);
    }
    let mut min = None;
    let mut max = None;
    for text in cells {
        let measured = measure_cell(text, max_width);
        min = Some(min.map_or(measured.min, |m: i64| m.max(measured.min)));
        max = Some(max.map_or(measured.max, |m: i64| m.max(measured.max)));
    }
    let mut measurement = Measurement {
        min: min.unwrap_or(1),
        max: max.unwrap_or(max_width),
    }
    .with_maximum(max_width);
    if let Some(min_width) = column.min_width {
        measurement = measurement.with_minimum(min_width as i64 + padding);
    }
    if let Some(max_width) = column.max_width {
        measurement = measurement.with_maximum(max_width as i64 + padding);
    }
    measurement
}

/// Divides `total` between slots in proportion to `ratios`, taking at most `maximums[i]` from
/// slot `i`, rounding halves to even; returns `values` less what was taken.
fn ratio_reduce(total: i64, ratios: &[i64], maximums: &[i64], values: &[i64]) -> Vec<i64> {
    let ratios: Vec<i64> = ratios
        .iter()
        .zip(maximums)
        .map(|(&ratio, &maximum)| if maximum != 0 { ratio } else { 0 })
        .collect();
    let mut total_ratio: i64 = ratios.iter().sum();
    if total_ratio == 0 {
        return values.to_vec();
    }
    let mut remaining = total;
    let mut out = Vec::with_capacity(values.len());
    for ((&ratio, &maximum), &value) in ratios.iter().zip(maximums).zip(values) {
        if ratio != 0 && total_ratio > 0 {
            let share = (ratio * remaining) as f64 / total_ratio as f64;
            let distributed = maximum.min(share.round_ties_even() as i64);
            out.push(value - distributed);
            remaining -= distributed;
            total_ratio -= ratio;
        } else {
            out.push(value);
        }
    }
    out
}

/// Narrows the widest of the columns that may wrap, level by level, until the widths fit
/// `max_width` or no column can give more.
fn collapse_widths(mut widths: Vec<i64>, wrapable: &[bool], max_width: i64) -> Vec<i64> {
    let mut total: i64 = widths.iter().sum();
    let mut excess = total - max_width;
    if !wrapable.iter().any(|&w| w) {
        return widths;
    }
    let mut rounds = 0;
    while total != 0 && excess > 0 && rounds < 100_000 {
        rounds += 1;
        let widest = widths
            .iter()
            .zip(wrapable)
            .filter(|&(_, &w)| w)
            .map(|(&width, _)| width)
            .max()
            .unwrap_or(0);
        let second = widths
            .iter()
            .zip(wrapable)
            .map(|(&width, &w)| if w && width != widest { width } else { 0 })
            .max()
            .unwrap_or(0);
        let difference = widest - second;
        let ratios: Vec<i64> = widths
            .iter()
            .zip(wrapable)
            .map(|(&width, &w)| i64::from(width == widest && w))
            .collect();
        if !ratios.iter().any(|&r| r != 0) || difference == 0 {
            break;
        }
        let reducible = vec![excess.min(difference); widths.len()];
        widths = ratio_reduce(excess, &ratios, &reducible, &widths);
        total = widths.iter().sum();
        excess = total - max_width;
    }
    widths
}

/// The width of every column, padding included and borders not, for `available` cells.
///
/// `cells[i]` holds the header and the cells of column `i`.
pub(crate) fn column_widths(columns: &[Column], cells: &[Vec<Text>], available: i64) -> Vec<usize> {
    let ranges: Vec<Measurement> = columns
        .iter()
        .zip(cells)
        .map(|(column, texts)| measure_column(column, texts, available))
        .collect();
    let mut widths: Vec<i64> = ranges
        .iter()
        .map(|range| if range.max != 0 { range.max } else { 1 })
        .collect();
    let mut total: i64 = widths.iter().sum();
    if total > available {
        let wrapable: Vec<bool> = columns
            .iter()
            .map(|column| column.width.is_none() && !column.no_wrap)
            .collect();
        widths = collapse_widths(widths, &wrapable, available);
        total = widths.iter().sum();
        if total > available {
            let excess = total - available;
            let ones = vec![1; widths.len()];
            widths = ratio_reduce(excess, &ones, &widths, &widths);
        }
        widths = columns
            .iter()
            .zip(cells)
            .zip(&widths)
            .map(|((column, texts), &width)| measure_column(column, texts, width).max)
            .collect();
    }
    widths.iter().map(|&width| width.max(0) as usize).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_reduce_rounds_halves_to_even() {
        assert_eq!(ratio_reduce(5, &[1, 1], &[9, 9], &[10, 10]), [8, 7]);
        assert_eq!(
            ratio_reduce(1, &[1, 1, 1], &[9, 9, 9], &[5, 5, 5]),
            [5, 5, 4]
        );
    }

    #[test]
    fn collapse_levels_the_widest_columns_first() {
        assert_eq!(
            collapse_widths(vec![30, 10, 10], &[true; 3], 40),
            [20, 10, 10]
        );
        assert_eq!(collapse_widths(vec![30, 10], &[false, false], 20), [30, 10]);
    }

    #[test]
    fn a_cell_measures_its_text_plus_padding() {
        let text = Text::new("alpha beta\nxy");
        let measured = measure_cell(&text, 40);
        assert_eq!((measured.min, measured.max), (7, 12));
        let measured = measure_cell(&text, 8);
        assert_eq!((measured.min, measured.max), (7, 8));
    }
}
