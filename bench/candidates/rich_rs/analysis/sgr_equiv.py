"""Supplementary: compare two ANSI streams cell by cell with a minimal SGR interpreter (no cursor movement).

Usage: python3 candidates/rich_rs/analysis/sgr_equiv.py <got> <want> [--gz-want] [--visible]

Reports whether the visible text and the style of every character are equal. For speed workloads S1, S2 and S4,
whose output is plain lines, this decides if a byte difference is SGR spelling only (speed.md: documented_deviation).
"""

import gzip
import re
import sys

TOKEN = re.compile(rb"\x1b\[([0-9;]*)m|\x1b\[[0-9;?]*[A-Za-z]|([^\x1b]+)")


def cells(data: bytes):
    state = {"fg": None, "bg": None, "b": 0, "d": 0, "i": 0, "u": 0, "s": 0, "r": 0}
    out = []
    for m in TOKEN.finditer(data):
        if m.group(2) is not None:
            snap = tuple(sorted(state.items()))
            out.extend((ch, snap) for ch in m.group(2).decode("utf-8", "replace"))
            continue
        if m.group(1) is None:
            continue
        p = [int(x) if x else 0 for x in m.group(1).split(b";")] if m.group(1) else [0]
        i = 0
        while i < len(p):
            c = p[i]
            if c == 0:
                state.update(fg=None, bg=None, b=0, d=0, i=0, u=0, s=0, r=0)
            elif c == 1: state["b"] = 1
            elif c == 2: state["d"] = 1
            elif c == 3: state["i"] = 1
            elif c == 4: state["u"] = 1
            elif c == 7: state["r"] = 1
            elif c == 9: state["s"] = 1
            elif c == 22: state["b"] = state["d"] = 0
            elif c == 23: state["i"] = 0
            elif c == 24: state["u"] = 0
            elif c == 27: state["r"] = 0
            elif c == 29: state["s"] = 0
            elif 30 <= c <= 37 or 90 <= c <= 97: state["fg"] = (c,)
            elif c == 39: state["fg"] = None
            elif 40 <= c <= 47 or 100 <= c <= 107: state["bg"] = (c,)
            elif c == 49: state["bg"] = None
            elif c in (38, 48):
                key = "fg" if c == 38 else "bg"
                if p[i + 1] == 5:
                    state[key] = (c, 5, p[i + 2]); i += 2
                else:
                    state[key] = (c, 2, p[i + 2], p[i + 3], p[i + 4]); i += 4
            i += 1
    # a space or newline carries no visible style except background/reverse/underline/strike; keep it simple and exact
    return out


def visible(cell_list):
    """Spaces show only background, underline, strike and reverse; bold, italic, dim and foreground are invisible."""
    out = []
    for ch, snap in cell_list:
        if ch == " ":
            d = dict(snap)
            snap = tuple(sorted((k, d[k]) for k in ("bg", "u", "s", "r")))
        out.append((ch, snap))
    return out


def main():
    got = open(sys.argv[1], "rb").read()
    want_raw = open(sys.argv[2], "rb").read()
    want = gzip.decompress(want_raw) if "--gz-want" in sys.argv else want_raw
    a, b = cells(got), cells(want)
    if "--visible" in sys.argv:
        a, b = visible(a), visible(b)
    text_equal = [c for c, _ in a] == [c for c, _ in b]
    both = a == b
    first = next((k for k in range(min(len(a), len(b))) if a[k] != b[k]), None)
    print(f"cells got={len(a)} want={len(b)} text_equal={text_equal} style_equal_everywhere={both}" + (f" first_diff_cell={first}" if first is not None else ""))


if __name__ == "__main__":
    main()
