use hud::{Column, Console, Justify, Table};

fn main() {
    let table = Table::new()
        .title("Build report")
        .column("Crate")
        .column(Column::new("Downloads").justify(Justify::Right))
        .row(["clap", "12,400,000"])
        .row(["serde", "[green]98,300,000[/]"]);
    Console::stdout().print(&table);
}
