//! The same styled table on a terminal and in a pipe: when the output is not a terminal, the
//! escape sequences are left out and the layout stays the same.
//!
//! ```bash
//! cargo run -p hud --example pipe            # styled on a terminal
//! cargo run -p hud --example pipe | cat      # plain text
//! ```

use hud::{Color, Column, Console, Justify, Stream, Style, Table, capabilities};

fn main() {
    let caps = capabilities(Stream::Stdout);
    hud::println!(
        "stdout: terminal={} color={:?} width={}",
        caps.is_tty,
        caps.color_system,
        caps.width
    );

    let table = Table::new()
        .header_style(Style::new().bold().color(Color::CYAN))
        .column("Service")
        .column(Column::new("Latency (ms)").justify(Justify::Right))
        .row(["api", "[green]12[/]"])
        .row(["search", "[yellow]84[/]"])
        .row(["billing", "[red]410[/]"]);
    Console::stdout().print(&table);
}
