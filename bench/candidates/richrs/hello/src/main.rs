use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    let mut table = Table::new().title("hello");
    table.add_column(Column::new("Crate"));
    table.add_column(Column::new("Status"));
    table.add_row_cells(["hud", "ok"]);
    console.write_segments(&table.render(console.width()))
}
