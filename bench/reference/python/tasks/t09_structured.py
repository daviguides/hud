import json
import os
import sys

from rich.console import Console
from rich.table import Table

fmt = os.environ.get("HUD_FORMAT", "").lower()

if fmt == "json":
    document = {
        "schema": "hud/1",
        "content": {
            "type": "table",
            "title": None,
            "caption": None,
            "columns": [
                {"header": "Name", "justify": "left"},
                {"header": "Count", "justify": "right"},
            ],
            "rows": [["api", "12"], ["cli", "7"]],
        },
    }
    sys.stdout.write(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
else:
    table = Table()
    table.add_column("Name")
    table.add_column("Count", justify="right")
    table.add_row("api", "12")
    table.add_row("cli", "7")
    if fmt == "plain":
        console = Console(force_terminal=False)
    else:
        console = Console()
    console.print(table)
