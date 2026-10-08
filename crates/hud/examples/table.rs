//! A table with a title, aligned columns and styled cells.
//!
//! ```bash
//! cargo run -p hud --example table
//! ```

use hud::{Column, Console, Justify, Table};

fn main() {
    let table = Table::new()
        .title("Release checklist")
        .column("Step")
        .column(Column::new("Owner").justify(Justify::Center))
        .column(Column::new("Minutes").justify(Justify::Right))
        .column("State")
        .row(["tag the release", "ana", "3", "[green]done[/]"])
        .row(["build the artifacts", "bruno", "22", "[green]done[/]"])
        .row(["update the changelog", "ana", "10", "[yellow]review[/]"])
        .row([
            "写发布说明 (release notes)",
            "chen",
            "15",
            "[red]blocked[/]",
        ]);
    Console::stdout().print(&table);
}
