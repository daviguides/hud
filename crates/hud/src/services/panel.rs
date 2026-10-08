//! A panel spec to styled runs: the body laid out inside its padding, the border drawn around it
//! with the title and subtitle set into the top and bottom edges.

use hud_width::cell_width;

use super::frame::{BOTTOM, MID, TOP, adjust_line, box_rows, markup_text, rule, run, split_lines};
use super::layout::{Measurement, measure_renderable};
use super::measure::{expand_tabs, pad_left, pad_right, truncate};
use super::render::render_text_ending;
use crate::model::{
    Align, Capabilities, Measure, Padding, Panel, Renderable, Segment, Span, Style, Text,
};

/// A body with blank space around it, as the panel prints it: the body fills the width that is
/// left, shorter lines are padded, and the sides and the blank lines are spaces.
struct Padded<'a> {
    body: &'a dyn Renderable,
    padding: Padding,
}

impl Renderable for Padded<'_> {
    fn render(&self, width: usize) -> Vec<Segment> {
        let Padding {
            top,
            right,
            bottom,
            left,
        } = self.padding;
        let null = Style::new();
        let inner = width.saturating_sub(left + right);
        let lines = if inner == 0 {
            Vec::new()
        } else {
            split_lines(self.body.render(inner), inner, Some(&null))
        };
        let mut out = Vec::new();
        let blank = |out: &mut Vec<Segment>| {
            out.push(run(" ".repeat(width), &null));
            out.push(run("\n", &null));
        };
        for _ in 0..top {
            blank(&mut out);
        }
        for line in lines {
            if left > 0 {
                out.push(run(" ".repeat(left), &null));
            }
            out.extend(line);
            if right > 0 {
                out.push(run(" ".repeat(right), &null));
            }
            out.push(run("\n", &null));
        }
        for _ in 0..bottom {
            blank(&mut out);
        }
        out
    }

    fn measure(&self, max_width: usize) -> Measure {
        let extra = (self.padding.left + self.padding.right) as i64;
        let room = max_width as i64;
        if room - extra < 1 {
            return Measure {
                min: max_width,
                max: max_width,
            };
        }
        let body = measure_renderable(self.body, room);
        Measurement {
            min: body.min + extra,
            max: body.max + extra,
        }
        .with_maximum(room)
        .measure()
    }
}

/// A title or subtitle as one line of text: markup read, line breaks turned into spaces, tabs
/// expanded and a space on each side, with the border style under whatever the markup sets.
fn caption(markup: &str, border: &Style) -> Text {
    let mut text = markup_text(markup);
    text.end = String::new();
    if text.plain.contains('\n') {
        text.plain = text.plain.replace('\n', " ");
    }
    text.no_wrap = true;
    let tab_size = text.tab_size;
    expand_tabs(&mut text, tab_size);
    pad_left(&mut text, 1);
    pad_right(&mut text, 1);
    if !border.is_null() {
        let end = text.plain.len();
        text.spans.insert(
            0,
            Span {
                start: 0,
                end,
                style: border.clone(),
            },
        );
    }
    text
}

/// `caption` cut to `width` cells and set into a run of `fill` characters in the border style,
/// where `align` says.
fn set_into(caption: &Text, width: usize, align: Align, fill: char, border: &Style) -> Text {
    let mut text = caption.clone();
    let overflow = text.overflow;
    truncate(&mut text, width, overflow, false);
    let excess = width.saturating_sub(cell_width(&text.plain));
    if excess == 0 {
        return text;
    }
    let (left, right) = match align {
        Align::Left => (0, excess),
        Align::Center => (excess / 2, excess - excess / 2),
        Align::Right => (excess, 0),
    };
    let fill_bytes = fill.len_utf8();
    let shift = left * fill_bytes;
    let mut plain = String::with_capacity(text.plain.len() + excess * fill_bytes);
    plain.extend(core::iter::repeat_n(fill, left));
    plain.push_str(&text.plain);
    plain.extend(core::iter::repeat_n(fill, right));
    let mut spans = Vec::with_capacity(text.spans.len() + 2);
    if left > 0 && !border.is_null() {
        spans.push(Span {
            start: 0,
            end: shift,
            style: border.clone(),
        });
    }
    spans.extend(text.spans.iter().map(|span| Span {
        start: span.start + shift,
        end: span.end + shift,
        style: span.style.clone(),
    }));
    if right > 0 && !border.is_null() {
        let start = shift + text.plain.len();
        spans.push(Span {
            start,
            end: start + right * fill_bytes,
            style: border.clone(),
        });
    }
    text.plain = plain;
    text.spans = spans;
    text
}

