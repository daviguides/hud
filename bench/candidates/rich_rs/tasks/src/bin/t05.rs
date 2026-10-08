use rich_rs::{Console, Panel, Style, Text};

fn main() {
    let mut console = Console::new();
    if let Some(width) = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()) {
        console.set_size(width, 40);
    }
    let body = Text::from_markup(
        "[bold]could not read config[/]\n\n[dim]Caused by:[/]\n    0: No such file or directory (os error 2)\n    1: path: /etc/hud/config.toml\n\n[cyan]hint: run again with --verbose for details[/]",
        false,
    )
    .unwrap();
    let panel = Panel::new(Box::new(body))
        .with_title("Error")
        .with_border_style(Style::parse("red").unwrap_or_default());
    console.print(&panel, None, None, None, false, "").unwrap();
}
