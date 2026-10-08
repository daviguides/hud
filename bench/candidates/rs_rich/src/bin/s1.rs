use rich::{Console, Justify, Table};

fn main() {
    let mut builder = Console::builder().highlight(false);
    if std::env::var_os("FORCE_COLOR").is_some() {
        builder = builder.force_terminal(true);
    }
    let mut table = Table::new().title("Build report").header_style("bold cyan");
    table.add_column("Crate");
    table.add_column_justify("Version", Justify::Center);
    table.add_column_justify("Downloads", Justify::Right);
    table.add_column("Status");
    let rows = [
        ("clap", "4.5.40", "12,400,000", "ok"),
        ("serde", "1.0.219", "98,300,000", "ok"),
        ("tokio", "1.46.1", "45,100,000", "ok"),
        ("ratatui", "0.29.0", "2,310,000", "warn"),
        ("syn", "2.0.104", "87,000,000", "fail"),
    ];
    for (name, version, downloads, status) in rows {
        let color = match status {
            "ok" => "green",
            "warn" => "yellow",
            _ => "red",
        };
        table.add_row(&[name, version, downloads, &format!("[{color}]{status}[/]")]);
    }
    builder.build().print(&table);
}
