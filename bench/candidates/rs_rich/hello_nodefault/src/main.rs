use rich::{Console, Table};

fn main() {
    let mut table = Table::new().title("Build report");
    table.add_column("Crate");
    table.add_column("Status");
    table.add_row(&["clap", "ok"]);
    Console::new().print(&table);
}
