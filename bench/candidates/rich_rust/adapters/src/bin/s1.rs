use rich_rust::prelude::*;

fn main() {
    let builder = Console::builder().highlight(false);
    let console = if std::env::args().any(|a| a == "--explicit-size") {
        let columns = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(100);
        builder.width(columns).height(24).build()
    } else {
        builder.build()
    };
    let mut table = Table::new()
        .title("Build report")
        .header_style(Style::parse("bold cyan").unwrap_or_default())
        .with_column(Column::new("Crate"))
        .with_column(Column::new("Version").justify(JustifyMethod::Center))
        .with_column(Column::new("Downloads").justify(JustifyMethod::Right))
        .with_column(Column::new("Status"));
    table.add_row_markup(["clap", "4.5.40", "12,400,000", "[green]ok[/]"]);
    table.add_row_markup(["serde", "1.0.219", "98,300,000", "[green]ok[/]"]);
    table.add_row_markup(["tokio", "1.46.1", "45,100,000", "[green]ok[/]"]);
    table.add_row_markup(["ratatui", "0.29.0", "2,310,000", "[yellow]warn[/]"]);
    table.add_row_markup(["syn", "2.0.104", "87,000,000", "[red]fail[/]"]);
    console.print_renderable(&table);
}
