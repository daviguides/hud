use hud::{Color, Column, Console, Justify, Style, Table};

fn main() {
    let rows = [
        ("clap", "4.5.40", "12,400,000", "ok", "green"),
        ("serde", "1.0.219", "98,300,000", "ok", "green"),
        ("tokio", "1.46.1", "45,100,000", "ok", "green"),
        ("ratatui", "0.29.0", "2,310,000", "warn", "yellow"),
        ("syn", "2.0.104", "87,000,000", "fail", "red"),
    ];
    let mut table = Table::new()
        .title("Build report")
        .header_style(Style::new().bold().color(Color::CYAN))
        .column("Crate")
        .column(Column::new("Version").justify(Justify::Center))
        .column(Column::new("Downloads").justify(Justify::Right))
        .column("Status");
    for (name, version, downloads, status, color) in rows {
        table.add_row([name, version, downloads, &format!("[{color}]{status}[/]")]);
    }
    Console::stdout().print(&table);
}
