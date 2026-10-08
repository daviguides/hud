use rich_rust::prelude::*;

fn main() {
    let console = Console::new();
    let mut table = Table::new().with_column(Column::new("Name"));
    table.add_row_cells(["hello"]);
    console.print_renderable(&table);
}
