//! A spinner's text: the frame, a space and the message, put together the way Rich's
//! `Text.assemble(frame, " ", text)` does. Pure: the frame is chosen elsewhere.

use crate::model::{Span, Style, Text};

/// The frame alone, styled, when there is no message; otherwise the frame, one space and the
/// message with its own spans moved to where it lands.
pub(crate) fn assemble(frame: &str, style: &Style, message: &Text) -> Text {
    if message.plain().is_empty() {
        return Text::styled(frame, style.clone());
    }
    let mut text = Text::new("");
    text.append(frame, style.clone());
    text.append(" ", Style::new());
    let base = text.plain.len();
    if !message.style().is_null() && !message.plain().is_empty() {
        text.spans.push(Span {
            start: base,
            end: base + message.plain().len(),
            style: message.style().clone(),
        });
    }
    text.plain.push_str(message.plain());
    for span in message.spans() {
        text.spans.push(Span {
            start: span.start + base,
            end: span.end + base,
            style: span.style.clone(),
        });
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::frame::markup_text;

    #[test]
    fn a_message_follows_the_frame_after_a_space_and_keeps_its_styles() {
        let text = assemble("⠋", &Style::new(), &markup_text("[bold]go[/] now"));
        assert_eq!(text.plain(), "⠋ go now");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(
            (text.spans()[0].start, text.spans()[0].end),
            ("⠋ ".len(), "⠋ go".len())
        );
    }

    #[test]
    fn no_message_means_the_bare_frame() {
        let text = assemble("⠋", &Style::new(), &markup_text(""));
        assert_eq!(text.plain(), "⠋");
    }
}
