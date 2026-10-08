use owo_colors::{OwoColorize, Style, XtermColors};
use std::env;
use std::io::IsTerminal;

fn set(name: &str) -> bool {
    env::var(name).is_ok_and(|v| v != "0")
}

fn main() {
    let term = env::var("TERM").unwrap_or_default();
    let forced = set("FORCE_COLOR") || set("CLICOLOR_FORCE");
    let on = (std::io::stdout().is_terminal() || forced)
        && env::var("CLICOLOR").as_deref() != Ok("0")
        && term != "dumb";
    if !on {
        println!("x");
        return;
    }
    let mut style = Style::new().bold();
    if !set("NO_COLOR") {
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        style = if colorterm == "truecolor" || colorterm == "24bit" {
            style.truecolor(255, 136, 0)
        } else if term.contains("256") {
            style.color(XtermColors::from(208))
        } else {
            style.yellow()
        };
    }
    println!("{}", "x".style(style));
}
