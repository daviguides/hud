//! Text to styled runs, and styled runs to bytes.

use hud_width::cell_width;

use super::measure::{set_cell_size, wrap};
use super::style::emit;
use crate::model::{Capabilities, Justify, Segment, Style, Text};

/// Splits one wrapped line into runs wherever a span starts or ends, each run carrying the
/// combination of every style in force there, the line's own style first.
fn line_segments(line: &Text, out: &mut Vec<Segment>) {
    let len = line.plain.len();
    if len == 0 {
        return;
    }
    let base_applies = !line.style.is_null();
    if line.spans.is_empty() && !base_applies {
        out.push(Segment {
            text: line.plain.clone(),
            style: Style::new(),
        });
        return;
    }
    let mut styles: Vec<&Style> = Vec::with_capacity(line.spans.len() + 1);
    let mut bounds: Vec<(usize, usize)> = Vec::with_capacity(line.spans.len() + 1);
    if base_applies {
        styles.push(&line.style);
        bounds.push((0, len));
    }
    for span in &line.spans {
        styles.push(&span.style);
        bounds.push((span.start.min(len), span.end.min(len)));
    }
    // Style 0 is the null style of the whole line; the rest are numbered from 1.
    let mut events: Vec<(usize, bool, usize)> = Vec::with_capacity(bounds.len() * 2 + 2);
    events.push((0, false, 0));
    events.extend(
        bounds
            .iter()
            .enumerate()
            .map(|(i, &(s, _))| (s, false, i + 1)),
    );
    events.extend(
        bounds
            .iter()
            .enumerate()
            .map(|(i, &(_, e))| (e, true, i + 1)),
    );
    events.push((len, true, 0));
    events.sort_by_key(|&(offset, leaving, _)| (offset, leaving));
    let mut active: Vec<usize> = Vec::new();
    for pair in events.windows(2) {
        let (offset, leaving, id) = pair[0];
        let next = pair[1].0;
        if leaving {
            if let Some(index) = active.iter().position(|&a| a == id) {
                active.remove(index);
            }
        } else {
            active.push(id);
        }
        if next > offset {
            let Some(text) = line.plain.get(offset..next) else {
                continue;
            };
            active.sort_unstable();
            let style = active
                .iter()
                .filter(|&&a| a > 0)
                .fold(Style::new(), |acc, &a| acc.combine(styles[a - 1]));
            out.push(Segment {
                text: text.to_string(),
                style,
            });
        }
    }
}

/// A text that wrapping, justifying and cropping would leave exactly as it is: one line with
/// no tab, no newline and a width within the limit.
fn fits_one_line(text: &Text, width: usize) -> bool {
    text.justify == Justify::Default
        && !text.plain.bytes().any(|b| b == b'\n' || b == b'\t')
        && cell_width(&text.plain) <= width
}

/// Renders `text` for a `width`-cell terminal: wrapped lines as styled runs, a newline between
/// lines and the text's end after the last.
pub(crate) fn render_text(text: &Text, width: usize) -> Vec<Segment> {
    render_text_ending(text, width, &text.end)
}

/// Like [`render_text`] with `end` after the last line instead of the text's own.
pub(crate) fn render_text_ending(text: &Text, width: usize, end: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    if fits_one_line(text, width) {
        line_segments(text, &mut out);
        if !end.is_empty() {
            out.push(Segment {
                text: end.to_string(),
                style: Style::new(),
            });
        }
        return out;
    }
    let lines = wrap(text, width);
    for (index, line) in lines.iter().enumerate() {
        line_segments(line, &mut out);
        let separator = if index + 1 == lines.len() { end } else { "\n" };
        if !separator.is_empty() {
            out.push(Segment {
                text: separator.to_string(),
                style: Style::new(),
            });
        }
    }
    out
}

/// Whether any line of `segments` is wider than `width` cells.
fn exceeds(segments: &[Segment], width: usize) -> bool {
    let mut used = 0usize;
    for segment in segments {
        for (index, piece) in segment.text.split('\n').enumerate() {
            if index > 0 {
                used = 0;
            }
            used += cell_width(piece);
            if used > width {
                return true;
            }
        }
    }
    false
}

