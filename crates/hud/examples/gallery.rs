//! Every widget in one run: markup, a table, a panel around a tree, an error report and a
//! progress display.
//!
//! ```bash
//! cargo run -p hud --example gallery
//! ```

use std::thread::sleep;
use std::time::Duration;

use hud::{Column, Console, ErrorReport, Justify, Panel, Progress, Table, Tree};

fn main() {
    let console = Console::stdout();

    hud::println!("[bold]hud[/] gallery");

    let table = Table::new()
        .title("Services")
        .column("Name")
        .column(Column::new("Requests").justify(Justify::Right))
        .column("Health")
        .row(["api", "1,204", "[green]ok[/]"])
        .row(["worker", "310", "[yellow]slow[/]"])
        .row(["mail", "0", "[red]down[/]"]);
    console.print(&table);

    let tree = Tree::new("deploy/")
        .child(
            Tree::new("staging/")
                .child("app.toml")
                .child("secrets.toml"),
        )
        .child("README.md");
    console.print(&Panel::new(tree).title("Layout"));

    let report = ErrorReport::new("deploy failed")
        .cause("health check timed out after 30s")
        .hint("run the deploy again with --verbose");
    console.print(&report);

    let progress = Progress::new();
    let upload = progress.add_task("upload", 40);
    for _ in 0..40 {
        progress.advance(&upload, 1);
        sleep(Duration::from_millis(20));
    }
    progress.finish();
}
