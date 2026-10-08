use richrs::console::ColorSystem;
use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    if !console.is_terminal() || std::env::var_os("NO_COLOR").is_some() {
        console.set_color_system(ColorSystem::None);
    }

    let mut table = Table::new().title("Build report");
    table.add_column(Column::new("Crate"));
    table.add_column(Column::new("Version").justify(Justify::Center));
    table.add_column(Column::new("Downloads").justify(Justify::Right));
    table.add_column(Column::new("Status"));
    table.add_row_cells(["clap", "4.5.40", "12,400,000", "ok"]);
    table.add_row_cells(["serde", "1.0.219", "98,300,000", "ok"]);
    table.add_row_cells(["tokio", "1.46.1", "45,100,000", "ok"]);
    table.add_row_cells(["ratatui", "0.29.0", "2,310,000", "warn"]);
    table.add_row_cells(["syn", "2.0.104", "87,000,000", "fail"]);

    console.write_segments(&table.render(console.width()))
}
