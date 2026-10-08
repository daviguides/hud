use owo_colors::{OwoColorize, Style};
use std::env;
use std::io::IsTerminal;

fn main() {
    let enabled = std::io::stdout().is_terminal() || env::var_os("FORCE_COLOR").is_some();
    let no_color = env::var_os("NO_COLOR").is_some();
    let mut error = Style::new().bold();
    let mut ok = Style::new();
    if !no_color {
        error = error.red();
        ok = ok.green();
    }
    if enabled {
        println!("{} {} plain", "error".style(error), "ok".style(ok));
    } else {
        println!("error ok plain");
    }
}
