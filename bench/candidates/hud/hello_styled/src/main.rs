use hud::{Style, Text};

fn main() {
    hud::println!("[bold]build[/] [green]ok[/] in [yellow]3.2s[/]");
    let style: Style = "bold red on white".parse().unwrap();
    println!("{}", Text::styled(" FAIL ", style));
}
