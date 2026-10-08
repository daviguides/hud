from rich.console import Console

console = Console(highlight=False)
console.print("[bold]Deploy[/bold] [green]ok[/green] in [italic yellow]3.2s[/italic yellow]")
console.print("[bold]outer [italic]inner [underline]deep[/underline][/italic] outer[/bold]")
console.print("[bold red on white] FAIL [/] [strike]retry[/strike] in 5s")
