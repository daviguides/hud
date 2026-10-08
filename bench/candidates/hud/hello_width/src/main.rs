use hud_width::{cell_width, truncate};

fn main() {
    let line = "你好 hud 🇧🇷";
    println!("{} {}", truncate(line, 8), cell_width(line));
}
