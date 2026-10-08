use comfy_table::{Cell, Color, Table};
use indicatif::ProgressBar;
use owo_colors::OwoColorize;

fn main() {
    let mut table = Table::new();
    table.set_header(["name", "status"]);
    table.add_row([Cell::new("hud"), Cell::new("ok").fg(Color::Green)]);
    println!("{table}");
    let bar = ProgressBar::hidden();
    bar.inc(1);
    println!("{}", "done".green().bold());
}
