//! An error report to a panel: the message in bold, the numbered causes under a dim heading and
//! the hint in cyan, in a red rounded panel titled `Error`.

use super::panel::{measure_panel, render_panel};
use crate::model::{Body, Color, ErrorReport, Group, Measure, Panel, Segment, Style, Text};

fn panel(report: &ErrorReport) -> Panel {
    let blank = || Text::new("");
    let mut lines = Group::new().push(Text::styled(report.message.clone(), Style::new().bold()));
    if !report.causes.is_empty() {
        lines = lines
            .push(blank())
            .push(Text::styled("Caused by:", Style::new().dim()));
        for (number, cause) in report.causes.iter().enumerate() {
            lines = lines.push(Text::new(format!("    {number}: {cause}")));
        }
    }
    if let Some(hint) = &report.hint {
        lines = lines.push(blank()).push(Text::styled(
            format!("hint: {hint}"),
            Style::new().color(Color::CYAN),
        ));
    }
    Panel::new(Body::new(lines))
        .title("Error")
        .border_style(Style::new().color(Color::RED))
}

pub(crate) fn render_report(report: &ErrorReport, width: usize) -> Vec<Segment> {
    render_panel(&panel(report), width)
}

pub(crate) fn measure_report(report: &ErrorReport, max_width: usize) -> Measure {
    measure_panel(&panel(report), max_width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Console;

    fn plain(report: &ErrorReport, width: u16) -> String {
        Console::builder()
            .width(width)
            .plain()
            .build()
            .render_to_plain(report)
    }

    #[test]
    fn a_message_alone_is_one_line_in_the_panel() {
        let out = plain(&ErrorReport::new("boom"), 20);
        assert_eq!(
            out,
            "╭───── Error ──────╮\n│ boom             │\n╰──────────────────╯\n"
        );
    }

    #[test]
    fn brackets_in_an_error_print_as_written() {
        let out = plain(&ErrorReport::new("bad [bold]tag[/]").cause("[red]x"), 30);
        assert!(out.contains("bad [bold]tag[/]"));
        assert!(out.contains("    0: [red]x"));
    }

    #[test]
    fn a_long_cause_wraps_inside_the_panel_without_widening_it() {
        let out = plain(&ErrorReport::new("e").cause("word ".repeat(20)), 30);
        assert!(out.lines().all(|line| hud_width::cell_width(line) == 30));
    }

    #[test]
    fn hint_comes_after_the_causes_and_is_separated_by_a_blank_line() {
        let out = plain(&ErrorReport::new("m").cause("c").hint("h"), 20);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[4].contains("0: c"));
        assert!(lines[5].replace(['│', ' '], "").is_empty());
        assert!(lines[6].contains("hint: h"));
    }
}
