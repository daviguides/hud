use unicode_width::UnicodeWidthStr;

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
    let causes = [
        "No such file or directory (os error 2)",
        "path: /etc/hud/config.toml",
    ];
    let mut body = wrap("could not read config", inner);
    body.push(String::new());
    body.push("Caused by:".to_string());
    for (i, cause) in causes.iter().enumerate() {
        body.push(format!("    {i}: {cause}"));
    }
    body.push(String::new());
    body.extend(wrap("hint: run again with --verbose for details", inner));
    let title = " Error ";
    let fill = width - 2 - title.width();
    println!(
        "╭{}{}{}╮",
        "─".repeat(fill / 2),
        title,
        "─".repeat(fill - fill / 2)
    );
    for line in body {
        println!("│ {}{} │", line, " ".repeat(inner - line.width()));
    }
    println!("╰{}╯", "─".repeat(width - 2));
}
