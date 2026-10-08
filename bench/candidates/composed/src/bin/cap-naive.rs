use owo_colors::{OwoColorize, Stream, Style};

fn main() {
    println!(
        "{}",
        "x".if_supports_color(Stream::Stdout, |t| t
            .style(Style::new().bold().truecolor(255, 136, 0)))
    );
}
