import time

from rich.progress import BarColumn, MofNCompleteColumn, Progress, TaskProgressColumn, TextColumn

steps = {"fetch index": 3, "download crates": 120, "verify": 12}
columns = (
    TextColumn("{task.description}"),
    BarColumn(bar_width=30),
    TaskProgressColumn(),
    MofNCompleteColumn(),
)
with Progress(*columns) as progress:
    ids = {name: progress.add_task(name, total=total) for name, total in steps.items()}
    for step in range(120):
        for name, total in steps.items():
            if step < total:
                progress.advance(ids[name])
        time.sleep(0.005)
