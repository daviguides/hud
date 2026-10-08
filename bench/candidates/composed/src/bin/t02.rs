use unicode_width::UnicodeWidthStr;

const TEXT: &str = "hud renders terminal output that reads at a glance. Tables, panels, trees and progress share one width model, so wide characters such as 日本語 and emoji keep every border aligned on any terminal.";

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let current = lines.last_mut().unwrap();
        if !current.is_empty() && current.width() + 1 + word.width() > width {
            lines.push(word.to_string());
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    lines
}

fn main() {
    let width: usize = std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(80);
    let inner = width - 4;
    let title = " Notice ";
    let fill = width - 2 - title.width();
    println!(
        "╭{}{}{}╮",
        "─".repeat(fill / 2),
        title,
        "─".repeat(fill - fill / 2)
    );
    for line in wrap(TEXT, inner) {
        println!("│ {}{} │", line, " ".repeat(inner - line.width()));
    }
    println!("╰{}╯", "─".repeat(width - 2));
}
