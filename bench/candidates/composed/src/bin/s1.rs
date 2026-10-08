use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentLineStyle, LineStyle, Table, TableStyle,
};
use owo_colors::{OwoColorize, Stream};

fn main() {
    let style = TableStyle::new()
        .top_border(LineStyle::new('┏', '━', '┳', '┓'))
        .header_lines(ContentLineStyle::new('┃', '┃', '┃'))
        .header_separator(LineStyle::new('┡', '━', '╇', '┩'))
        .content_lines(ContentLineStyle::new('│', '│', '│'))
        .bottom_border(LineStyle::new('└', '─', '┴', '┘'));
    let mut table = Table::new();
    table.load_style(style);
    if supports_color::on(supports_color::Stream::Stdout).is_some() {
        table.enforce_styling();
    }
    table.set_header(
        ["Crate", "Version", "Downloads", "Status"]
            .map(|h| Cell::new(h).add_attribute(Attribute::Bold).fg(Color::Cyan)),
    );
    let rows = [
        ["clap", "4.5.40", "12,400,000", "ok"],
        ["serde", "1.0.219", "98,300,000", "ok"],
        ["tokio", "1.46.1", "45,100,000", "ok"],
        ["ratatui", "0.29.0", "2,310,000", "warn"],
        ["syn", "2.0.104", "87,000,000", "fail"],
    ];
    let w = rows
        .iter()
        .map(|r| r[1].len())
        .max()
        .unwrap()
        .max("Version".len());
    for [name, version, downloads, status] in rows {
        let color = match status {
            "ok" => Color::Green,
            "warn" => Color::Yellow,
            _ => Color::Red,
        };
        let version = format!("{version:^w$}");
        table.add_row([
            Cell::new(name),
            Cell::new(version),
            Cell::new(downloads),
            Cell::new(status).fg(color),
        ]);
    }
    table
        .column_mut(2)
        .unwrap()
        .set_cell_alignment(CellAlignment::Right);
    let lines: Vec<String> = table.lines().collect();
    let width = lines[0].chars().count();
    println!(
        "{}",
        format!("{:^width$}", "Build report").if_supports_color(Stream::Stdout, |t| t.italic())
    );
    for line in lines {
        println!("{line}");
    }
}
