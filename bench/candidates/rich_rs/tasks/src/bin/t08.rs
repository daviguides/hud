use rich_rs::{Console, Text};

fn main() {
    let mut console = Console::new();
    let text = Text::from_markup("[bold red]error[/] [green]ok[/] plain", false).unwrap();
    console.print(&text, None, None, None, false, "\n").unwrap();
}
