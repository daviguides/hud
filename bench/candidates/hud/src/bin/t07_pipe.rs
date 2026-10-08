use hud::{Color, Column, Console, Justify, Style, Table};

fn main() {
    let table = Table::new()
        .title("Build report")
        .header_style(Style::new().bold().color(Color::CYAN))
        .column("Crate")
        .column(Column::new("Version").justify(Justify::Center))
        .column(Column::new("Downloads").justify(Justify::Right))
        .column("Status")
        .row(["clap", "4.5.40", "12,400,000", "[green]ok[/]"])
        .row(["serde", "1.0.219", "98,300,000", "[green]ok[/]"])
        .row(["tokio", "1.46.1", "45,100,000", "[green]ok[/]"])
        .row(["ratatui", "0.29.0", "2,310,000", "[yellow]warn[/]"])
        .row(["syn", "2.0.104", "87,000,000", "[red]fail[/]"]);
    Console::stdout().print(&table);
}
