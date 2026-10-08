// format-probe: prints the table of task t09 with ONE print call on the standard output console.
// `HUD_PROBE_FORMAT` (rich, plain, json), when set, is the format chosen on the console itself
// (ConsoleBuilder::format), to check that it wins over `HUD_FORMAT` (spec/format.md, criterion S6).
use hud::{Column, Console, Format, Justify, Table};

fn main() {
    let table = Table::new()
        .column("Name")
        .column(Column::new("Count").justify(Justify::Right))
        .row(["api", "12"])
        .row(["cli", "7"]);
    let console = match std::env::var("HUD_PROBE_FORMAT").ok().and_then(|v| Format::parse(&v)) {
        Some(format) => Console::builder().format(format).build(),
        None => Console::stdout(),
    };
    console.print(&table);
}
