use rich_rs::{Console, Group, Panel, Style, Text};

fn main() {
    let mut console = Console::new();
    if let Some(width) = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()) {
        console.set_size(width, 40);
    }
    let body = Group::new([
        Text::styled("could not read config", Style::parse("bold").unwrap_or_default()),
        Text::plain(""),
        Text::styled("Caused by:", Style::parse("dim").unwrap_or_default()),
        Text::plain("    0: No such file or directory (os error 2)"),
        Text::plain("    1: path: /etc/hud/config.toml"),
        Text::plain(""),
        Text::styled(
            "hint: run again with --verbose for details",
            Style::parse("cyan").unwrap_or_default(),
        ),
    ]);
    let panel = Panel::new(Box::new(body))
        .with_title("Error")
        .with_border_style(Style::parse("red").unwrap_or_default());
    console.print(&panel, None, None, None, false, "").unwrap();
}
