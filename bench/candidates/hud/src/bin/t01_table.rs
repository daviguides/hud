use hud::{Column, Console, Justify, Table};

fn main() {
    let table = Table::new()
        .title("Build report")
        .column("Crate")
        .column(Column::new("Version").justify(Justify::Center))
        .column(Column::new("Downloads").justify(Justify::Right))
        .column("Status")
        .row(["clap", "4.5.40", "12,400,000", "ok"])
        .row(["serde", "1.0.219", "98,300,000", "ok"])
        .row(["tokio", "1.46.1", "45,100,000", "ok"])
        .row(["ratatui", "0.29.0", "2,310,000", "warn"])
        .row(["syn", "2.0.104", "87,000,000", "fail"]);
    Console::stdout().print(&table);
}
