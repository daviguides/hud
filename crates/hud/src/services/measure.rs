//! Wrapping, tab expansion, justification and cropping of styled text, by cell width and
//! never inside a grapheme cluster.

use hud_width::{cell_width, clusters, fold};

use crate::model::{Justify, Overflow, Span, Style, Text};

/// A copy of the text properties with `plain` and `spans` replaced.
fn with_content(text: &Text, plain: String, spans: Vec<Span>) -> Text {
    Text {
        plain,
        spans,
        ..text.clone_without_content()
    }
}

impl Text {
    /// Everything but the plain string and the spans.
    fn clone_without_content(&self) -> Text {
        Text {
            plain: String::new(),
            style: self.style.clone(),
            spans: Vec::new(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            tab_size: self.tab_size,
            end: self.end.clone(),
        }
    }
}

/// The part of `text` between two byte offsets, with spans clipped to it and moved to start
/// at zero. Spans left empty by the clip are dropped.
fn slice(text: &Text, start: usize, end: usize) -> Text {
    let spans = text
        .spans
        .iter()
        .filter_map(|span| {
            let from = span.start.max(start);
            let to = span.end.min(end);
            (to > from).then(|| Span {
                start: from - start,
                end: to - start,
                style: span.style.clone(),
            })
        })
        .collect();
    with_content(text, text.plain[start..end].to_string(), spans)
}

/// Cuts `text` at the given ascending byte offsets.
fn divide(text: &Text, offsets: &[usize]) -> Vec<Text> {
    if offsets.is_empty() {
        return vec![text.clone()];
    }
    let mut lines = Vec::with_capacity(offsets.len() + 1);
    let mut start = 0;
    for &offset in offsets {
        lines.push(slice(text, start, offset));
        start = offset;
    }
    lines.push(slice(text, start, text.plain.len()));
    lines
}

/// Replaces the plain text. When it gets shorter, spans keep the characters they covered and
/// are clipped to the new length, counting characters, so text cut short or ended with an
/// ellipsis keeps its styles on character boundaries.
fn set_plain(text: &mut Text, plain: String) {
    let new_chars = plain.chars().count();
    if new_chars < text.plain.chars().count() && !text.spans.is_empty() {
        let offsets: Vec<usize> = plain
            .char_indices()
            .map(|(at, _)| at)
            .chain(core::iter::once(plain.len()))
            .collect();
        let old = &text.plain;
        let count = |at: usize| old[..at].chars().count();
        text.spans = text
            .spans
            .iter()
            .filter_map(|span| {
                let start = count(span.start);
                (start < new_chars).then(|| Span {
                    start: offsets[start],
                    end: offsets[count(span.end).min(new_chars).max(start)],
                    style: span.style.clone(),
                })
            })
            .collect();
    }
    text.plain = plain;
}

/// `(byte offset, word)` for each `\s*\S+\s*` run of `text`.
fn words(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut position = 0;
    while position < text.len() {
        let rest = &text[position..];
        let lead = rest.len() - rest.trim_start().len();
        let after_lead = &rest[lead..];
        let word = after_lead
            .find(char::is_whitespace)
            .unwrap_or(after_lead.len());
        if word == 0 {
            break;
        }
        let after_word = &after_lead[word..];
        let trail = after_word.len() - after_word.trim_start().len();
        let total = lead + word + trail;
        out.push((position, &rest[..total]));
        position += total;
    }
    out
}

/// Byte offsets where a line of `text` breaks so that every line fits `width` cells: before the
/// first word that does not fit, and inside a word longer than a line when `fold_words`.
pub(crate) fn divide_line(text: &str, width: usize, fold_words: bool) -> Vec<usize> {
    let mut breaks = Vec::new();
    let mut cell_offset = 0usize;
    for (start, word) in words(text) {
        let word_length = cell_width(word.trim_end());
        if cell_offset + word_length <= width {
            cell_offset += cell_width(word);
        } else if word_length > width {
            if fold_words {
                let pieces: Vec<&str> = fold(word, width).collect();
                let mut at = start;
                for (index, piece) in pieces.iter().enumerate() {
                    if at > 0 {
                        breaks.push(at);
                    }
                    if index + 1 == pieces.len() {
                        cell_offset = cell_width(piece);
                    } else {
                        at += piece.len();
                    }
                }
            } else {
                if start > 0 {
                    breaks.push(start);
                }
                cell_offset = cell_width(word);
            }
        } else if cell_offset > 0 && start > 0 {
            breaks.push(start);
            cell_offset = cell_width(word);
        }
    }
    breaks
}

/// The lines of `text` split at `\n`; a trailing newline gives a last empty line.
fn split_newlines(text: &Text) -> Vec<Text> {
    if !text.plain.contains('\n') {
        return vec![text.clone()];
    }
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, _) in text.plain.match_indices('\n') {
        lines.push(slice(text, start, index));
        start = index + 1;
    }
    lines.push(slice(text, start, text.plain.len()));
    lines
}

