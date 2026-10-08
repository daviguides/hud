"""Differential test vectors: random styles and markup rendered by the pinned Rich.

The corpus (cases/correctness.jsonl) is the gate; these vectors are a development oracle that
reaches far past it: every color name, every palette index, thousands of RGB values through the
256-color and 16-color downgrade, and random markup and styled text with wrapping, tabs and
justification. The Rust test `crates/hud/tests/oracle.rs` reads them. Deterministic (seeded).

Rich caches a Style's escape codes on first use whatever the color system, so every markup case
renders in a fresh process, and the style vectors render each input in one fresh process with the
caches cleared between color systems (verified against fully fresh processes on a sample).
"""

import json
import random
import sys
from multiprocessing import Pool
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

ROOT = HERE.parents[1]
OUT = ROOT / "crates" / "hud" / "tests" / "fixtures"
SEED = 20261008

ATTRS = ["bold", "b", "dim", "d", "italic", "i", "underline", "u", "blink", "blink2", "reverse", "r",
         "conceal", "c", "strike", "s", "underline2", "uu", "frame", "encircle", "overline", "o"]
NAMED = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white", "bright_black", "bright_red",
         "bright_green", "bright_yellow", "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
         "grey50", "gray37", "navy_blue", "dark_orange", "deep_pink4", "default", "orange1", "purple"]
ASCII_WORDS = ["alpha", "beta", "gamma", "delta", "lexer", "parser", "worker", "x", "ok", "failed", "retry",
               "supercalifragilisticexpialidocious", "a", "bb", "ccc", "deploy", "12.5s", "[x]", "a-b", "end."]
UNICODE_WORDS = ["你好世界", "日本語", "한국어", "é", "é", "ｆｕｌｌ", "😀", "👍🏽", "🇧🇷", "naïve", "café", "😀😀😀"]
BAD_STYLES = ["nonsense", "bold nonsense", "color(300)", "#fff", "rgb(1,2)", "on", "not", "red on", "not red",
              "link", "bold,red", "x y z"]


def color_word(rng):
    kind = rng.random()
    if kind < 0.45:
        return rng.choice(NAMED)
    if kind < 0.6:
        return f"color({rng.randrange(256)})"
    if kind < 0.85:
        return "#%06x" % rng.randrange(1 << 24)
    return "rgb(%d,%d,%d)" % (rng.randrange(256), rng.randrange(256), rng.randrange(256))


def style_string(rng, allow_bad=True):
    if allow_bad and rng.random() < 0.08:
        return rng.choice(BAD_STYLES)
    words = []
    for _ in range(rng.randrange(1, 5)):
        r = rng.random()
        if r < 0.45:
            words.append(rng.choice(ATTRS))
        elif r < 0.55:
            words.append("not " + rng.choice(ATTRS))
        elif r < 0.8:
            words.append(color_word(rng))
        else:
            words.append("on " + color_word(rng))
    rng.shuffle(words) if rng.random() < 0.3 else None
    return " ".join(words)


def style_inputs(rng):
    from rich.color import ANSI_COLOR_NAMES

    inputs = []
    for name in sorted(ANSI_COLOR_NAMES):
        inputs += [name, "on " + name, name.upper(), f"bold {name} on {name}"]
    inputs += [f"color({n})" for n in range(256)]
    inputs += [f"on color({n})" for n in range(256)]
    edges = [0, 1, 46, 47, 48, 94, 95, 96, 115, 116, 135, 136, 154, 155, 175, 215, 216, 254, 255]
    for r in edges:
        for g in (0, 95, 135, 255):
            for b in (0, 47, 155, 255):
                inputs.append("#%02x%02x%02x" % (r, g, b))
    for v in range(0, 256, 3):
        inputs.append("#%02x%02x%02x" % (v, v, v))
        inputs.append("#%02x%02x%02x" % (v, v, min(255, v + 12)))
    inputs += ["#%06x" % rng.randrange(1 << 24) for _ in range(2500)]
    inputs += ["rgb(%d, %d,%d)" % (rng.randrange(256), rng.randrange(256), rng.randrange(256)) for _ in range(300)]
    inputs += [style_string(rng) for _ in range(900)]
    inputs += ATTRS + ["not " + a for a in ATTRS] + ["", "none", "  ", "BOLD", "Bold Red", "rgb(1,2,3,4)",
                                                       "rgb(256,0,0)", "color(256)", "#GGGGGG", "#12345", "#1234567"]
    inputs += BAD_STYLES
    seen, unique = set(), []
    for item in inputs:
        if item not in seen:
            seen.add(item)
            unique.append(item)
    return unique


def style_vector(item):
    from rich.color import ColorSystem
    from rich.style import Style

    systems = {"standard": ColorSystem.STANDARD, "256": ColorSystem.EIGHT_BIT, "truecolor": ColorSystem.TRUECOLOR}
    try:
        Style.parse(item)
    except Exception:
        return {"input": item, "ok": False}
    out = {"input": item, "ok": True, "ansi": {}}
    for name, system in systems.items():
        Style.parse.cache_clear()
        Style._add.cache_clear()
        out["ansi"][name] = Style.parse(item).render("X", color_system=system)
    Style.parse.cache_clear()
    out["canonical"] = str(Style.parse(item))
    return out


def style_fresh(item_system):
    item, name = item_system
    from rich.color import ColorSystem
    from rich.style import Style

    system = {"standard": ColorSystem.STANDARD, "256": ColorSystem.EIGHT_BIT, "truecolor": ColorSystem.TRUECOLOR}[name]
    return Style.parse(item).render("X", color_system=system)


