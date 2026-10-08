use rich_rust::prelude::*;

fn main() {
    let console = Console::new();
    let line = |text: &str, style: Style| vec![Segment::styled(text.to_string(), style)];
    let plain = Style::new();
    let lines = vec![
        line("could not read config", Style::new().bold()),
        line("", plain.clone()),
        line("Caused by:", Style::new().dim()),
        line("    0: No such file or directory (os error 2)", plain.clone()),
        line("    1: path: /etc/hud/config.toml", plain.clone()),
        line("", plain),
        line("hint: run again with --verbose for details", Style::parse("cyan").unwrap_or_default()),
    ];
    let panel = Panel::new(lines)
        .title("Error")
        .border_style(Style::parse("red").unwrap_or_default());
    console.print_renderable(&panel);
}
