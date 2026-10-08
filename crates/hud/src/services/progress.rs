//! A progress display as lines of styled runs: one line per task, columns laid out as the
//! cells of a grid with one cell of padding after every column but the last.

use super::frame::{adjust_line, markup_text, run};
use super::layout::{Measurement, arrange, measure_cell_padded, measure_padded};
use super::table::content_lines;
use crate::model::{
    BarColumn, Justify, Overflow, ProgressColumn, Segment, Style, TaskSnapshot, Text,
};
use crate::model::{MofNCompleteColumn, SpinnerColumn, TaskProgressColumn, TextColumn};
use crate::model::{TimeElapsedColumn, TimeRemainingColumn};

const FULL: char = '━';
const HALF_RIGHT: char = '╸';
const HALF_LEFT: char = '╺';
const DOTS: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const DOTS_INTERVAL: f64 = 0.080;

/// Everything one frame of a progress display is drawn from.
pub(crate) struct Frame<'a> {
    pub(crate) columns: &'a [ProgressColumn],
    pub(crate) tasks: &'a [TaskSnapshot],
    /// The moment of the frame, in seconds, for spinners.
    pub(crate) now: f64,
    /// Per column, the moment its spinner started turning (ignored by other columns).
    pub(crate) spinner_origins: &'a [f64],
    /// The part of a bar that is not complete is drawn; false when the output has no color.
    pub(crate) draw_track: bool,
}

enum Cell<'a> {
    Text(Text),
    Bar {
        column: &'a BarColumn,
        completed: u64,
        total: u64,
    },
}

fn percentage(task: &TaskSnapshot) -> f64 {
    if task.total == 0 {
        return 0.0;
    }
    let done = (task.completed as f64 / task.total as f64) * 100.0;
    done.clamp(0.0, 100.0)
}

fn fill(template: &str, task: &TaskSnapshot) -> String {
    template
        .replace("{task.description}", &task.description)
        .replace("{task.completed}", &task.completed.to_string())
        .replace("{task.total}", &task.total.to_string())
}

/// `timedelta` as Python prints it: `0:01:05`, `1 day, 2:00:00`.
fn clock(seconds: f64) -> String {
    let whole = seconds.max(0.0) as u64;
    let (days, rest) = (whole / 86_400, whole % 86_400);
    let (hours, minutes, secs) = (rest / 3600, rest % 3600 / 60, rest % 60);
    let time = format!("{hours}:{minutes:02}:{secs:02}");
    match days {
        0 => time,
        1 => format!("1 day, {time}"),
        _ => format!("{days} days, {time}"),
    }
}

fn remaining_text(column: &TimeRemainingColumn, task: &TaskSnapshot) -> Text {
    let (seconds, style) = if column.elapsed_when_finished && task.finished() {
        (task.finished_time, &column.elapsed_style)
    } else {
        (task.time_remaining, &column.style)
    };
    let text = match seconds {
        None if column.compact => "--:--".to_string(),
        None => "-:--:--".to_string(),
        Some(seconds) => {
            let whole = seconds.max(0.0) as u64;
            let (hours, minutes, secs) = (whole / 3600, whole % 3600 / 60, whole % 60);
            if column.compact && hours == 0 {
                format!("{minutes:02}:{secs:02}")
            } else {
                format!("{hours}:{minutes:02}:{secs:02}")
            }
        }
    };
    Text::styled(text, style.clone())
}

fn spinner_text(column: &SpinnerColumn, task: &TaskSnapshot, now: f64, origin: f64) -> Text {
    if task.finished() {
        return markup_text(&column.finished_text);
    }
    let frame = ((now - origin) * column.speed) / DOTS_INTERVAL;
    let index = (frame as i64).rem_euclid(DOTS.len() as i64) as usize;
    Text::styled(DOTS[index], column.style.clone())
}

/// Rich builds text and percentage columns with `no_wrap`: they are cut, never wrapped, and
/// they are the last to give cells up when the line is too narrow.
fn no_wrap(column: &ProgressColumn) -> bool {
    matches!(
        column,
        ProgressColumn::Text(_) | ProgressColumn::TaskProgress(_)
    )
}

fn cell_for<'a>(
    column: &'a ProgressColumn,
    task: &TaskSnapshot,
    now: f64,
    origin: f64,
) -> Cell<'a> {
    let text = match column {
        ProgressColumn::Text(TextColumn { template, style }) => {
            let mut text = markup_text(&fill(template, task));
            text.style = style.clone();
            text
        }
        ProgressColumn::Bar(column) => {
            return Cell::Bar {
                column,
                completed: task.completed,
                total: task.total,
            };
        }
        ProgressColumn::TaskProgress(TaskProgressColumn { style }) => {
            let shown = format!("{:>3}%", percentage(task).round_ties_even() as i64);
            let mut text = Text::new(shown);
            let len = text.plain().len();
            text.stylize(0..len, style.clone());
            text
        }
        ProgressColumn::MofNComplete(MofNCompleteColumn { separator, style }) => {
            let width = task.total.to_string().len();
            let shown = format!("{:>width$}{separator}{}", task.completed, task.total);
            Text::styled(shown, style.clone())
        }
        ProgressColumn::TimeElapsed(TimeElapsedColumn { style }) => {
            let elapsed = if task.finished() {
                task.finished_time
            } else {
                task.elapsed
            };
            let shown = elapsed.map_or_else(|| "-:--:--".to_string(), clock);
            Text::styled(shown, style.clone())
        }
        ProgressColumn::TimeRemaining(column) => remaining_text(column, task),
        ProgressColumn::Spinner(column) => spinner_text(column, task, now, origin),
    };
    let mut text = text;
    text.justify = Justify::Left;
    text.overflow = Overflow::Ellipsis;
    text.no_wrap = no_wrap(column);
    Cell::Text(text)
}

