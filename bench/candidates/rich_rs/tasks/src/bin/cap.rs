use rich_rs::{Console, Text};

fn main() {
    let mut console = Console::new();
    let text = Text::from_markup("[bold #ff8800]x[/]", false).unwrap();
    console.print(&text, None, None, None, false, "\n").unwrap();
}
