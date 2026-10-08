use hud::{Column, Console, Justify, Table};

fn main() {
    let table = Table::new()
        .column("Name")
        .column(Column::new("Count").justify(Justify::Right))
        .row(["api", "12"])
        .row(["cli", "7"]);
    Console::stdout().print(&table);
}
