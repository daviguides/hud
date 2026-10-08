from rich.console import Console
from rich.layout import Layout
from rich.panel import Panel
from rich.progress import BarColumn, Progress, TaskProgressColumn, TextColumn
from rich.table import Table

console = Console()

table = Table("Crate", "Version")
table.add_row("hud", "0.1.0")
table.add_row("hud-width", "0.1.0")

panel = Panel("All checks passed", title="Status")

progress = Progress(
    TextColumn("{task.description}"),
    BarColumn(bar_width=20),
    TaskProgressColumn(),
    auto_refresh=False,
    console=console,
)
progress.add_task("build", total=10, completed=10)
progress.add_task("test", total=40, completed=40)

layout = Layout()
layout.split_column(Layout(name="top"), Layout(name="bottom", size=4))
layout["top"].split_row(Layout(table, name="left"), Layout(panel, name="right"))
layout["bottom"].update(progress)
console.print(layout)
