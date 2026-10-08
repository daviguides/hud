"""Differential test vectors for Spinner and Status: random cases rendered by the pinned Rich 15.0.0, one
fresh process per case.

The corpus (cases/status.jsonl) is the gate; these vectors are a development oracle that reaches past it:
every animation, odd speeds, updates in the middle, texts with markup and, for the Unicode set, wide
characters, flags and combining marks. The Rust test `crates/hud/tests/oracle_status.rs` reads them.
Deterministic (seeded) and separate from the other generators, so no existing fixture changes.

  gen_oracle_vectors_status.py [spinner|status|all] [--count N] [--unicode N]
"""

import argparse
import json
import random
import sys
from multiprocessing import Pool
from pathlib import Path

from rich._spinners import SPINNERS

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from gen_cases import COLOR_SYSTEMS, NAMES, STATUS, WIDTHS, words  # noqa: E402
from render_status_reference import render_case_isolated  # noqa: E402

ROOT = HERE.parents[1]
OUT = ROOT / "crates" / "hud" / "tests" / "fixtures"
SEED = 20261008 + 9
UNICODE_WORDS = ["你好世界", "日本語", "한국어", "é", "é", "ｆｕｌｌ", "😀", "👍🏽", "🇧🇷", "naïve", "café", "😀😀😀"]
STYLES = [None, "bold", "red", "bold red", "cyan on black", "dim italic", "underline green", "reverse", "bold yellow on blue"]
SPEEDS = [1.0, 1.0, 0.3, 0.5, 1.7, 2.0, 2.5, 3.0, 7.5]
NAMES_ALL = sorted(SPINNERS)


def text_markup(rng, unicode_):
    kind = rng.random()
    if kind < 0.1:
        return ""
    if kind < 0.45:
        out = rng.choice(NAMES) + ("-" + words(rng, 1) if rng.random() < 0.4 else "")
    elif kind < 0.7:
        out = f"[{rng.choice(['green', 'red', 'yellow', 'cyan', 'bold', 'italic', 'dim'])}]{rng.choice(STATUS)}[/] {words(rng, rng.randint(0, 3))}".strip()
    elif kind < 0.85:
        out = f"[bold]{words(rng, 1)}[/bold] {words(rng, rng.randint(1, 5))}"
    else:
        out = words(rng, rng.randint(1, 9))
    if unicode_ and rng.random() < 0.7:
        parts = out.split(" ")
        parts.insert(rng.randrange(len(parts) + 1), rng.choice(UNICODE_WORDS))
        out = " ".join(parts)
    return out


def base(feature, index, rng):
    return {"id": f"{feature}-oracle-{index:04d}", "feature": feature, "width": rng.choice(WIDTHS), "height": 24,
            "color_system": COLOR_SYSTEMS[index % 4], "corpus": "5"}


def step(rng, t, interval):
    return round(t + rng.choice([rng.uniform(0.001, 0.3), interval * rng.uniform(0.2, 5.0), rng.uniform(0.5, 4.0)]), 3)


def spinner_case(index, rng, unicode_):
    name = rng.choice(NAMES_ALL)
    interval = SPINNERS[name]["interval"] / 1000.0
    case = base("spinner", index, rng)
    t = round(rng.choice([0.0, 0.5, 1.0, 12.345, 999.999, 100000.0]), 3)
    ops = [{"op": "print", "at": t}]
    for _ in range(rng.randint(1, 5)):
        if rng.random() < 0.25:
            ops.append({"op": "update", "text": text_markup(rng, unicode_), "style": rng.choice(STYLES),
                        "speed": rng.choice([0.0, *SPEEDS])})
        t = step(rng, t, interval)
        ops.append({"op": "print", "at": t})
    case["renderable"] = {"t": "spinner", "name": name, "text": text_markup(rng, unicode_) or None,
                          "style": rng.choice(STYLES), "speed": rng.choice(SPEEDS), "ops": ops}
    return case


def status_case(index, rng, unicode_):
    name = rng.choice(NAMES_ALL)
    interval = SPINNERS[name]["interval"] / 1000.0
    case = base("status", index, rng)
    t = round(rng.choice([0.0, 1.0, 12.345, 5000.5]), 3)
    events = [{"op": "start", "at": t}]
    for _ in range(rng.randint(1, 7)):
        t = step(rng, t, interval)
        roll = rng.random()
        if roll < 0.5:
            events.append({"op": "refresh", "at": t})
        else:
            ev = {"op": "update", "at": t}
            if rng.random() < 0.6:
                ev["text"] = text_markup(rng, unicode_)
            if rng.random() < 0.35:
                ev["spinner"] = rng.choice(NAMES_ALL)
            if rng.random() < 0.4:
                ev["spinner_style"] = rng.choice(STYLES[1:])
            if rng.random() < 0.4:
                ev["speed"] = rng.choice(SPEEDS)
            events.append(ev)
    events.append({"op": "stop", "at": step(rng, t, interval)})
    case["renderable"] = {"t": "status", "text": text_markup(rng, unicode_), "spinner": name,
                          "spinner_style": rng.choice(STYLES), "speed": rng.choice(SPEEDS),
                          "terminal": rng.random() < 0.85, "events": events}
    return case


MAKERS = {"spinner": spinner_case, "status": status_case}


def render(case):
    return {"case": case, "out": render_case_isolated(case).decode("utf-8")}


def write(feature, unicode_, count):
    rng = random.Random(SEED + (1000 if unicode_ else 0) + sum(map(ord, feature)))
    cases = [MAKERS[feature](i, rng, unicode_) for i in range(count)]
    with Pool(4) as pool:
        rows = pool.map(render, cases, chunksize=8)
    name = f"{feature}_{'unicode' if unicode_ else 'ascii'}.jsonl"
    with (OUT / name).open("w") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"{len(rows)} vectors -> {name}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", nargs="?", default="all", choices=[*MAKERS, "all"])
    ap.add_argument("--count", type=int, default=1100)
    ap.add_argument("--unicode", type=int, default=300)
    args = ap.parse_args()
    features = list(MAKERS) if args.mode == "all" else [args.mode]
    for feature in features:
        write(feature, False, args.count)
        write(feature, True, args.unicode)


if __name__ == "__main__":
    main()
