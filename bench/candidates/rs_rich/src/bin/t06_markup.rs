use rich::Console;

fn main() {
    let console = Console::builder().highlight(false).build();
    console
        .print_str("[bold]Deploy[/bold] [green]ok[/green] in [italic yellow]3.2s[/italic yellow]");
    console
        .print_str("[bold]outer [italic]inner [underline]deep[/underline][/italic] outer[/bold]");
    console.print_str("[bold red on white] FAIL [/] [strike]retry[/strike] in 5s");
}
