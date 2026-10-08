use rich_rust::prelude::*;

fn main() {
    let console = Console::builder().width(60).height(24).build();
    let mut table = Table::new()
        .title("Users")
        .with_column(Column::new("Name"))
        .with_column(Column::new("Role").justify(JustifyMethod::Right));
    table.add_row_cells(["Alice", "Admin"]);
    table.add_row_cells(["Bob", "User"]);
    console.print_renderable(&table);
    let panel = Panel::from_text("Hello, World!").title("Greeting").width(40);
    console.print_renderable(&panel);
    let long = "hud renders terminal output that reads at a glance. Tables, panels, trees and progress share one width model.";
    console.print_renderable(&Panel::from_text(long).title("from_text"));
    console.print_renderable(&Text::new("no newline after this"));
    console.print("|<- previous line had no newline");
}
