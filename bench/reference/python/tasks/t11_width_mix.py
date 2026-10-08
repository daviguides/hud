from rich.console import Console
from rich.panel import Panel
from rich.table import Table

table = Table("Locale", "Greeting", "Note")
table.add_row("ja", "こんにちは世界", "wide characters")
table.add_row("ko", "안녕하세요", "wide characters")
table.add_row("pt", "Olá, mundo", "one accent")
table.add_row("emoji", "🚀 launch 🎉", "two wide symbols")
Console().print(Panel(table, title="Release notes"))
