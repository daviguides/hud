use rich_rust::prelude::*;

fn main() {
    let console = Console::new();
    let mut table = Table::new()
        .title("Build report")
        .with_column(Column::new("Crate"))
        .with_column(Column::new("Version").justify(JustifyMethod::Center))
        .with_column(Column::new("Downloads").justify(JustifyMethod::Right))
        .with_column(Column::new("Status"));
    table.add_row_cells(["clap", "4.5.40", "12,400,000", "ok"]);
    table.add_row_cells(["serde", "1.0.219", "98,300,000", "ok"]);
    table.add_row_cells(["tokio", "1.46.1", "45,100,000", "ok"]);
    table.add_row_cells(["ratatui", "0.29.0", "2,310,000", "warn"]);
    table.add_row_cells(["syn", "2.0.104", "87,000,000", "fail"]);
    console.print_renderable(&table);
}
