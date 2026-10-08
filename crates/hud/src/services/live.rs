//! The redraw protocol of a live display, as Rich writes it, shared by `Live` and `Progress`.
//!
//! A frame is its lines separated by newlines with none after the last, so the cursor rests at
//! the end of the frame. To draw the next one the cursor goes to the start of the line, erases it,
//! and goes up erasing each earlier line of the frame. The cursor is hidden from the first frame
//! to the last, which is followed by a newline unless the display is transient, in which case the
//! frame is erased the same way.

use super::frame::{run, split_lines};
use super::render::{render_text_ending, to_ansi};
use crate::model::{
    Capabilities, Color, Justify, Overflow, Segment, Style, Text, VerticalOverflow,
};

pub(crate) const HIDE_CURSOR: &str = "\x1b[?25l";
pub(crate) const SHOW_CURSOR: &str = "\x1b[?25h";

/// The lines of rendered runs, each cropped to `width` cells and not padded.
pub(crate) fn frame_lines(segments: Vec<Segment>, width: usize) -> Vec<Vec<Segment>> {
    split_lines(segments, width, None)
}

/// The line that ends a frame that does not fit: `...`, centered, in bold red (Rich's
/// `live.ellipsis`).
fn ellipsis_line(width: usize) -> Vec<Segment> {
    let text = Text::styled("...", Style::new().bold().color(Color::RED))
        .justify(Justify::Center)
        .overflow(Overflow::Crop)
        .end("");
    let mut line = render_text_ending(&text, width, "");
    line.retain(|segment| !segment.text.is_empty());
    line
}

/// Applies the rule for a frame taller than the terminal: with `ellipsis` the frame keeps one line
/// less than the terminal has and ends with `...`, with `crop` it keeps what fits.
pub(crate) fn fit_height(
    lines: &mut Vec<Vec<Segment>>,
    rows: usize,
    width: usize,
    overflow: VerticalOverflow,
) {
    if lines.len() <= rows {
        return;
    }
    match overflow {
        VerticalOverflow::Visible => {}
        VerticalOverflow::Crop => lines.truncate(rows),
        VerticalOverflow::Ellipsis => {
            lines.truncate(rows.saturating_sub(1));
            lines.push(ellipsis_line(width));
        }
    }
}

/// Appends a frame as bytes for a stream with these capabilities: its lines separated by
/// newlines, none after the last.
pub(crate) fn push_frame(out: &mut String, lines: Vec<Vec<Segment>>, caps: &Capabilities) {
    let newline = run("\n", &Style::new());
    let count = lines.len();
    let mut flat = Vec::new();
    for (index, line) in lines.into_iter().enumerate() {
        flat.extend(line);
        if index + 1 < count {
            flat.push(newline.clone());
        }
    }
    out.push_str(&to_ansi(&flat, caps));
}

/// Moves the cursor to the start of a frame of `height` lines and clears each of its lines.
fn position_cursor(out: &mut String, height: usize) {
    if height == 0 {
        return;
    }
    out.push_str("\r\x1b[2K");
    for _ in 1..height {
        out.push_str("\x1b[1A\x1b[2K");
    }
}

/// Clears a frame of `height` lines and leaves the cursor where it started.
fn restore_cursor(out: &mut String, height: usize) {
    if height == 0 {
        return;
    }
    out.push('\r');
    for _ in 0..height {
        out.push_str("\x1b[1A\x1b[2K");
    }
}

/// What a live display has put on the terminal: whether the cursor is hidden and how tall the
/// last frame was.
#[derive(Debug, Default)]
pub(crate) struct Screen {
    open: bool,
    height: usize,
}

impl Screen {
    /// Hides the cursor, once; the display is on the terminal from now on.
    pub(crate) fn open(&mut self, out: &mut String) {
        if !self.open {
            self.open = true;
            out.push_str(HIDE_CURSOR);
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Records a frame of `height` lines drawn now; nothing for the next one to erase if it is empty.
    pub(crate) fn drawn(&mut self, height: usize) {
        self.height = height;
    }

    /// Writes what has to come before a frame so that it lands on the previous one.
    pub(crate) fn rewind(&self, out: &mut String) {
        position_cursor(out, self.height);
    }

    /// Writes what ends a display after its last frame: a newline, the cursor back and, for a
    /// transient display, the erase of the frame.
    pub(crate) fn close(&mut self, out: &mut String, transient: bool) {
        if self.height > 0 {
            out.push('\n');
        }
        out.push_str(SHOW_CURSOR);
        if transient {
            restore_cursor(out, self.height);
        }
        self.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ColorSystem;

    fn caps() -> Capabilities {
        Capabilities {
            color_system: ColorSystem::None,
            attributes: false,
            is_tty: true,
            interactive: true,
            width: 20,
            height: 6,
        }
    }

    fn text(lines: &[&str]) -> Vec<Vec<Segment>> {
        lines
            .iter()
            .map(|line| vec![run(*line, &Style::new())])
            .collect()
    }

    #[test]
    fn a_frame_has_no_newline_after_its_last_line() {
        let mut out = String::new();
        push_frame(&mut out, text(&["a", "b", "c"]), &caps());
        assert_eq!(out, "a\nb\nc");
    }

    #[test]
    fn the_next_frame_erases_the_previous_one_line_by_line() {
        let mut screen = Screen::default();
        let mut out = String::new();
        screen.open(&mut out);
        screen.rewind(&mut out);
        push_frame(&mut out, text(&["a", "b", "c"]), &caps());
        screen.drawn(3);
        screen.rewind(&mut out);
        push_frame(&mut out, text(&["d"]), &caps());
        screen.drawn(1);
        assert_eq!(
            out,
            "\x1b[?25la\nb\nc\r\x1b[2K\x1b[1A\x1b[2K\x1b[1A\x1b[2Kd"
        );
    }

    #[test]
    fn closing_adds_a_newline_and_the_cursor_and_a_transient_display_erases_itself() {
        let mut screen = Screen::default();
        screen.drawn(2);
        let mut out = String::new();
        screen.close(&mut out, false);
        assert_eq!(out, "\n\x1b[?25h");
        screen.drawn(2);
        let mut out = String::new();
        screen.close(&mut out, true);
        assert_eq!(out, "\n\x1b[?25h\r\x1b[1A\x1b[2K\x1b[1A\x1b[2K");
    }

    #[test]
    fn ellipsis_keeps_one_line_less_and_crop_keeps_what_fits() {
        let mut lines = text(&["1", "2", "3", "4", "5"]);
        fit_height(&mut lines, 3, 9, VerticalOverflow::Ellipsis);
        let mut out = String::new();
        push_frame(&mut out, lines, &caps());
        assert_eq!(out, "1\n2\n   ...   ");
        let mut lines = text(&["1", "2", "3", "4", "5"]);
        fit_height(&mut lines, 3, 9, VerticalOverflow::Crop);
        assert_eq!(lines.len(), 3);
        let mut lines = text(&["1", "2", "3", "4", "5"]);
        fit_height(&mut lines, 3, 9, VerticalOverflow::Visible);
        assert_eq!(lines.len(), 5);
    }
}
