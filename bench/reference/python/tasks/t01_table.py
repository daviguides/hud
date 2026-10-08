from rich.console import Console
from rich.table import Table

table = Table(title="Build report")
table.add_column("Crate")
table.add_column("Version", justify="center")
table.add_column("Downloads", justify="right")
table.add_column("Status")
table.add_row("clap", "4.5.40", "12,400,000", "ok")
table.add_row("serde", "1.0.219", "98,300,000", "ok")
table.add_row("tokio", "1.46.1", "45,100,000", "ok")
table.add_row("ratatui", "0.29.0", "2,310,000", "warn")
table.add_row("syn", "2.0.104", "87,000,000", "fail")
Console().print(table)