/// The runs of a bar `width` cells wide: the complete part, a half cell when the count falls
/// between two cells, and the track when the output has color.
fn bar_segments(
    column: &BarColumn,
    completed: u64,
    total: u64,
    width: usize,
    draw_track: bool,
) -> Vec<Segment> {
    let done = completed.min(total);
    let halves = if total > 0 {
        ((width * 2 * done as usize) as f64 / total as f64) as usize
    } else {
        width * 2
    };
    let (bars, half) = (halves / 2, halves % 2);
    let finished = completed >= total;
    let complete_style = if finished {
        &column.finished
    } else {
        &column.complete
    };
    let mut out = Vec::with_capacity(4);
    if bars > 0 {
        out.push(run(FULL.to_string().repeat(bars), complete_style));
    }
    if half > 0 {
        out.push(run(HALF_RIGHT.to_string(), complete_style));
    }
    if draw_track {
        let mut remaining = width - bars - half;
        if remaining > 0 {
            if half == 0 && bars > 0 {
                out.push(run(HALF_LEFT.to_string(), &column.back));
                remaining -= 1;
            }
            if remaining > 0 {
                out.push(run(FULL.to_string().repeat(remaining), &column.back));
            }
        }
    }
    out
}

/// What a cell asks for within `max_width` cells, `pad` cells of padding included.
fn measure_cell(cell: &Cell<'_>, max_width: i64, pad: i64) -> Measurement {
    match cell {
        Cell::Text(text) => measure_cell_padded(text, max_width, pad),
        Cell::Bar { column, .. } => measure_padded(
            || match column.bar_width {
                Some(width) => Measurement {
                    min: width as i64,
                    max: width as i64,
                },
                None => Measurement {
                    min: 4,
                    max: max_width,
                },
            },
            max_width,
            pad,
        ),
    }
}

/// The lines of one cell in a column `width` cells wide whose last `pad` cells are padding,
/// each exactly `width` cells.
fn cell_lines(cell: &Cell<'_>, width: usize, pad: usize, draw_track: bool) -> Vec<Vec<Segment>> {
    let inner = width.saturating_sub(pad);
    if inner == 0 {
        return Vec::new();
    }
    let null = Style::new();
    let mut lines = match cell {
        Cell::Text(text) => content_lines(text, inner),
        Cell::Bar {
            column,
            completed,
            total,
        } => {
            let bar_width = column.bar_width.unwrap_or(inner).min(inner);
            let mut line = bar_segments(column, *completed, *total, bar_width, draw_track);
            adjust_line(&mut line, inner, &null, true);
            vec![line]
        }
    };
    if pad > 0 {
        for line in &mut lines {
            line.push(run(" ".repeat(pad), &null));
        }
    }
    lines
}

/// The lines of the display, each without its newline: a row per task, every cell laid out in
/// its column, the columns shrunk to fit `width` the way Rich shrinks a table.
pub(crate) fn render_lines(frame: &Frame<'_>, width: usize) -> Vec<Vec<Segment>> {
    let count = frame.columns.len();
    if count == 0 || frame.tasks.is_empty() {
        return Vec::new();
    }
    let rows: Vec<Vec<Cell<'_>>> = frame
        .tasks
        .iter()
        .map(|task| {
            frame
                .columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    let origin = frame.spinner_origins.get(index).copied().unwrap_or(0.0);
                    cell_for(column, task, frame.now, origin)
                })
                .collect()
        })
        .collect();
    let pads: Vec<usize> = (0..count)
        .map(|index| usize::from(index + 1 < count))
        .collect();

    let wrapable: Vec<bool> = frame
        .columns
        .iter()
        .map(|column| !no_wrap(column))
        .collect();
    let widths = arrange(
        &wrapable,
        |index, max_width| {
            let column_cells: Vec<&Cell<'_>> = rows.iter().map(|row| &row[index]).collect();
            measure_column(&column_cells, max_width, pads[index] as i64)
        },
        width as i64,
    );

    let null = Style::new();
    let mut out = Vec::new();
    for row in &rows {
        let mut cells: Vec<Vec<Vec<Segment>>> = row
            .iter()
            .zip(&widths)
            .zip(&pads)
            .map(|((cell, &w), &pad)| cell_lines(cell, w, pad, frame.draw_track))
            .collect();
        let height = cells.iter().map(Vec::len).max().unwrap_or(0);
        for (lines, &w) in cells.iter_mut().zip(&widths) {
            while lines.len() < height {
                lines.push(vec![run(" ".repeat(w), &null)]);
            }
        }
        if height == 0 {
            out.push(widths.iter().map(|&w| run(" ".repeat(w), &null)).collect());
            continue;
        }
        for line_no in 0..height {
            let mut line = Vec::new();
            for lines in &mut cells {
                line.append(&mut lines[line_no]);
            }
            out.push(line);
        }
    }
    out
}

fn measure_column(cells: &[&Cell<'_>], max_width: i64, pad: i64) -> Measurement {
    if max_width < 1 {
        return Measurement { min: 0, max: 0 };
    }
    let (mut min, mut max) = (None, None);
    for cell in cells {
        let measured = measure_cell(cell, max_width, pad);
        min = Some(min.map_or(measured.min, |m: i64| m.max(measured.min)));
        max = Some(max.map_or(measured.max, |m: i64| m.max(measured.max)));
    }
    Measurement {
        min: min.unwrap_or(1),
        max: max.unwrap_or(max_width),
    }
    .with_maximum(max_width)
}
