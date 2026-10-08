//! The Rich-shaped names and signatures added in v0.7, each against what Rich 15.0.0 does.

use hud::{
    Color, ColorSystem, Console, Overflow, Padding, Progress, Stream, Style, Text, cell_len,
    cell_width,
};

fn bold() -> Style {
    Style::new().bold()
}

#[test]
fn stylize_takes_the_style_first_and_a_range_with_open_ends() {
    let mut text = Text::new("hello world");
    text.stylize(bold(), ..5);
    text.stylize(Style::new().italic(), 6..);
    text.stylize(Style::new().underline(), ..);
    let spans: Vec<(usize, usize)> = text.spans().iter().map(|s| (s.start, s.end)).collect();
    assert_eq!(spans, [(0, 5), (6, 11), (0, 11)]);
}

#[test]
fn stylize_clamps_and_ignores_empty_ranges() {
    let mut text = Text::new("héllo");
    text.stylize(bold(), 0..99);
    text.stylize(bold(), 3..3);
    text.stylize(Style::new(), ..);
    assert_eq!(text.spans().len(), 1);
    assert_eq!(text.spans()[0].end, "héllo".len());
    let mut moved = Text::new("é");
    moved.stylize(bold(), 1..2);
    assert_eq!((moved.spans()[0].start, moved.spans()[0].end), (0, 2));
}

#[test]
fn truncate_crops_like_rich() {
    let mut text = Text::new("hello world");
    text.truncate(8, Overflow::Ellipsis, false);
    assert_eq!(text.plain(), "hello w\u{2026}");
    let mut cut = Text::new("日本語です");
    cut.truncate(5, Overflow::Crop, false);
    assert_eq!(
        cut.plain(),
        "日本 ",
        "Rich pads the cell a wide character would overflow"
    );
    let mut padded = Text::new("ab");
    padded.truncate(5, Overflow::Crop, true);
    assert_eq!(padded.plain(), "ab   ");
}

#[test]
fn wrap_breaks_at_words_and_keeps_the_text_settings() {
    let lines = Text::new("aaaa bbbb cccc dddd").wrap(10);
    let plain: Vec<&str> = lines.iter().map(Text::plain).collect();
    assert_eq!(plain, ["aaaa bbbb ", "cccc dddd"]);
    let folded = Text::new("aaaaaaaaaaaaaaaaaaaaaaaa").wrap(10);
    assert_eq!(folded.len(), 3);
}

#[test]
fn color_parse_reads_every_form_rich_reads() {
    assert_eq!(Color::parse("red").unwrap(), Color::RED);
    assert_eq!(Color::parse("  Bright_Blue ").unwrap(), Color::Standard(12));
    assert_eq!(Color::parse("color(208)").unwrap(), Color::EightBit(208));
    assert_eq!(Color::parse("#ff8800").unwrap(), Color::rgb(255, 136, 0));
    assert_eq!(Color::parse("rgb(1,2,3)").unwrap(), Color::rgb(1, 2, 3));
    assert_eq!(Color::parse("default").unwrap(), Color::Default);
    assert!(Color::parse("nonsense").is_err());
}

#[test]
fn cell_len_is_cell_width() {
    for text in ["", "abc", "日本語", "👨‍👩‍👧", "e\u{301}"] {
        assert_eq!(cell_len(text), cell_width(text));
    }
}

#[test]
fn console_accessors_report_the_profile() {
    let console = Console::builder()
        .width(72)
        .color_system(ColorSystem::EightBit)
        .attributes(true)
        .force_terminal(true)
        .build();
    assert_eq!(console.width(), 72);
    assert_eq!(console.color_system(), ColorSystem::EightBit);
    assert!(!console.no_color());
    assert!(console.is_terminal());
    assert!(console.force_terminal());

    let plain = Console::builder()
        .width(40)
        .plain()
        .force_terminal(false)
        .build();
    assert!(plain.no_color());
    assert!(!plain.is_terminal());
    assert!(!plain.force_terminal());

    let kept = Console::builder()
        .color_system(ColorSystem::None)
        .attributes(true)
        .build();
    assert!(
        kept.no_color(),
        "NO_COLOR removes color only; attributes stay on"
    );
    assert!(kept.capabilities().attributes);
    let _ = Stream::Stdout;
}

#[test]
fn padding_is_the_renderable_wrapper_and_pad_the_spacing() {
    let console = Console::builder().width(10).plain().build();
    let padded = Padding::new("ok", (0, 1));
    assert_eq!(console.render_to_plain(&padded), " ok       \n");
}

#[test]
fn progress_update_applies_when_the_statement_ends() {
    let progress = Progress::builder().disable(true).build();
    let task = progress.add_task("fetch", 10);
    progress.update(&task).advance(3);
    assert_eq!(task.completed(), 3);
    progress.update(&task).total(20).completed(7);
    assert_eq!((task.completed(), task.total()), (7, 20));
    progress
        .update(&task)
        .description("[bold]index[/]")
        .advance(2);
    assert_eq!(task.completed(), 9);
    progress.update(&task).advance(100);
    assert_eq!(task.completed(), 109);
    assert!(task.is_finished());
}

#[test]
fn progress_update_completed_wins_over_advance_like_rich() {
    let progress = Progress::builder().disable(true).build();
    let task = progress.add_task("t", 10);
    progress.update(&task).advance(4).completed(2);
    assert_eq!(task.completed(), 2);
}

#[test]
fn hidden_tasks_are_not_drawn_and_come_back() {
    let progress = Progress::builder().disable(true).build();
    let first = progress.add_task("first", 10);
    let second = progress.add_task("second", 10);
    let console = Console::builder().width(60).plain().build();
    let both = console.render_to_plain(&progress);
    assert!(both.contains("first") && both.contains("second"));
    progress.update(&first).visible(false);
    let one = console.render_to_plain(&progress);
    assert!(!one.contains("first") && one.contains("second"));
    assert!(!first.is_visible() && second.is_visible());
    progress.update(&first).visible(true);
    assert_eq!(console.render_to_plain(&progress), both);
}

#[test]
fn truncate_keeps_spans_inside_the_text_when_an_ellipsis_changes_the_byte_length() {
    // U+10FFFF is four bytes and one cell; the ellipsis that replaces it is three bytes.
    let mut text = Text::new("\u{10ffff}");
    text.stylize(bold(), ..);
    text.truncate(0, Overflow::Ellipsis, false);
    assert_eq!(text.plain(), "\u{2026}");
    for span in text.spans() {
        assert!(span.end <= text.plain().len() && text.plain().is_char_boundary(span.end));
    }
}

#[test]
fn progress_update_refresh_draws_now() {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let capture = Capture::default();
    let progress = Progress::builder()
        .console(Console::builder().width(40).plain().build())
        .writer(capture.clone())
        .interactive(true)
        .auto_refresh(false)
        .build();
    let task = progress.add_task("job", 10);
    let before = capture.0.lock().unwrap().len();
    progress.update(&task).completed(5);
    assert_eq!(
        capture.0.lock().unwrap().len(),
        before,
        "no refresh requested, nothing drawn"
    );
    progress.update(&task).completed(7).refresh(true);
    assert!(capture.0.lock().unwrap().len() > before);
    assert!(String::from_utf8_lossy(&capture.0.lock().unwrap()).contains("70%"));
}