/// Joins pieces into one text: each piece with a style of its own gets a span over it, as the
/// piece's spans are moved to where the piece lands.
fn join(like: &Text, pieces: &[Text]) -> Text {
    let mut plain = String::new();
    let mut spans = Vec::new();
    for piece in pieces {
        let offset = plain.len();
        plain.push_str(&piece.plain);
        if !piece.style.is_null() {
            spans.push(Span {
                start: offset,
                end: plain.len(),
                style: piece.style.clone(),
            });
        }
        spans.extend(piece.spans.iter().map(|s| Span {
            start: s.start + offset,
            end: s.end + offset,
            style: s.style.clone(),
        }));
    }
    with_content(like, plain, spans)
}

/// Replaces each tab with a space and pads up to the next tab stop, the padding taking the
/// style of the spans that reach the tab.
fn expand_tabs(line: &mut Text, tab_size: usize) {
    let tab_size = tab_size.max(1);
    let offsets: Vec<usize> = line
        .plain
        .char_indices()
        .filter(|&(_, c)| c == '\t')
        .map(|(i, _)| i + 1)
        .collect();
    let mut parts = divide(line, &offsets);
    if line.plain.ends_with('\t') {
        parts.pop();
    }
    let mut cell_position = 0;
    for part in &mut parts {
        if part.plain.ends_with('\t') {
            part.plain.pop();
            part.plain.push(' ');
            cell_position += cell_width(&part.plain);
            let remainder = cell_position % tab_size;
            if remainder != 0 {
                let spaces = tab_size - remainder;
                let end = part.plain.len();
                for span in &mut part.spans {
                    if span.end >= end {
                        span.end += spaces;
                    }
                }
                part.plain.extend(core::iter::repeat_n(' ', spaces));
                cell_position += spaces;
            }
        } else {
            cell_position += cell_width(&part.plain);
        }
    }
    let joined = join(line, &parts);
    line.plain = joined.plain;
    line.spans = joined.spans;
}

/// Cuts whitespace from the end of a line that is wider than `size`, as much as the excess.
fn rstrip_end(line: &mut Text, size: usize) {
    let width = cell_width(&line.plain);
    if width <= size {
        return;
    }
    let excess = width - size;
    let trimmed = line.plain.trim_end();
    let whitespace = line.plain[trimmed.len()..].chars().count();
    let cut = whitespace.min(excess);
    if cut == 0 {
        return;
    }
    let keep = line.plain.chars().count() - cut;
    let end = line
        .plain
        .char_indices()
        .nth(keep)
        .map_or(line.plain.len(), |(i, _)| i);
    let plain = line.plain[..end].to_string();
    set_plain(line, plain);
}

