use owo_colors::{OwoColorize, Style, XtermColors};
use supports_color::Stream;

fn main() {
    let Some(level) = supports_color::on(Stream::Stdout) else {
        println!("x");
        return;
    };
    let bold = Style::new().bold();
    let style = if level.has_16m {
        bold.truecolor(255, 136, 0)
    } else if level.has_256 {
        bold.color(XtermColors::from(208))
    } else {
        bold.yellow()
    };
    println!("{}", "x".style(style));
}
