"""Generate the Spinner and Status corpus (cases/status.jsonl, corpus 5).

Deterministic: same script, same file. A separate file and a separate seed range (501...) leave
every earlier corpus as it was (see spec/status-schema.md). The clock is part of the case: every
frame is drawn at a stated time, so the animation is reproducible on both sides.
"""

import json
import random
import sys
from pathlib import Path

from rich._spinners import SPINNERS

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from gen_cases import COLOR_SYSTEMS, WIDTHS, words  # noqa: E402

OUT = HERE.parent / "cases" / "status.jsonl"
CORPUS_VERSION = "5"
STYLES = [None, "bold red", "cyan", "bold yellow on blue", "dim", "italic magenta", "underline green"]
TEXTS = [
    None,
    "Loading",
    "[bold]Installing[/bold] crates",
    "[green]ready[/] to [red]go[/]",
    "コンパイル中 日本語",
    "build 🚀 done ✔",
    "a rather long status line that is wider than the narrow consoles of this corpus and must wrap or cut",
    "tab\there",
]


def base(feature, idx, rng):
    return {
        "id": f"{feature}-{idx:03d}",
        "feature": feature,
        "width": rng.choice(WIDTHS),
        "height": 24,
        "color_system": COLOR_SYSTEMS[idx % 4],
        "corpus": CORPUS_VERSION,
    }


def clock_start(rng):
    return round(rng.choice([0.0, 1.0, 12.345, 100.0, 5000.5]), 3)


def spinner_cases():
    """Every animation, twice: printed at three clock readings, with updates in a subset."""
    rng = random.Random(501)
    cases = []
    idx = 0
    for round_no in range(2):
        for name in sorted(SPINNERS):
            c = base("spinner", idx, rng)
            interval = SPINNERS[name]["interval"] / 1000.0
            t0 = clock_start(rng)
            # readings span several frames of this animation; the second round uses odd offsets
            steps = [round(t0 + interval * rng.uniform(0.2, 3.7), 3), round(t0 + interval * rng.uniform(4.0, 40.0), 3)]
            text = rng.choice(TEXTS) if (idx % 3) else None
            speed = rng.choice([1.0, 1.0, 1.0, 0.5, 2.0, 3.5]) if idx % 4 == 0 else 1.0
            style = rng.choice(STYLES) if idx % 2 == 0 else None
            ops = [{"op": "print", "at": t0}, {"op": "print", "at": steps[0]}]
            if round_no == 1 and idx % 5 == 0:
                ops.append({"op": "update", "text": rng.choice(TEXTS[1:4]), "speed": rng.choice([0.5, 2.0, 4.0]),
                            "style": rng.choice(STYLES[1:])})
            ops.append({"op": "print", "at": steps[1]})
            c["renderable"] = {"t": "spinner", "name": name, "text": text, "style": style, "speed": speed, "ops": ops}
            cases.append(c)
            idx += 1
    return cases


def status_cases():
    """Status lifecycles: start, refresh at stated times, update, stop; interactive or not."""
    rng = random.Random(502)
    cases = []
    names = ["dots", "line", "arc", "moon", "bouncingBall", "earth", "clock", "dots12", "aesthetic", "arrow3",
             "star", "toggle5", "weather", "runner", "pong"]
    for i in range(36):
        c = base("status", i, rng)
        name = names[i % len(names)]
        interval = SPINNERS[name]["interval"] / 1000.0
        t = clock_start(rng)
        events = [{"op": "start", "at": t}]
        for _ in range(rng.randint(1, 5)):
            t = round(t + interval * rng.uniform(0.3, 5.0), 3)
            events.append({"op": "refresh", "at": t})
            roll = rng.random()
            if roll < 0.3:
                t = round(t + interval * rng.uniform(0.1, 2.0), 3)
                ev = {"op": "update", "at": t}
                if rng.random() < 0.7:
                    ev["text"] = rng.choice(TEXTS[1:])
                if rng.random() < 0.4:
                    ev["spinner"] = rng.choice(names)
                if rng.random() < 0.4:
                    ev["spinner_style"] = rng.choice(STYLES[1:])
                if rng.random() < 0.4:
                    ev["speed"] = rng.choice([0.5, 2.0, 3.0])
                events.append(ev)
        t = round(t + interval * rng.uniform(0.2, 3.0), 3)
        events.append({"op": "stop", "at": t})
        c["renderable"] = {
            "t": "status",
            "text": rng.choice(TEXTS[1:]) if i % 7 else "",
            "spinner": name,
            "spinner_style": rng.choice(STYLES) if i % 2 == 0 else None,
            "speed": rng.choice([1.0, 1.0, 2.0, 0.5]) if i % 3 == 0 else 1.0,
            "terminal": i % 6 != 5,
            "events": events,
        }
        cases.append(c)
    return cases


def main():
    cases = spinner_cases() + status_cases()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w") as f:
        for c in cases:
            f.write(json.dumps(c, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"{len(cases)} cases -> {OUT}")


if __name__ == "__main__":
    main()
