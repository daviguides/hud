//! A progress display as lines of styled runs: one line per task, columns laid out as the
//! cells of a grid with one cell of padding after every column but the last.

use super::frame::{adjust_line, markup_text, run};
use super::layout::{Measurement, arrange, measure_cell_padded, measure_padded};
use super::style::emit;
use super::table::content_lines;
use crate::model::{
    BarColumn, ColorSystem, Justify, Overflow, ProgressColumn, Segment, Style, TaskSnapshot, Text,
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

fn percentage(completed: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let done = (completed as f64 / total as f64) * 100.0;
    done.clamp(0.0, 100.0)
}

/// The whole percent a task shows, rounded half to even as Python's `{:.0f}` does.
fn shown_percent(completed: u64, total: u64) -> i64 {
    percentage(completed, total).round_ties_even() as i64
}

fn percent_text(column: &TaskProgressColumn, percent: i64) -> Text {
    let mut text = Text::new(format!("{percent:>3}%"));
    let len = text.plain().len();
    text.stylize(0..len, column.style.clone());
    text
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

/// A cell of `text` placed the way every progress column places it: left, cut with an ellipsis.
fn finish_text<'a>(mut text: Text, column: &ProgressColumn) -> Cell<'a> {
    text.justify = Justify::Left;
    text.overflow = Overflow::Ellipsis;
    text.no_wrap = no_wrap(column);
    Cell::Text(text)
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
        ProgressColumn::TaskProgress(column) => {
            percent_text(column, shown_percent(task.completed, task.total))
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
    finish_text(text, column)
}

/// How many half cells of a bar `width` cells wide are complete.
fn bar_halves(completed: u64, total: u64, width: usize) -> usize {
    if total == 0 {
        return width * 2;
    }
    let product = width as u128 * 2 * u128::from(completed.min(total));
    (product as f64 / total as f64) as usize
}

/// The runs of a bar `width` cells wide: the complete part, a half cell when the count falls
/// between two cells, and the track when the output has color.
fn bar_runs(
    column: &BarColumn,
    halves: usize,
    finished: bool,
    width: usize,
    draw_track: bool,
) -> Vec<Segment> {
    let (bars, half) = (halves / 2, halves % 2);
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

/// The effective width of a bar in a cell with `inner` cells of room.
fn effective_bar_width(column: &BarColumn, inner: usize) -> usize {
    column.bar_width.unwrap_or(inner).min(inner)
}

/// The one line of a bar cell: the bar, padded to `inner` cells, then `pad` cells of padding.
fn bar_cell_line(
    column: &BarColumn,
    bar_width: usize,
    halves: usize,
    finished: bool,
    inner: usize,
    pad: usize,
    draw_track: bool,
) -> Vec<Segment> {
    let null = Style::new();
    let mut line = bar_runs(column, halves, finished, bar_width, draw_track);
    adjust_line(&mut line, inner, &null, true);
    if pad > 0 {
        line.push(run(" ".repeat(pad), &null));
    }
    line
}

/// The lines of one cell in a column `width` cells wide whose last `pad` cells are padding,
/// each exactly `width` cells.
fn cell_lines(cell: &Cell<'_>, width: usize, pad: usize, draw_track: bool) -> Vec<Vec<Segment>> {
    let inner = width.saturating_sub(pad);
    if inner == 0 {
        return Vec::new();
    }
    let null = Style::new();
    match cell {
        Cell::Text(text) => {
            let mut lines = content_lines(text, inner);
            if pad > 0 {
                for line in &mut lines {
                    line.push(run(" ".repeat(pad), &null));
                }
            }
            lines
        }
        Cell::Bar {
            column,
            completed,
            total,
        } => {
            let bar_width = effective_bar_width(column, inner);
            let halves = bar_halves(*completed, *total, bar_width);
            vec![bar_cell_line(
                column,
                bar_width,
                halves,
                completed >= total,
                inner,
                pad,
                draw_track,
            )]
        }
    }
}

fn build_rows<'a>(frame: &Frame<'a>) -> Vec<Vec<Cell<'a>>> {
    frame
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
        .collect()
}

/// Cells of padding after each column: one, except after the last.
fn column_pads(count: usize) -> Vec<usize> {
    (0..count)
        .map(|index| usize::from(index + 1 < count))
        .collect()
}

fn arrange_columns(frame: &Frame<'_>, rows: &[Vec<Cell<'_>>], width: usize) -> Vec<usize> {
    let pads = column_pads(frame.columns.len());
    let wrapable: Vec<bool> = frame
        .columns
        .iter()
        .map(|column| !no_wrap(column))
        .collect();
    arrange(
        &wrapable,
        |index, max_width| {
            let column_cells: Vec<&Cell<'_>> = rows.iter().map(|row| &row[index]).collect();
            measure_column(&column_cells, max_width, pads[index] as i64)
        },
        width as i64,
    )
}

/// The lines of the display, each without its newline: a row per task, every cell laid out in
/// its column, the columns shrunk to fit `width` the way Rich shrinks a table.
pub(crate) fn render_lines(frame: &Frame<'_>, width: usize) -> Vec<Vec<Segment>> {
    if frame.columns.is_empty() || frame.tasks.is_empty() {
        return Vec::new();
    }
    let rows = build_rows(frame);
    let pads = column_pads(frame.columns.len());
    let widths = arrange_columns(frame, &rows, width);

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

/// The bytes of one single-line cell, or `None` when it would take more than one line.
fn cell_ansi(
    cell: &Cell<'_>,
    width: usize,
    pad: usize,
    draw_track: bool,
    color_system: ColorSystem,
    attributes: bool,
) -> Option<String> {
    let lines = cell_lines(cell, width, pad, draw_track);
    let [line] = lines.as_slice() else {
        return None;
    };
    let mut out = String::new();
    for segment in line {
        emit(
            &mut out,
            &segment.text,
            &segment.style,
            color_system,
            attributes,
        );
    }
    Some(out)
}

/// The text that opens and closes a styled run for these capabilities.
fn style_wrap(style: &Style, color_system: ColorSystem, attributes: bool) -> (String, String) {
    const MARK: char = '\u{1}';
    let mut marked = String::new();
    emit(
        &mut marked,
        &MARK.to_string(),
        style,
        color_system,
        attributes,
    );
    match marked.split_once(MARK) {
        Some((open, close)) => (open.to_string(), close.to_string()),
        None => (String::new(), String::new()),
    }
}

fn digits(number: u64) -> usize {
    number.checked_ilog10().map_or(1, |log| log as usize + 1)
}

fn push_number(out: &mut String, mut number: u64) {
    let mut buffer = [0u8; 20];
    let mut at = buffer.len();
    loop {
        at -= 1;
        buffer[at] = b'0' + (number % 10) as u8;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for &byte in &buffer[at..] {
        out.push(char::from(byte));
    }
}

/// A bar column whose cells are built once per distinct state.
struct BarPlan {
    column: BarColumn,
    inner: usize,
    pad: usize,
    bar_width: usize,
    draw_track: bool,
    /// Indexed by `halves * 2 + finished`.
    cache: Vec<Option<String>>,
}

/// A percentage column whose cells are built once per whole percent.
struct PercentPlan {
    column: TaskProgressColumn,
    width: usize,
    pad: usize,
    draw_track: bool,
    cache: Vec<Option<String>>,
}

/// A `completed/total` column written straight into the frame.
struct CountPlan {
    open: String,
    close: String,
    separator: String,
    pad: String,
    rows: Vec<CountRow>,
}

struct CountRow {
    digits: usize,
    total: String,
    spaces: usize,
}

enum PlanColumn {
    /// The finished bytes of the cell, per row.
    Fixed(Vec<String>),
    Bar(BarPlan),
    Percent(PercentPlan),
    Count(CountPlan),
}

/// A frame assembled from cached pieces, for displays of text, bar, percentage and count
/// columns with one line per task. It is valid until the tasks, their descriptions or totals,
/// the console width or the output capabilities change, and gives the bytes of the general path.
pub(crate) struct FastPlan {
    rows: usize,
    columns: Vec<PlanColumn>,
    color_system: ColorSystem,
    attributes: bool,
}

impl FastPlan {
    /// Plans the frame of `frame` at `width` cells, or `None` when the display is outside what
    /// the plan can draw: other columns, wrapped cells, or columns wider than the console.
    pub(crate) fn build(
        frame: &Frame<'_>,
        width: usize,
        color_system: ColorSystem,
        attributes: bool,
    ) -> Option<FastPlan> {
        let count = frame.columns.len();
        if count == 0 || frame.tasks.is_empty() {
            return None;
        }
        let rows = build_rows(frame);
        let widths = arrange_columns(frame, &rows, width);
        if widths.iter().sum::<usize>() > width {
            return None;
        }
        let pads = column_pads(count);
        let mut columns = Vec::with_capacity(count);
        for (index, column) in frame.columns.iter().enumerate() {
            let (cell_width, pad) = (widths[index], pads[index]);
            let inner = cell_width.checked_sub(pad).filter(|&inner| inner > 0)?;
            columns.push(match column {
                ProgressColumn::Text(text) => {
                    if text.template.contains("{task.completed}") {
                        return None;
                    }
                    let fixed = rows
                        .iter()
                        .map(|row| {
                            cell_ansi(
                                &row[index],
                                cell_width,
                                pad,
                                frame.draw_track,
                                color_system,
                                attributes,
                            )
                        })
                        .collect::<Option<Vec<String>>>()?;
                    PlanColumn::Fixed(fixed)
                }
                ProgressColumn::Bar(bar) => PlanColumn::Bar(BarPlan {
                    column: bar.clone(),
                    inner,
                    pad,
                    bar_width: effective_bar_width(bar, inner),
                    draw_track: frame.draw_track,
                    cache: Vec::new(),
                }),
                ProgressColumn::TaskProgress(percent) => {
                    if inner < 4 {
                        return None;
                    }
                    PlanColumn::Percent(PercentPlan {
                        column: percent.clone(),
                        width: cell_width,
                        pad,
                        draw_track: frame.draw_track,
                        cache: vec![None; 101],
                    })
                }
                ProgressColumn::MofNComplete(count_column) => {
                    if !count_column
                        .separator
                        .bytes()
                        .all(|byte| (0x20..0x7f).contains(&byte))
                    {
                        return None;
                    }
                    let (open, close) = style_wrap(&count_column.style, color_system, attributes);
                    let mut plan_rows = Vec::with_capacity(frame.tasks.len());
                    for task in frame.tasks {
                        let total_digits = digits(task.total);
                        let shown = total_digits + count_column.separator.len() + total_digits;
                        if shown > inner || digits(task.completed) > total_digits {
                            return None;
                        }
                        plan_rows.push(CountRow {
                            digits: total_digits,
                            total: task.total.to_string(),
                            spaces: inner - shown,
                        });
                    }
                    PlanColumn::Count(CountPlan {
                        open,
                        close,
                        separator: count_column.separator.clone(),
                        pad: " ".repeat(pad),
                        rows: plan_rows,
                    })
                }
                _ => return None,
            });
        }
        Some(FastPlan {
            rows: frame.tasks.len(),
            columns,
            color_system,
            attributes,
        })
    }

    /// Appends the frame for tasks with these `(completed, total)` counts: the lines joined by
    /// newlines and ended by one. `false` means the counts are outside what the plan assumed
    /// and nothing is appended: draw the frame the general way.
    pub(crate) fn render(&mut self, tasks: &[(u64, u64)], out: &mut String) -> bool {
        if tasks.len() != self.rows {
            return false;
        }
        for (row, &(completed, total)) in tasks.iter().enumerate() {
            for column in &self.columns {
                if let PlanColumn::Count(count) = column {
                    let info = &count.rows[row];
                    if digits(completed) > info.digits || digits(total) != info.digits {
                        return false;
                    }
                }
            }
        }
        for (row, &(completed, total)) in tasks.iter().enumerate() {
            for column in &mut self.columns {
                match column {
                    PlanColumn::Fixed(cells) => out.push_str(&cells[row]),
                    PlanColumn::Bar(bar) => {
                        let halves = bar_halves(completed, total, bar.bar_width);
                        let finished = completed >= total;
                        let slot = halves * 2 + usize::from(finished);
                        if bar.cache.len() <= slot {
                            bar.cache.resize(slot + 1, None);
                        }
                        let (color_system, attributes) = (self.color_system, self.attributes);
                        let cell = bar.cache[slot].get_or_insert_with(|| {
                            let line = bar_cell_line(
                                &bar.column,
                                bar.bar_width,
                                halves,
                                finished,
                                bar.inner,
                                bar.pad,
                                bar.draw_track,
                            );
                            let mut text = String::new();
                            for segment in &line {
                                emit(
                                    &mut text,
                                    &segment.text,
                                    &segment.style,
                                    color_system,
                                    attributes,
                                );
                            }
                            text
                        });
                        out.push_str(cell);
                    }
                    PlanColumn::Percent(percent) => {
                        let shown = shown_percent(completed, total).clamp(0, 100) as usize;
                        let (color_system, attributes) = (self.color_system, self.attributes);
                        let cell = percent.cache[shown].get_or_insert_with(|| {
                            let text = percent_text(&percent.column, shown as i64);
                            let cell = finish_text(
                                text,
                                &ProgressColumn::TaskProgress(percent.column.clone()),
                            );
                            cell_ansi(
                                &cell,
                                percent.width,
                                percent.pad,
                                percent.draw_track,
                                color_system,
                                attributes,
                            )
                            .unwrap_or_default()
                        });
                        out.push_str(cell);
                    }
                    PlanColumn::Count(count) => {
                        let info = &count.rows[row];
                        out.push_str(&count.open);
                        for _ in digits(completed)..info.digits {
                            out.push(' ');
                        }
                        push_number(out, completed);
                        out.push_str(&count.separator);
                        out.push_str(&info.total);
                        for _ in 0..info.spaces {
                            out.push(' ');
                        }
                        out.push_str(&count.close);
                        out.push_str(&count.pad);
                    }
                }
            }
            out.push('\n');
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Capabilities, MofNCompleteColumn, TaskProgressColumn, TextColumn};
    use crate::services::render::to_ansi;

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    fn snapshot(description: &str, completed: u64, total: u64) -> TaskSnapshot {
        TaskSnapshot {
            description: description.to_string(),
            total,
            completed,
            elapsed: Some(0.0),
            finished_time: None,
            time_remaining: None,
        }
    }

    fn general(frame: &Frame<'_>, width: usize, caps: &Capabilities) -> String {
        let mut flat = Vec::new();
        for line in render_lines(frame, width) {
            flat.extend(line);
            flat.push(run("\n", &Style::new()));
        }
        to_ansi(&flat, caps)
    }

    #[test]
    fn the_cached_plan_draws_the_bytes_of_the_general_path() {
        let mut rng = Lcg(20_261_008);
        let systems = [
            ColorSystem::None,
            ColorSystem::Standard,
            ColorSystem::EightBit,
            ColorSystem::TrueColor,
        ];
        let mut planned = 0;
        for case in 0..2000 {
            let width = 30 + rng.below(110) as usize;
            let color_system = systems[rng.below(4) as usize];
            let attributes = color_system != ColorSystem::None || rng.below(2) == 0;
            let caps = Capabilities {
                color_system,
                attributes,
                is_tty: true,
                interactive: true,
                width: width as u16,
                height: 40,
            };
            let mut columns = vec![
                ProgressColumn::Text(TextColumn::new("{task.description}")),
                ProgressColumn::Bar(BarColumn::new().bar_width(5 + rng.below(40) as usize)),
                ProgressColumn::TaskProgress(TaskProgressColumn::new()),
                ProgressColumn::MofNComplete(MofNCompleteColumn::new()),
            ];
            match rng.below(5) {
                0 => columns.swap(0, 3),
                1 => columns.truncate(3),
                2 => {
                    columns.remove(1);
                }
                3 => columns[1] = ProgressColumn::Bar(BarColumn::new().full_width()),
                _ => {}
            }
            let names = [
                "fetch",
                "download crates",
                "verify",
                "x",
                "a long description here",
            ];
            let rows = 1 + rng.below(6) as usize;
            let mut tasks: Vec<TaskSnapshot> = (0..rows)
                .map(|row| {
                    let total = [0, 1, 3, 12, 99, 120, 1000, 12_500][rng.below(8) as usize];
                    let completed = if total == 0 { 0 } else { rng.below(total + 1) };
                    snapshot(names[(row + case) % names.len()], completed, total)
                })
                .collect();
            let origins = vec![0.0; columns.len()];
            let draw_track = color_system != ColorSystem::None;
            let frame = |tasks: &[TaskSnapshot]| -> String {
                general(
                    &Frame {
                        columns: &columns,
                        tasks,
                        now: 0.0,
                        spinner_origins: &origins,
                        draw_track,
                    },
                    width,
                    &caps,
                )
            };
            let Some(mut plan) = FastPlan::build(
                &Frame {
                    columns: &columns,
                    tasks: &tasks,
                    now: 0.0,
                    spinner_origins: &origins,
                    draw_track,
                },
                width,
                color_system,
                attributes,
            ) else {
                continue;
            };
            planned += 1;
            for _ in 0..6 {
                for task in &mut tasks {
                    task.completed = if task.total == 0 {
                        0
                    } else {
                        rng.below(task.total + 1)
                    };
                }
                let pairs: Vec<(u64, u64)> = tasks.iter().map(|t| (t.completed, t.total)).collect();
                let mut fast = String::new();
                assert!(plan.render(&pairs, &mut fast), "case {case}");
                assert_eq!(fast, frame(&tasks), "case {case} width {width}");
            }
        }
        assert!(planned > 1000, "only {planned} of 2000 cases had a plan");
    }

    #[test]
    fn a_count_wider_than_its_total_is_left_to_the_general_path() {
        let columns = [ProgressColumn::MofNComplete(MofNCompleteColumn::new())];
        let tasks = [snapshot("t", 5, 9)];
        let origins = [0.0];
        let frame = Frame {
            columns: &columns,
            tasks: &tasks,
            now: 0.0,
            spinner_origins: &origins,
            draw_track: false,
        };
        let mut plan = FastPlan::build(&frame, 40, ColorSystem::None, false).expect("plan");
        let mut out = String::new();
        assert!(plan.render(&[(5, 9)], &mut out));
        assert_eq!(out, "5/9\n");
        let mut out = String::new();
        assert!(!plan.render(&[(10, 9)], &mut out));
        assert!(out.is_empty());
    }
}
