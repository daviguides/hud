//! Inline markup: `[bold red]text[/]`, nested and adjacent tags, `\[` for a literal bracket.

use super::style;
use crate::model::{MarkupError, Span, Style, Text};

/// A tag as written: its name and the text after `=`, when there is one.
struct Tag<'a> {
    name: &'a str,
    parameters: Option<&'a str>,
    /// Byte offset of the `[`.
    at: usize,
    /// Byte offset one past the `]`.
    end: usize,
}

enum Piece<'a> {
    Text(&'a str),
    Backslashes(usize),
    Tag(Tag<'a>),
}

fn is_tag_start(byte: u8) -> bool {
    byte.is_ascii_lowercase() || matches!(byte, b'#' | b'/' | b'@')
}

/// The end (one past the `]`) of a tag whose `[` is at `open`, if one starts there: the first
/// character after `[` is a lowercase letter, `#`, `/` or `@`, and a `]` follows with no `[`
/// in between.
fn tag_end(markup: &str, open: usize) -> Option<usize> {
    let bytes = markup.as_bytes();
    if !is_tag_start(*bytes.get(open + 1)?) {
        return None;
    }
    for (offset, &byte) in bytes[open + 2..].iter().enumerate() {
        match byte {
            b']' => return Some(open + 2 + offset + 1),
            b'[' => return None,
            _ => {}
        }
    }
    None
}

fn scan(markup: &str) -> Vec<Piece<'_>> {
    let bytes = markup.as_bytes();
    let mut pieces = Vec::new();
    let mut position = 0;
    let mut search = 0;
    while let Some(found) = markup[search..].find('[') {
        let open = search + found;
        search = open + 1;
        let Some(end) = tag_end(markup, open) else {
            continue;
        };
        let mut run_start = open;
        while run_start > position && bytes[run_start - 1] == b'\\' {
            run_start -= 1;
        }
        if run_start > position {
            pieces.push(Piece::Text(&markup[position..run_start]));
        }
        let slashes = open - run_start;
        let (literal_pairs, escaped) = (slashes / 2, slashes % 2 == 1);
        if literal_pairs > 0 {
            pieces.push(Piece::Backslashes(literal_pairs));
        }
        position = end;
        search = end;
        if escaped {
            pieces.push(Piece::Text(&markup[open..end]));
            continue;
        }
        let inner = &markup[open + 1..end - 1];
        let (name, parameters) = match inner.split_once('=') {
            Some((name, parameters)) => (name, Some(parameters)),
            None => (inner, None),
        };
        pieces.push(Piece::Tag(Tag {
            name,
            parameters,
            at: open,
            end,
        }));
    }
    if position < markup.len() {
        pieces.push(Piece::Text(&markup[position..]));
    }
    pieces
}

/// The canonical text of a style word list, or the trimmed lowercase text when it is not a
/// style: what two tags are compared by.
fn normalize(name: &str) -> String {
    match style::parse(name) {
        Ok(parsed) => parsed.canonical,
        Err(_) => name.trim().to_lowercase(),
    }
}

fn char_position(markup: &str, byte: usize) -> usize {
    markup[..byte].chars().count()
}

/// Parses markup into plain text and the spans its tags open. A tag whose style does not
/// parse yields a span with a null style, as an unknown style name does in Rich.
pub(crate) fn parse(markup: &str) -> Result<(String, Vec<Span>), MarkupError> {
    let mut plain = String::with_capacity(markup.len());
    let mut stack: Vec<(usize, String, Option<&str>)> = Vec::new();
    let mut closed: Vec<(usize, usize, String)> = Vec::new();
    for piece in scan(markup) {
        match piece {
            // An escaped bracket that does not start a tag loses its backslash too.
            Piece::Text(text) if text.contains("\\[") => plain.push_str(&text.replace("\\[", "[")),
            Piece::Text(text) => plain.push_str(text),
            Piece::Backslashes(count) => plain.extend(core::iter::repeat_n('\\', count)),
            Piece::Tag(tag) => {
                if let Some(rest) = tag.name.strip_prefix('/') {
                    let wanted = rest.trim();
                    let popped = if wanted.is_empty() {
                        stack.pop().ok_or_else(|| {
                            MarkupError::new(format!(
                                "closing tag '[/]' at position {} has nothing to close",
                                char_position(markup, tag.at)
                            ))
                        })?
                    } else {
                        let wanted = normalize(wanted);
                        let index = stack
                            .iter()
                            .rposition(|(_, name, _)| *name == wanted)
                            .ok_or_else(|| {
                                MarkupError::new(format!(
                                    "closing tag '{}' at position {} doesn't match any open tag",
                                    &markup[tag.at..tag.end],
                                    char_position(markup, tag.at)
                                ))
                            })?;
                        stack.remove(index)
                    };
                    closed.push((popped.0, plain.len(), definition(&popped.1, popped.2)));
                } else {
                    stack.push((plain.len(), normalize(tag.name), tag.parameters));
                }
            }
        }
    }
    while let Some((start, name, parameters)) = stack.pop() {
        closed.push((start, plain.len(), definition(&name, parameters)));
    }
    // Outer spans first, so an inner span's style wins where they overlap.
    closed.reverse();
    closed.sort_by_key(|&(start, _, _)| start);
    let spans = closed
        .into_iter()
        .map(|(start, end, definition)| Span {
            start,
            end,
            style: style::parse(&definition)
                .map(|parsed| parsed.style)
                .unwrap_or_else(|_| Style::new()),
        })
        .collect();
    Ok((plain, spans))
}

