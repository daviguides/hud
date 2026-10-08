use comfy_table::{CellAlignment, ContentLineStyle, LineStyle, Table, TableStyle};

fn main() {
    let style = TableStyle::new()
        .top_border(LineStyle::new('┏', '━', '┳', '┓'))
        .header_lines(ContentLineStyle::new('┃', '┃', '┃'))
        .header_separator(LineStyle::new('┡', '━', '╇', '┩'))
        .content_lines(ContentLineStyle::new('│', '│', '│'))
        .bottom_border(LineStyle::new('└', '─', '┴', '┘'));
    let mut table = Table::new();
    table.load_style(style);
    table.set_header(["Crate", "Version", "Downloads", "Status"]);
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
        table.add_row([name, &format!("{version:^w$}"), downloads, status]);
    }
    table
        .column_mut(2)
        .unwrap()
        .set_cell_alignment(CellAlignment::Right);
    let lines: Vec<String> = table.lines().collect();
    let width = lines[0].chars().count();
    println!("{:^width$}", "Build report");
    for line in lines {
        println!("{line}");
    }
}
