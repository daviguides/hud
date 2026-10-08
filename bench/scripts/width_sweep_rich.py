"""Dump Rich's per-code-point cell width for every Unicode scalar value.

  width_sweep_rich.py <out.tsv>

Each line: `<hex code point>\t<rich cell width>\t<general category>`. The general category comes from the pinned
UCD 17.0.0 `DerivedGeneralCategory.txt` of this repository. The dump is a measurement of Rich 15.0.0 (the
harness version) and is consumed by `tools/width-sweep`; it is not committed (it lives under results/).
"""

import sys
from pathlib import Path

from rich.cells import get_character_cell_size

BENCH = Path(__file__).resolve().parent.parent
GC = BENCH.parent / "xtask" / "data" / "ucd-17.0.0" / "DerivedGeneralCategory.txt"


def categories():
    table = {}
    for line in GC.read_text(encoding="utf-8").splitlines():
        body = line.split("#")[0].strip()
        if not body:
            continue
        cps, cat = [p.strip() for p in body.split(";")]
        lo, _, hi = cps.partition("..")
        for cp in range(int(lo, 16), int(hi or lo, 16) + 1):
            table[cp] = cat
    return table


def main():
    cats = categories()
    with open(sys.argv[1], "w", encoding="utf-8") as out:
        for cp in range(0x110000):
            if 0xD800 <= cp <= 0xDFFF:
                continue
            out.write(f"{cp:X}\t{get_character_cell_size(chr(cp))}\t{cats.get(cp, 'Cn')}\n")


if __name__ == "__main__":
    main()
