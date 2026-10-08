use rich_rs::{Column, Console, JustifyMethod, Style, Table};

fn main() {
    let mut console = Console::new();
    let mut table = Table::new()
        .with_title("Build report")
        .with_header_style(Style::parse("bold cyan").unwrap_or_default());
    table.add_column(Column::with_header_str("Crate"));
    table.add_column(Column::with_header_str("Version").justify(JustifyMethod::Center));
    table.add_column(Column::with_header_str("Downloads").justify(JustifyMethod::Right));
    table.add_column(Column::with_header_str("Status"));
    table.add_row_strs(&["clap", "4.5.40", "12,400,000", "ok"]);
    table.add_row_strs(&["serde", "1.0.219", "98,300,000", "ok"]);
    table.add_row_strs(&["tokio", "1.46.1", "45,100,000", "ok"]);
    table.add_row_strs(&["ratatui", "0.29.0", "2,310,000", "warn"]);
    table.add_row_strs(&["syn", "2.0.104", "87,000,000", "fail"]);
    console.print(&table, None, None, None, false, "").unwrap();
}
