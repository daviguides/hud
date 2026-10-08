use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    console.print("[bold]Deploy[/] [green]ok[/] in [italic yellow]3.2s[/]")?;
    console.print(
        "[bold]outer [italic]inner [underline]deep[/underline][/italic] outer[/bold]",
    )?;
    console.print("[bold red on white] FAIL [/] [strike]retry[/strike] in 5s")
}