def markup_text(rng, unicode_words):
    pool = ASCII_WORDS + (UNICODE_WORDS if unicode_words else [])
    out, open_tags = [], []
    for _ in range(rng.randrange(1, 14)):
        r = rng.random()
        if r < 0.45:
            out.append(rng.choice(pool))
        elif r < 0.6:
            out.append(rng.choice([" ", " ", "  ", "   ", "\t", "\n"]))
        elif r < 0.75:
            style = style_string(rng) if rng.random() < 0.8 else rng.choice(["BOLD", "Bold", "x", "link=https://x.org"])
            out.append(f"[{style}]")
            open_tags.append(style)
        elif r < 0.9:
            if open_tags and rng.random() < 0.85:
                style = open_tags.pop(rng.randrange(len(open_tags))) if rng.random() < 0.3 else open_tags.pop()
                out.append("[/]" if rng.random() < 0.5 else f"[/{style}]")
            else:
                out.append(rng.choice(["[/]", "[/bold]", "[/nope]"]))
        elif r < 0.95:
            out.append(rng.choice(["\\[", "\\\\", "\\[bold]", "\\\\[bold]", "[1]", "[]", "[ ]", "a\\[1]"]))
        else:
            out.append(rng.choice(pool) + " ")
    return "".join(out)


def make_markup_case(rng, index, unicode_words):
    return {
        "id": f"{'mu' if unicode_words else 'ma'}-{index:04d}",
        "kind": "markup",
        "markup": markup_text(rng, unicode_words),
        "width": rng.choice([5, 8, 10, 12, 20, 30, 40, 60, 80]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
        "justify": rng.choice(["default"] * 6 + ["left", "center", "right", "full"]),
        "overflow": rng.choice(["fold"] * 6 + ["crop", "ellipsis", "ignore"]),
        "no_wrap": rng.random() < 0.1,
        "tab_size": rng.choice([8] * 4 + [2, 4, 5]),
        "end": rng.choice(["\n"] * 6 + ["", " |", "\n\n"]),
    }


def make_styled_case(rng, index, unicode_words):
    pool = ASCII_WORDS + (UNICODE_WORDS if unicode_words else [])
    plain = "".join(rng.choice(pool) + rng.choice([" ", "  ", "\t", ""]) for _ in range(rng.randrange(1, 9)))
    return {
        "id": f"{'su' if unicode_words else 'sa'}-{index:04d}",
        "kind": "styled",
        "plain": plain,
        "style": style_string(rng, allow_bad=False),
        "width": rng.choice([8, 12, 20, 40, 80]),
        "color_system": rng.choice(["truecolor", "256", "standard", "none"]),
        "justify": rng.choice(["default"] * 5 + ["left", "center", "right", "full"]),
        "overflow": "fold",
        "no_wrap": False,
        "tab_size": 8,
        "end": "\n",
    }


def render_case(case):
    from rich.console import Group
    from rich.text import Text

    from render_reference import make_console

    console = make_console(case["width"], case["color_system"])
    try:
        if case["kind"] == "markup":
            text = Text.from_markup(case["markup"])
        else:
            text = Text(case["plain"], style=case["style"])
        # Console.print joins plain Text arguments into a new Text (dropping their own justify,
        # overflow and no_wrap) and wraps `justify=` in an Align of the whole block. A Text inside
        # a Group is rendered as a renderable of its own, which is what hud's Text means.
        text.justify = None if case["justify"] == "default" else case["justify"]
        text.overflow = case["overflow"]
        text.no_wrap = case["no_wrap"]
        text.tab_size = case["tab_size"]
        text.end = case["end"]
        console.print(Group(text))
    except Exception as error:
        return {**case, "error": type(error).__name__}
    return {**case, "ansi": console.file.getvalue()}


def write(name, rows):
    path = OUT / name
    with path.open("w") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"{name}: {len(rows)} rows", file=sys.stderr)


def main():
    rng = random.Random(SEED)
    OUT.mkdir(parents=True, exist_ok=True)
    inputs = style_inputs(rng)
    ascii_markup = [make_markup_case(rng, i, False) for i in range(1800)]
    unicode_markup = [make_markup_case(rng, i, True) for i in range(700)]
    ascii_styled = [make_styled_case(rng, i, False) for i in range(500)]
    unicode_styled = [make_styled_case(rng, i, True) for i in range(200)]
    with Pool(8, maxtasksperchild=1) as pool:
        styles = pool.map(style_vector, inputs, chunksize=1)
        sample = rng.sample([s for s in styles if s["ok"]], 150)
        fresh_jobs = [(s["input"], name) for s in sample for name in ("standard", "256", "truecolor")]
        fresh = pool.map(style_fresh, fresh_jobs, chunksize=1)
        for (item, name), got in zip(fresh_jobs, fresh):
            want = next(s for s in sample if s["input"] == item)["ansi"][name]
            assert got == want, f"cache contamination for {item!r} {name}: {got!r} != {want!r}"
        write("style.jsonl", styles)
        write("markup_ascii.jsonl", pool.map(render_case, ascii_markup + ascii_styled, chunksize=1))
        write("markup_unicode.jsonl", pool.map(render_case, unicode_markup + unicode_styled, chunksize=1))
    import importlib.metadata as md

    (OUT / "manifest.json").write_text(json.dumps({
        "rich": md.version("rich"), "seed": SEED, "generator": "bench/scripts/gen_oracle_vectors.py",
        "note": "Differential oracle vectors, not the gate. Regenerate with bench/.venv/bin/python.",
    }, indent=1) + "\n")


if __name__ == "__main__":
    main()
