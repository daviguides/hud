use rich_rust::prelude::*;

fn main() {
    let console = Console::builder().highlight(false).build();
    let mut table = Table::new()
        .title("Build report")
        .header_style(Style::parse("bold cyan").unwrap_or_default())
        .with_column(Column::new("Crate"))
        .with_column(Column::new("Version").justify(JustifyMethod::Center))
        .with_column(Column::new("Downloads").justify(JustifyMethod::Right))
        .with_column(Column::new("Status"));
    let rows = [
        ["clap", "4.5.40", "12,400,000", "ok"],
        ["serde", "1.0.219", "98,300,000", "ok"],
        ["tokio", "1.46.1", "45,100,000", "ok"],
        ["ratatui", "0.29.0", "2,310,000", "warn"],
        ["syn", "2.0.104", "87,000,000", "fail"],
    ];
    for [name, version, downloads, status] in rows {
        let color = match status {
            "ok" => "green",
            "warn" => "yellow",
            _ => "red",
        };
        table.add_row_markup([
            name.to_string(),
            version.to_string(),
            downloads.to_string(),
            format!("[{color}]{status}[/]"),
        ]);
    }
    console.print_renderable(&table);
}