/// The body as the panel renders it: padded when there is padding.
fn with_padding<R>(panel: &Panel, with: impl FnOnce(&dyn Renderable) -> R) -> R {
    if panel.padding.is_none() {
        with(&*panel.body.0)
    } else {
        with(&Padded {
            body: &*panel.body.0,
            padding: panel.padding,
        })
    }
}

fn title_of(markup: &Option<String>, border: &Style) -> Option<Text> {
    markup
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(|text| caption(text, border))
}

/// One border edge: `rule` when there is nothing to set in it, otherwise the corner, the
/// caption between runs of the edge character, and the other corner.
fn edge(
    parts: [char; 4],
    caption: Option<&Text>,
    align: Align,
    total: usize,
    border: &Style,
) -> Vec<Segment> {
    let Some(caption) = caption.filter(|_| total > 4) else {
        return vec![run(rule(parts, &[total - 2]), border)];
    };
    let mut out = vec![run(format!("{}{}", parts[0], parts[1]), border)];
    let set = set_into(caption, total - 4, align, parts[1], border);
    out.extend(render_text_ending(&set, total - 4, ""));
    out.push(run(format!("{}{}", parts[1], parts[3]), border));
    out
}

/// Renders `panel` for a console `width` cells wide: the top edge, the body between the sides,
/// the bottom edge.
pub(crate) fn render_panel(panel: &Panel, width: usize) -> Vec<Segment> {
    render_panel_in(panel, width, None)
}

/// A line of `width` plain spaces.
fn blank_line(width: usize) -> Vec<Segment> {
    vec![run(" ".repeat(width), &Style::new())]
}

/// The body's lines when the panel has `height` lines to fill (Rich passes the height of a layout
/// region down): the body is cropped or padded with blank lines to the room inside the panel,
/// padding included, and so is the panel's inside.
fn body_lines_in(
    panel: &Panel,
    child_width: usize,
    height: usize,
    caps: &Capabilities,
) -> Vec<Vec<Segment>> {
    let child_height = height.saturating_sub(2);
    let Padding {
        top,
        right,
        bottom,
        left,
    } = panel.padding;
    let inner_width = child_width.saturating_sub(left + right);
    let inner_height = child_height.saturating_sub(top + bottom);
    let null = Style::new();
    let mut inner = if inner_width == 0 {
        Vec::new()
    } else {
        split_lines(
            panel.body.0.render_region(inner_width, inner_height, caps),
            inner_width,
            Some(&null),
        )
    };
    if !panel.padding.is_none() {
        inner.truncate(inner_height);
        while inner.len() < inner_height {
            inner.push(blank_line(inner_width));
        }
    }
    let mut lines = Vec::with_capacity(inner.len() + top + bottom);
    lines.extend((0..top).map(|_| blank_line(child_width)));
    for mut line in inner {
        if left > 0 {
            line.insert(0, run(" ".repeat(left), &null));
        }
        if right > 0 {
            line.push(run(" ".repeat(right), &null));
        }
        lines.push(line);
    }
    lines.extend((0..bottom).map(|_| blank_line(child_width)));
    lines.truncate(child_height);
    while lines.len() < child_height {
        lines.push(blank_line(child_width));
    }
    for line in &mut lines {
        adjust_line(line, child_width, &null, true);
    }
    lines
}

