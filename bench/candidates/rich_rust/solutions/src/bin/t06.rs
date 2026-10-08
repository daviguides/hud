use rich_rust::prelude::*;

fn main() {
    let console = Console::builder().highlight(false).build();
    console.print("[bold]Deploy[/bold] [green]ok[/green] in [italic yellow]3.2s[/italic yellow]");
    console.print("[bold]outer [italic]inner [underline]deep[/underline][/italic] outer[/bold]");
    console.print("[bold red on white] FAIL [/] [strike]retry[/strike] in 5s");
}
