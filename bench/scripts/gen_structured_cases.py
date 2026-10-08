"""Generate corpus 4 for structured output (v0.8): cases/structured.jsonl and the expected JSON documents.

Deterministic: same script, same files. Each case is a nested renderable in the language-neutral node schema of
spec/case-schema.md (extended with `group`, `padding`, nested `progress`/`error` and a layout tree, see
spec/structured-json.md); the expected document is built HERE, in Python, from the case alone, by the rules of
spec/structured-json.md and written with `json.dumps(indent=2, ensure_ascii=False) + "\\n"`, so it is an oracle
independent of the Rust writer. No Rich golden is involved: JSON has no Rich oracle, and the plain/rich comparison
(criterion S4) compares hud with itself.

Markup strings use a closed vocabulary (known style words, `\\[` escapes) so the plain text a reader sees is known.
"""

import json
import random
from pathlib import Path

HERE = Path(__file__).resolve().parent
BENCH = HERE.parent
CASES = BENCH / "cases" / "structured.jsonl"
EXPECTED = BENCH / "golden" / "structured" / "expected.jsonl"
SEED = 401
PER_ROOT = 70
WIDTHS = [20, 40, 60, 80, 100, 120]
COLOR_SYSTEMS = ["truecolor", "256", "standard", "none"]
WORDS = [
    "api", "cli", "hud", "build", "test", "release", "width", "crate", "日本語", "한국어", "café", "naïve",
    "🚀", "🎉", "ß", "Ünï", 'say "hi"', "back\\slash", "a/b", "x_y", "100%", "{braces}", "<tag>", "tab-free",
]
STYLES = ["bold", "italic", "underline", "red", "green", "blue", "yellow", "bold red", "italic green"]
JUSTIFY = ["left", "center", "right", "full"]
TYPES = ["text", "table", "panel", "tree", "progress", "error", "columns", "layout", "group", "padding"]


def words(rng, n):
    return " ".join(rng.choice(WORDS) for _ in range(n))


def markup(rng):
    """(markup, plain): known tags and escapes only, so the plain text is known."""
    parts_m, parts_p = [], []
    for _ in range(rng.randint(1, 3)):
        word = words(rng, rng.randint(1, 3))
        kind = rng.random()
        if kind < 0.5:
            parts_m.append(word)
            parts_p.append(word)
        elif kind < 0.8:
            style = rng.choice(STYLES)
            parts_m.append(f"[{style}]{word}[/]")
            parts_p.append(word)
        else:
            parts_m.append(f"\\[{word}]")
            parts_p.append(f"[{word}]")
    sep = rng.choice([" ", " ", "\n"])
    return sep.join(parts_m), sep.join(parts_p)


def mk(rng):
    m, _ = markup(rng)
    return m


def plain_of(m):
    """Plain text of a markup string made by `markup` (closed vocabulary)."""
    out, i = [], 0
    while i < len(m):
        if m.startswith("\\[", i):
            out.append("[")
            i += 2
        elif m[i] == "[":
            j = m.index("]", i)
            i = j + 1
        else:
            out.append(m[i])
            i += 1
    return "".join(out)


def opt_markup(rng, p=0.5):
    return mk(rng) if rng.random() < p else None


def spec_node(rng, depth, kind=None):
    """A node of the case schema."""
    kinds = TYPES if depth < 3 else ["text", "table", "tree", "progress", "error"]
    kind = kind or rng.choice(kinds)
    if kind == "text":
        return {"t": "text", "markup": mk(rng)}
    if kind == "table":
        ncols = rng.randint(1, 4)
        columns = [{"header": mk(rng), "justify": rng.choice(JUSTIFY + [None])} for _ in range(ncols)]
        rows = []
        for _ in range(rng.randint(0, 4)):
            width = ncols if rng.random() < 0.85 else rng.randint(0, ncols)
            rows.append([mk(rng) for _ in range(width)])
        return {"t": "table", "title": opt_markup(rng), "caption": opt_markup(rng), "columns": columns, "rows": rows}
    if kind == "panel":
        return {"t": "panel", "title": opt_markup(rng), "subtitle": opt_markup(rng, 0.3),
                "body": spec_node(rng, depth + 1)}
    if kind == "tree":
        def tree(d):
            return {"label": mk(rng), "children": [tree(d + 1) for _ in range(rng.randint(0, 2))] if d < 3 else []}
        return {"t": "tree", "root": tree(0)}
    if kind == "progress":
        tasks = []
        for _ in range(rng.randint(0, 4)):
            total = rng.randint(0, 50)
            tasks.append({"description": words(rng, rng.randint(1, 2)), "total": total,
                          "completed": rng.randint(0, total + 3), "visible": rng.random() < 0.8})
        return {"t": "progress", "tasks": tasks}
    if kind == "error":
        junk = ["", "\u0001", "tab\there", "quote\"s", "back\\slash", "日本語", "🚀"]
        return {"t": "error", "message": words(rng, 2) + rng.choice(junk),
                "causes": [words(rng, rng.randint(1, 3)) + rng.choice(junk) for _ in range(rng.randint(0, 3))],
                "hint": words(rng, 2) if rng.random() < 0.5 else None}
    if kind == "columns":
        return {"t": "columns", "title": opt_markup(rng, 0.3),
                "items": [spec_node(rng, depth + 1) for _ in range(rng.randint(0, 4))]}
    if kind == "layout":
        def layout(d):
            node = {"name": rng.choice([None, "main", "side", "a\"b", "\u0002ctl"]), "ratio": rng.randint(1, 3),
                    "size": rng.choice([None, None, rng.randint(1, 20)]), "visible": rng.random() < 0.9,
                    "split": None, "body": None, "children": []}
            if d < 2 and rng.random() < 0.6:
                node["split"] = rng.choice(["row", "column"])
                node["children"] = [layout(d + 1) for _ in range(rng.randint(1, 3))]
            elif rng.random() < 0.85:
                node["body"] = spec_node(rng, depth + 1)
            return node
        return {"t": "layout", "root": layout(0)}
    if kind == "group":
        return {"t": "group", "items": [spec_node(rng, depth + 1) for _ in range(rng.randint(0, 3))]}
    if kind == "padding":
        return {"t": "padding", "pad": [rng.randint(0, 3) for _ in range(4)], "body": spec_node(rng, depth + 1)}
    raise AssertionError(kind)


