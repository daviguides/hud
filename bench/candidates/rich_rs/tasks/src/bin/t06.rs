use rich_rs::{Console, Text};

fn main() {
    let mut console = Console::new();
    for line in [
        "[bold]Deploy[/bold] [green]ok[/green] in [italic yellow]3.2s[/italic yellow]",
        "[bold]outer [italic]inner [underline]deep[/underline][/italic] outer[/bold]",
        "[bold red on white] FAIL [/] [strike]retry[/strike] in 5s",
    ] {
        let text = Text::from_markup(line, false).unwrap();
        console.print(&text, None, None, None, false, "\n").unwrap();
    }
}
