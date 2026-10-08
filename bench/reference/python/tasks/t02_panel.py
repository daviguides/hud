from rich.console import Console
from rich.panel import Panel

text = (
    "hud renders terminal output that reads at a glance. Tables, panels, trees and "
    "progress share one width model, so wide characters such as 日本語 and emoji "
    "keep every border aligned on any terminal."
)
Console().print(Panel(text, title="Notice"))
