use hud::{Stream, capabilities, cell_width, truncate};

fn main() {
    let caps = capabilities(Stream::Stdout);
    let line = "你好 hud 🇧🇷";
    println!("{} {}", truncate(line, usize::from(caps.width)), cell_width(line));
}