/// Cuts every line of `segments` to `width` cells, dropping what is past it, as printing does:
/// a line that fits is left alone, and a run that crosses the edge keeps the cells before it.
pub(crate) fn crop_lines(segments: Vec<Segment>, width: usize) -> Vec<Segment> {
    if !exceeds(&segments, width) {
        return segments;
    }
    let mut out = Vec::with_capacity(segments.len());
    let mut used = 0usize;
    let mut full = false;
    for segment in segments {
        let mut rest = segment.text.as_str();
        while !rest.is_empty() {
            let (piece, newline) = match rest.split_once('\n') {
                Some((piece, tail)) => {
                    rest = tail;
                    (piece, true)
                }
                None => {
                    let piece = rest;
                    rest = "";
                    (piece, false)
                }
            };
            if !piece.is_empty() && !full {
                let cells = cell_width(piece);
                if used + cells <= width {
                    out.push(Segment {
                        text: piece.to_string(),
                        style: segment.style.clone(),
                    });
                    used += cells;
                } else {
                    out.push(Segment {
                        text: set_cell_size(piece, width - used),
                        style: segment.style.clone(),
                    });
                    full = true;
                }
            }
            if newline {
                out.push(Segment {
                    text: "\n".to_string(),
                    style: Style::new(),
                });
                used = 0;
                full = false;
            }
        }
    }
    out
}

/// Writes runs as bytes for a stream with these capabilities: escape sequences only for what
/// the stream shows, none at all when it shows nothing.
pub(crate) fn to_ansi(segments: &[Segment], caps: &Capabilities) -> String {
    let mut out = String::new();
    for segment in segments {
        emit(
            &mut out,
            &segment.text,
            &segment.style,
            caps.color_system,
            caps.attributes,
        );
    }
    out
}

/// The text of the runs with no escape sequences.
pub(crate) fn to_plain(segments: &[Segment]) -> String {
    segments.iter().map(|s| s.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ColorSystem;

    fn caps(color_system: ColorSystem, attributes: bool) -> Capabilities {
        Capabilities {
            color_system,
            attributes,
            is_tty: true,
            width: 80,
            height: 24,
        }
    }

    fn ansi(markup: &str, width: usize, caps: &Capabilities) -> String {
        to_ansi(
            &render_text(&Text::from_markup(markup).unwrap(), width),
            caps,
        )
    }

    #[test]
    fn nested_markup_splits_into_runs_with_combined_styles() {
        let caps = caps(ColorSystem::TrueColor, true);
        assert_eq!(
            ansi("[bold]B[italic]BI[/italic]B[/bold]", 100, &caps),
            "\x1b[1mB\x1b[0m\x1b[1;3mBI\x1b[0m\x1b[1mB\x1b[0m\n"
        );
    }

    #[test]
    fn adjacent_runs_of_one_style_are_not_merged() {
        let caps = caps(ColorSystem::Standard, true);
        assert_eq!(
            ansi("[bold]a[/][bold]b[/]", 100, &caps),
            "\x1b[1ma\x1b[0m\x1b[1mb\x1b[0m\n"
        );
    }

    #[test]
    fn no_color_keeps_attributes_and_none_keeps_nothing() {
        let markup = "[bold red]x[/] y";
        assert_eq!(
            ansi(markup, 100, &caps(ColorSystem::None, true)),
            "\x1b[1mx\x1b[0m y\n"
        );
        assert_eq!(ansi(markup, 100, &caps(ColorSystem::None, false)), "x y\n");
    }

    #[test]
    fn base_style_covers_wrapped_lines_but_not_the_newline() {
        let text = Text::styled("aaa bbb", Style::new().bold());
        let out = to_ansi(&render_text(&text, 4), &caps(ColorSystem::None, true));
        assert_eq!(out, "\x1b[1maaa \x1b[0m\n\x1b[1mbbb\x1b[0m\n");
    }
}