def doc(node):
    """The expected node of the document, by spec/structured-json.md."""
    t = node["t"]
    opt = lambda m: None if m is None else plain_of(m)  # noqa: E731
    if t == "text":
        return {"type": "text", "text": plain_of(node["markup"])}
    if t == "table":
        return {"type": "table", "title": opt(node["title"]), "caption": opt(node["caption"]),
                "columns": [{"header": plain_of(c["header"]), "justify": c["justify"] or "left"} for c in node["columns"]],
                "rows": [[plain_of(c) for c in row] for row in node["rows"]]}
    if t == "panel":
        return {"type": "panel", "title": opt(node["title"]), "subtitle": opt(node["subtitle"]),
                "body": doc(node["body"])}
    if t == "tree":
        def tree(n):
            return {"type": "tree", "label": plain_of(n["label"]), "children": [tree(c) for c in n["children"]]}
        return tree(node["root"])
    if t == "progress":
        return {"type": "progress", "tasks": [
            {"description": k["description"], "completed": k["completed"], "total": k["total"],
             "finished": k["completed"] >= k["total"], "visible": k["visible"]} for k in node["tasks"]]}
    if t == "error":
        return {"type": "error", "message": node["message"], "causes": node["causes"], "hint": node["hint"]}
    if t == "columns":
        return {"type": "columns", "title": opt(node["title"]), "items": [doc(i) for i in node["items"]]}
    if t == "layout":
        def layout(n):
            return {"type": "layout", "name": n["name"], "ratio": n["ratio"], "size": n["size"],
                    "visible": n["visible"], "direction": n["split"] or "none",
                    "content": doc(n["body"]) if n["body"] is not None and not n["children"] else None,
                    "children": [layout(c) for c in n["children"]]}
        return layout(node["root"])
    if t == "group":
        return {"type": "group", "items": [doc(i) for i in node["items"]]}
    if t == "padding":
        top, right, bottom, left = node["pad"]
        return {"type": "padding", "top": top, "right": right, "bottom": bottom, "left": left,
                "content": doc(node["body"])}
    raise AssertionError(t)


def canonical(document):
    return json.dumps({"schema": "hud/1", "content": document}, indent=2, ensure_ascii=False) + "\n"


def main():
    rng = random.Random(SEED)
    cases, expected = [], []
    for i in range(PER_ROOT * len(TYPES)):
        kind = TYPES[i % len(TYPES)]
        node = spec_node(rng, 0, kind)
        case_id = f"structured-{i:03d}"
        cases.append({"id": case_id, "feature": "structured", "width": WIDTHS[i % len(WIDTHS)], "height": 24,
                      "color_system": COLOR_SYSTEMS[i % 4], "corpus": "4", "renderable": node})
        expected.append({"id": case_id, "json": canonical(doc(node))})
    CASES.parent.mkdir(parents=True, exist_ok=True)
    EXPECTED.parent.mkdir(parents=True, exist_ok=True)
    CASES.write_text("".join(json.dumps(c, ensure_ascii=False) + "\n" for c in cases))
    EXPECTED.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in expected))
    counts = {}

    def walk(n):
        if isinstance(n, dict):
            if "type" in n and isinstance(n["type"], str) and n["type"] in TYPES:
                counts[n["type"]] = counts.get(n["type"], 0) + 1
            for v in n.values():
                walk(v)
        elif isinstance(n, list):
            for v in n:
                walk(v)

    for e in expected:
        walk(json.loads(e["json"]))
    print(f"{len(cases)} cases; nodes per type: {dict(sorted(counts.items()))}")


if __name__ == "__main__":
    main()
