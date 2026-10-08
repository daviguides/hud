"""Deterministic inputs for the speed workloads (S2 table rows, S4 markup lines)."""

import random
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "cases" / "speed"
WORDS = "alpha beta gamma delta epsilon zeta theta kappa lambda sigma omega cache index parser render stream socket worker queue token route".split()
STATUS = ["ok", "warn", "fail", "skip"]


def s2():
    rng = random.Random(301)
    lines = []
    for i in range(10_000):
        name = f"{rng.choice(WORDS)}-{rng.choice(WORDS)}-{i % 997}"
        lines.append("\t".join([str(i), name, rng.choice(STATUS), f"{rng.uniform(0, 9999):.2f}",
                                " ".join(rng.choice(WORDS) for _ in range(rng.randint(2, 5)))]))
    (OUT / "s2_table.tsv").write_text("\n".join(lines) + "\n")


def s4():
    rng = random.Random(302)
    patterns = [
        "[bold]{a}[/bold] {b} [red]{c}[/red]",
        "[green]{a}[/green] and [bold blue]{b}[/bold blue]: {c}",
        "[bold]{a} [italic]{b} [underline]{c}[/underline][/italic][/bold]",
        "plain {a} {b} {c}",
        "[red]{a}[/red][green]{b}[/green][blue]{c}[/blue]",
        "[bold red]{a}[/] \\[{b}] [dim]{c}[/dim]",
        "[#ff8800]{a}[/#ff8800] [on blue]{b}[/on blue] {c}",
        "[italic cyan]{a}[/italic cyan] [strike]{b}[/strike] [bold]{c}[/bold]",
    ]
    lines = []
    for i in range(1000):
        p = rng.choice(patterns)
        lines.append(f"{i:04d} " + p.format(a=rng.choice(WORDS), b=rng.choice(WORDS), c=rng.choice(WORDS)))
    (OUT / "s4_lines.txt").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    s2()
    s4()
    print("speed inputs ->", OUT)
