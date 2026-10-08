"""Build the width reference (golden/width/width_ref.jsonl).

Two independent references (Rich cell_len, wcwidth) plus a third cluster-based
computation used only to hand-resolve disagreements. Resolutions live in
spec/width_resolutions.json (written by hand, checked here). The resolved width
is the ground truth candidates are measured against.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH, cluster_width, load_jsonl, rich_width, wcwidth_width  # noqa: E402

CORPUS = BENCH / "cases" / "width_corpus.jsonl"
RESOLUTIONS = BENCH / "spec" / "width_resolutions.json"
OUT = BENCH / "golden" / "width" / "width_ref.jsonl"


def main():
    items = load_jsonl(CORPUS)
    resolutions = json.loads(RESOLUTIONS.read_text()) if RESOLUTIONS.exists() else {"entries": {}}
    entries = resolutions["entries"]
    OUT.parent.mkdir(parents=True, exist_ok=True)
    unresolved = []
    stale = []
    with OUT.open("w") as f:
        for it in items:
            rich, wc, cl = rich_width(it["text"]), wcwidth_width(it["text"]), cluster_width(it["text"])
            rec = {"id": it["id"], "category": it["category"], "rich": rich, "wcwidth": wc}
            if rich == wc:
                rec.update(ref=rich, status="agree")
                if it["id"] in entries:
                    stale.append(it["id"])
            elif it["id"] in entries:
                e = entries[it["id"]]
                rec.update(ref=e["ref"], status="resolved", resolution=e["rule"], cluster=cl)
            else:
                unresolved.append((it["id"], rich, wc, cl))
                continue
            f.write(json.dumps(rec, ensure_ascii=False, sort_keys=True) + "\n")
    if stale:
        print("stale resolutions (references now agree):", stale)
    if unresolved:
        print("UNRESOLVED disagreements (add to spec/width_resolutions.json):")
        for u in unresolved:
            print(" ", u)
        raise SystemExit(1)
    n_res = sum(1 for k in entries)
    print(f"{len(items)} strings -> {OUT} ({n_res} hand-resolved)")


if __name__ == "__main__":
    main()
