use richrs::prelude::*;
use richrs::table::Row;

fn main() -> Result<()> {
    let mut console = Console::new();
    let header = Style::parse("bold cyan")?;
    let mut table = Table::new().title("Build report");
    table.add_column(Column::new("Crate").header_style(header.clone()));
    table.add_column(Column::new("Version").header_style(header.clone()).justify(Justify::Center));
    table.add_column(Column::new("Downloads").header_style(header.clone()).justify(Justify::Right));
    table.add_column(Column::new("Status").header_style(header));
    for (name, version, downloads, status, color) in [
        ("clap", "4.5.40", "12,400,000", "ok", "green"),
        ("serde", "1.0.219", "98,300,000", "ok", "green"),
        ("tokio", "1.46.1", "45,100,000", "ok", "green"),
        ("ratatui", "0.29.0", "2,310,000", "warn", "yellow"),
        ("syn", "2.0.104", "87,000,000", "fail", "red"),
    ] {
        table.add_row(Row::new([
            Text::from(name),
            Text::from(version),
            Text::from(downloads),
            Text::styled(status, Style::parse(color)?),
        ]));
    }
    console.write_segments(&table.render(console.width()))
}
