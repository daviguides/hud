use hud::{BarColumn, Body, Console, Layout, Panel, Progress, Table, TaskProgressColumn, TextColumn};

fn main() {
    let table = Table::new()
        .column("Crate")
        .column("Version")
        .row(["hud", "0.1.0"])
        .row(["hud-width", "0.1.0"]);
    let panel = Panel::new("All checks passed").title("Status");
    let progress = Progress::builder()
        .disable(true)
        .column(TextColumn::new("{task.description}"))
        .column(BarColumn::new().bar_width(20))
        .column(TaskProgressColumn::new())
        .build();
    progress.add_task("build", 10).advance(10);
    progress.add_task("test", 40).advance(40);
    let layout = Layout::column([
        Layout::row([Layout::new(table), Layout::new(panel)]),
        Layout::new(Body::new(progress)).size(4),
    ]);
    Console::stdout().print(&layout);
}