/// `text` cut to exactly `total` cells: whole clusters up to the width, and a space when a wide
/// cluster would straddle the edge.
pub(crate) fn set_cell_size(text: &str, total: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for cluster in clusters(text) {
        let width = hud_width::cell_width_of_cluster(cluster);
        if used + width > total {
            break;
        }
        out.push_str(cluster);
        used += width;
    }
    out.extend(core::iter::repeat_n(' ', total - used));
    out
}

/// Crops a line to `max_width` by the overflow setting; with `pad`, pads a shorter line to it.
fn truncate(line: &mut Text, max_width: usize, overflow: Overflow, pad: bool) {
    if overflow == Overflow::Ignore {
        return;
    }
    let length = cell_width(&line.plain);
    if length > max_width {
        let plain = if overflow == Overflow::Ellipsis {
            let mut cut = set_cell_size(&line.plain, max_width.saturating_sub(1));
            cut.push('\u{2026}');
            cut
        } else {
            set_cell_size(&line.plain, max_width)
        };
        set_plain(line, plain);
    }
    if pad && length < max_width {
        line.plain
            .extend(core::iter::repeat_n(' ', max_width - length));
    }
}

fn pad_left(line: &mut Text, count: usize) {
    if count == 0 {
        return;
    }
    let mut plain = " ".repeat(count);
    plain.push_str(&line.plain);
    line.plain = plain;
    for span in &mut line.spans {
        span.start += count;
        span.end += count;
    }
}

fn pad_right(line: &mut Text, count: usize) {
    line.plain.extend(core::iter::repeat_n(' ', count));
}

/// The style in effect at a byte offset of `word`: its own style plus the spans that cover it.
fn style_at(word: &Text, offset: isize) -> Style {
    let offset = if offset < 0 {
        word.plain.len() as isize + offset
    } else {
        offset
    };
    let mut style = word.style.clone();
    for span in &word.spans {
        if (span.end as isize) > offset && offset >= span.start as isize {
            style = style.combine(&span.style);
        }
    }
    style
}

/// Stretches the spaces between the words of `line` until it fills `width`.
fn justify_full(line: &Text, width: usize) -> Text {
    let mut words: Vec<Text> = {
        let mut out = Vec::new();
        let mut start = 0;
        for (index, _) in line.plain.match_indices(' ') {
            out.push(slice(line, start, index));
            start = index + 1;
        }
        out.push(slice(line, start, line.plain.len()));
        out
    };
    if line.plain.ends_with(' ') {
        words.pop();
    }
    let words_size: usize = words.iter().map(|w| cell_width(&w.plain)).sum();
    let mut num_spaces = words.len().saturating_sub(1);
    let mut spaces = vec![1usize; num_spaces];
    if !spaces.is_empty() {
        let mut index = 0;
        while words_size + num_spaces < width {
            let slot = spaces.len() - index - 1;
            spaces[slot] += 1;
            num_spaces += 1;
            index = (index + 1) % spaces.len();
        }
    }
    let mut tokens: Vec<Text> = Vec::new();
    for (index, word) in words.iter().enumerate() {
        tokens.push(word.clone());
        if let (Some(count), Some(next)) = (spaces.get(index), words.get(index + 1)) {
            let style = style_at(word, -1);
            let next_style = style_at(next, 0);
            let space_style = if style == next_style {
                style
            } else {
                line.style.clone()
            };
            let mut space = Text::styled(" ".repeat(*count), space_style);
            space.end = String::new();
            tokens.push(space);
        }
    }
    let mut blank = line.clone_without_content();
    blank.style = Style::new();
    join(&blank, &tokens)
}

