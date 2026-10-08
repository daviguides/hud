use rich_rs::{Column, Console, JustifyMethod, Row, Style, Table, Text};

fn main() {
    let mut console = Console::new();
    let mut table = Table::new()
        .with_title("Build report")
        .with_title_style(Style::parse("italic").unwrap_or_default())
        .with_header_style(Style::parse("bold cyan").unwrap_or_default());
    table.add_column(Column::with_header_str("Crate"));
    table.add_column(Column::with_header_str("Version").justify(JustifyMethod::Center));
    table.add_column(Column::with_header_str("Downloads").justify(JustifyMethod::Right));
    table.add_column(Column::with_header_str("Status"));
    for (krate, version, downloads, status) in [
        ("clap", "4.5.40", "12,400,000", "ok"),
        ("serde", "1.0.219", "98,300,000", "ok"),
        ("tokio", "1.46.1", "45,100,000", "ok"),
        ("ratatui", "0.29.0", "2,310,000", "warn"),
        ("syn", "2.0.104", "87,000,000", "fail"),
    ] {
        let color = match status {
            "ok" => "green",
            "warn" => "yellow",
            _ => "red",
        };
        table.add_row(Row::new(vec![
            Box::new(Text::plain(krate)),
            Box::new(Text::plain(version)),
            Box::new(Text::plain(downloads)),
            Box::new(Text::styled(status, Style::parse(color).unwrap_or_default())),
        ]));
    }
    console.print(&table, None, None, None, false, "").unwrap();
}