/// Like [`render_panel`], for a box `region` lines tall when there is one: the panel then fills
/// the region from its top edge to its bottom edge.
pub(crate) fn render_panel_in(
    panel: &Panel,
    width: usize,
    region: Option<(usize, &Capabilities)>,
) -> Vec<Segment> {
    let rows = box_rows(panel.box_style);
    let border = &panel.border_style;
    let title = title_of(&panel.title, border);
    let subtitle = title_of(&panel.subtitle, border);
    let room = width.saturating_sub(2);
    let mut child_width = if panel.expand {
        room
    } else {
        with_padding(panel, |body| measure_renderable(body, room as i64).max) as usize
    };
    if let Some(title) = &title {
        child_width = room.min(child_width.max(cell_width(&title.plain) + 2));
    }
    let total = child_width + 2;
    let lines = match region {
        Some((height, caps)) if height > 0 => body_lines_in(panel, child_width, height, caps),
        _ if child_width == 0 => Vec::new(),
        _ => with_padding(panel, |body| {
            split_lines(body.render(child_width), child_width, Some(&Style::new()))
        }),
    };

    let newline = || run("\n", &Style::new());
    let mut out = edge(rows[TOP], title.as_ref(), panel.title_align, total, border);
    out.push(newline());
    for line in lines {
        out.push(run(rows[MID][0].to_string(), border));
        out.extend(line);
        out.push(run(rows[MID][3].to_string(), border));
        out.push(newline());
    }
    out.extend(edge(
        rows[BOTTOM],
        subtitle.as_ref(),
        panel.subtitle_align,
        total,
        border,
    ));
    out.push(newline());
    out
}

/// The widths a panel asks for when no more than `max_width` cells are available: its body, or
/// its title if that is wider, plus the padding and the two sides.
pub(crate) fn measure_panel(panel: &Panel, max_width: usize) -> Measure {
    let extra = panel.padding.left + panel.padding.right + 2;
    let room = max_width as i64 - extra as i64;
    let mut best = measure_renderable(&*panel.body.0, room);
    if let Some(title) = title_of(&panel.title, &Style::new()) {
        let measured = measure_renderable(&title, room);
        best = Measurement {
            min: best.min.max(measured.min),
            max: best.max.max(measured.max),
        };
    }
    let width = best.max.max(0) as usize + extra;
    Measure {
        min: width,
        max: width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BoxStyle, Table};
    use crate::services::render::to_plain;

    fn plain(panel: &Panel, width: usize) -> String {
        to_plain(&render_panel(panel, width))
    }

    #[test]
    fn a_titled_panel_sets_the_title_into_the_top_edge() {
        let panel = Panel::new("hello").title("T");
        assert_eq!(
            plain(&panel, 20),
            "╭─────── T ────────╮\n│ hello            │\n╰──────────────────╯\n"
        );
    }

    #[test]
    fn a_fitting_panel_is_as_wide_as_its_body_and_padding() {
        let panel = Panel::fit("hello");
        assert_eq!(plain(&panel, 40), "╭───────╮\n│ hello │\n╰───────╯\n");
    }

    #[test]
    fn a_title_wider_than_the_body_widens_a_fitting_panel() {
        let panel = Panel::fit("hi").title("a longer title");
        let text = plain(&panel, 40);
        assert!(text.starts_with("╭─ a longer title ─╮\n│ hi"));
    }

    #[test]
    fn vertical_padding_adds_blank_lines_inside_the_sides() {
        let panel = Panel::fit("x").padding((1, 1)).box_style(BoxStyle::Ascii);
        assert_eq!(plain(&panel, 40), "+---+\n|   |\n| x |\n|   |\n+---+\n");
    }

    #[test]
    fn subtitle_and_alignment_set_into_the_bottom_edge() {
        let panel = Panel::new("x")
            .subtitle("end")
            .subtitle_align(Align::Right)
            .box_style(BoxStyle::Ascii);
        assert_eq!(
            plain(&panel, 20),
            "+------------------+\n| x                |\n+------------ end -+\n"
        );
    }

    #[test]
    fn a_title_longer_than_the_edge_is_cut_inside_it() {
        let panel = Panel::new("x")
            .title("abcdefghijklmnop")
            .box_style(BoxStyle::Ascii);
        let text = plain(&panel, 12);
        assert_eq!(text.lines().next(), Some("+- abcdefg-+"));
    }

    #[test]
    fn a_panel_holds_a_table_and_another_panel() {
        let table = Table::new().column("A").row(["x"]);
        let panel = Panel::fit(Panel::fit(table)).box_style(BoxStyle::Square);
        let text = plain(&panel, 40);
        assert!(text.lines().all(|line| line.chars().count() == 13));
        assert_eq!(text.lines().count(), 9);
    }
}