fn justify_lines(lines: &mut [Text], width: usize, justify: Justify, overflow: Overflow) {
    match justify {
        Justify::Default => {}
        Justify::Left => {
            for line in lines {
                truncate(line, width, overflow, true);
            }
        }
        Justify::Center | Justify::Right => {
            for line in lines {
                let trimmed = line.plain.trim_end().to_string();
                set_plain(line, trimmed);
                truncate(line, width, overflow, false);
                let free = width.saturating_sub(cell_width(&line.plain));
                if justify == Justify::Center {
                    pad_left(line, free / 2);
                    let free = width.saturating_sub(cell_width(&line.plain));
                    pad_right(line, free);
                } else {
                    pad_left(line, free);
                }
            }
        }
        Justify::Full => {
            let last = lines.len().saturating_sub(1);
            for line in &mut lines[..last] {
                *line = justify_full(line, width);
            }
        }
    }
}

/// Breaks `text` into lines for a `width`-cell terminal: at newlines, at word boundaries (or
/// inside a word that is longer than a line), with tabs expanded, then justified and cropped.
pub(crate) fn wrap(text: &Text, width: usize) -> Vec<Text> {
    let ignore = text.overflow == Overflow::Ignore;
    let no_wrap = text.no_wrap || ignore;
    let mut lines = Vec::new();
    for mut line in split_newlines(text) {
        if line.plain.contains('\t') {
            expand_tabs(&mut line, text.tab_size);
        }
        if ignore {
            lines.push(line);
            continue;
        }
        let mut pieces = if no_wrap {
            vec![line]
        } else {
            let offsets = divide_line(&line.plain, width, text.overflow == Overflow::Fold);
            let mut pieces = divide(&line, &offsets);
            for piece in &mut pieces {
                rstrip_end(piece, width);
            }
            pieces
        };
        justify_lines(&mut pieces, width, text.justify, text.overflow);
        for piece in &mut pieces {
            truncate(piece, width, text.overflow, false);
        }
        lines.extend(pieces);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(markup: &str, width: usize) -> Vec<String> {
        let text = Text::from_markup(markup).unwrap();
        wrap(&text, width).into_iter().map(|l| l.plain).collect()
    }

    #[test]
    fn words_wrap_and_keep_their_trailing_space() {
        assert_eq!(
            lines("aaaa bbbb cccc dddd", 10),
            ["aaaa bbbb ", "cccc dddd"]
        );
    }

    #[test]
    fn a_long_word_folds_and_extra_spaces_are_cut_to_the_width() {
        assert_eq!(
            lines("aaaaaaaaaaaaaaaaaaaaaaaa", 10),
            ["aaaaaaaaaa", "aaaaaaaaaa", "aaaa"]
        );
        assert_eq!(lines("aaa   bbb", 5), ["aaa  ", "bbb"]);
    }

    #[test]
    fn newlines_and_tabs() {
        assert_eq!(lines("a\nb\n", 10), ["a", "b", ""]);
        assert_eq!(lines("tab\tx\ty", 30), ["tab     x       y"]);
    }

    #[test]
    fn justify_places_lines_in_the_width() {
        let text = Text::new("hello world foo").justify(Justify::Center);
        let plain: Vec<String> = wrap(&text, 8).into_iter().map(|l| l.plain).collect();
        assert_eq!(plain, [" hello  ", " world  ", "  foo   "]);
        let text = Text::new("hello world foo").justify(Justify::Right);
        assert_eq!(wrap(&text, 20)[0].plain, "     hello world foo");
        let text = Text::new("hello world foo bar").justify(Justify::Full);
        let plain: Vec<String> = wrap(&text, 12).into_iter().map(|l| l.plain).collect();
        assert_eq!(plain, ["hello  world", "foo bar"]);
    }

    #[test]
    fn ellipsis_and_crop() {
        let text = Text::new("abcdefghij")
            .overflow(Overflow::Ellipsis)
            .no_wrap(true);
        assert_eq!(wrap(&text, 5)[0].plain, "abcd\u{2026}");
        let text = Text::new("abcdefghij")
            .overflow(Overflow::Crop)
            .no_wrap(true);
        assert_eq!(wrap(&text, 5)[0].plain, "abcde");
    }
}
