"""Find golden cases that depend on process state: re-render each case with Rich in a fresh process
and compare with the golden. A mismatch means the golden holds bytes that no stateless renderer
produces (Rich caches a Style's ANSI codes on the first color system that renders it)."""

import json
import multiprocessing as mp
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(BENCH / "scripts"))


def fresh(case_json):
    from render_reference import render_case

    case = json.loads(case_json)
    return case["id"], render_case(case)


def main():
    cases = [ln for ln in (BENCH / "cases/correctness.jsonl").read_text().splitlines() if ln.strip()]
    out = {}
    with mp.get_context("spawn").Pool(8, maxtasksperchild=1) as pool:
        for cid, data in pool.imap_unordered(fresh, cases):
            golden = (BENCH / "golden/correctness" / f"{cid}.ansi").read_bytes()
            if data != golden:
                out[cid] = {"class": "reference_quirk", "reason": "golden differs from a fresh-process Rich render (Rich caches ANSI codes per Style across color systems)"}
    Path(sys.argv[1]).write_text(json.dumps(out, indent=1, sort_keys=True) + "\n")
    print(len(out), "of", len(cases), "golden cases are process-state dependent")


if __name__ == "__main__":
    main()
