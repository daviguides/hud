from rich import box
from rich.console import Console, Group
from rich.panel import Panel
from rich.text import Text

body = Group(
    Text("could not read config", style="bold"),
    Text(""),
    Text("Caused by:", style="dim"),
    Text("    0: No such file or directory (os error 2)"),
    Text("    1: path: /etc/hud/config.toml"),
    Text(""),
    Text("hint: run again with --verbose for details", style="cyan"),
)
Console().print(Panel(body, title="Error", box=box.ROUNDED, border_style="red"))
