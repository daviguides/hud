from rich.console import Console
from rich.table import Table

table = Table(title="Build report", header_style="bold cyan")
table.add_column("Crate")
table.add_column("Version", justify="center")
table.add_column("Downloads", justify="right")
table.add_column("Status")
rows = [
    ("clap", "4.5.40", "12,400,000", "ok"),
    ("serde", "1.0.219", "98,300,000", "ok"),
    ("tokio", "1.46.1", "45,100,000", "ok"),
    ("ratatui", "0.29.0", "2,310,000", "warn"),
    ("syn", "2.0.104", "87,000,000", "fail"),
]
colors = {"ok": "green", "warn": "yellow", "fail": "red"}
for crate, version, downloads, status in rows:
    table.add_row(crate, version, downloads, f"[{colors[status]}]{status}[/]")
Console(highlight=False).print(table)