fn definition(name: &str, parameters: Option<&str>) -> String {
    match parameters {
        Some(parameters) => format!("{name} {parameters}"),
        None => name.to_string(),
    }
}

impl Text {
    /// Builds text from inline markup: `[bold red]warning[/]`, nested tags, `[/]` to close the
    /// innermost tag, `\[` for a literal bracket. A tag left open runs to the end. A style that
    /// does not parse is ignored and its text kept; a closing tag with nothing to close is an
    /// error.
    ///
    /// ```
    /// use hud::Text;
    ///
    /// let text = Text::from_markup("[bold]Deploy[/bold] [green]ok[/green]")?;
    /// assert_eq!(text.plain(), "Deploy ok");
    /// assert_eq!(text.spans().len(), 2);
    /// assert!(Text::from_markup("oops[/]").is_err());
    /// # Ok::<(), hud::MarkupError>(())
    /// ```
    pub fn from_markup(markup: &str) -> Result<Text, MarkupError> {
        let (plain, spans) = parse(markup)?;
        let mut text = Text::new(plain);
        text.spans = spans;
        Ok(text)
    }
}

/// Escapes markup tags in `text` so they print literally.
///
/// ```
/// use hud::{Text, escape};
///
/// let text = Text::from_markup(&escape("[bold]not a tag[/bold]"))?;
/// assert_eq!(text.plain(), "[bold]not a tag[/bold]");
/// # Ok::<(), hud::MarkupError>(())
/// ```
pub fn escape(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 2);
    let mut position = 0;
    let mut search = 0;
    while let Some(found) = text[search..].find('[') {
        let open = search + found;
        search = open + 1;
        let Some(end) = tag_end(text, open) else {
            continue;
        };
        let mut run_start = open;
        while run_start > position && bytes[run_start - 1] == b'\\' {
            run_start -= 1;
        }
        out.push_str(&text[position..run_start]);
        let slashes = &text[run_start..open];
        out.push_str(slashes);
        out.push_str(slashes);
        out.push('\\');
        out.push_str(&text[open..end]);
        position = end;
        search = end;
    }
    out.push_str(&text[position..]);
    if out.ends_with('\\') && !out.ends_with("\\\\") {
        out.push('\\');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(markup: &str) -> Vec<(usize, usize, String)> {
        let (_, spans) = parse(markup).unwrap();
        spans
            .into_iter()
            .map(|s| (s.start, s.end, s.style.to_string()))
            .collect()
    }

    #[test]
    fn nested_tags_put_the_outer_span_first() {
        assert_eq!(
            spans("[bold]a[italic]b[/italic]c[/bold]"),
            [(0, 3, "bold".to_string()), (1, 2, "italic".to_string())]
        );
    }

    #[test]
    fn slash_closes_the_innermost_and_names_close_by_style() {
        assert_eq!(spans("[b][i]x[/b][/i]").len(), 2);
        assert_eq!(parse("[bold red]x[/red bold]").unwrap().1.len(), 1);
    }

    #[test]
    fn backslash_pairs_stay_and_odd_ones_escape_the_tag() {
        assert_eq!(parse("\\[red]x").unwrap().0, "[red]x");
        assert_eq!(parse("\\\\[red]x").unwrap().0, "\\x");
        assert_eq!(parse("a\\[1] b").unwrap().0, "a[1] b");
        assert_eq!(parse("[1] [Bold] [ ]").unwrap().0, "[1] [Bold] [ ]");
    }

    #[test]
    fn unknown_styles_are_null_spans_and_unclosed_tags_run_to_the_end() {
        let (plain, spans) = parse("[nope]x[bold]y").unwrap();
        assert_eq!(plain, "xy");
        assert!(spans[0].style.is_null());
        assert_eq!((spans[1].start, spans[1].end), (1, 2));
    }

    #[test]
    fn closing_errors_name_the_tag_and_position() {
        let err = parse("ab[/]").unwrap_err();
        assert_eq!(
            err.to_string(),
            "closing tag '[/]' at position 2 has nothing to close"
        );
        let err = parse("[bold]a[/italic]").unwrap_err();
        assert!(err.to_string().contains("'[/italic]' at position 7"));
    }

    #[test]
    fn escape_round_trips() {
        for text in ["[bold]x[/bold]", "a\\[red]", "plain", "[1] [bold]"] {
            assert_eq!(
                Text::from_markup(&escape(text)).unwrap().plain(),
                text,
                "{text}"
            );
        }
    }
}
