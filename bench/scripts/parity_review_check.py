"""Checks that a signature review covers exactly the names the matcher knows and agrees with it on existence.

  parity_review_check.py <review.json> <api.json>

`api.json` is the record `pilot.py static` writes (it lists the names the matcher did not find). Every name of
`spec/name_parity.json` must appear once in the review, and `exists` must equal "not in the matcher's missing
list". A recognizable item must exist. Exits 1 on any disagreement.
"""

import json
import sys
from pathlib import Path

BENCH = Path(__file__).resolve().parent.parent


def main():
    review = json.loads(Path(sys.argv[1]).read_text())
    api = json.loads(Path(sys.argv[2]).read_text())
    names = json.loads((BENCH / "spec" / "name_parity.json").read_text())["names"]
    missing = set(api["missing"])
    problems = []
    seen = [item["name"] for item in review["items"]]
    if sorted(seen) != sorted(names):
        problems.append(f"review names differ from name_parity.json: {sorted(set(seen) ^ set(names))}")
    for item in review["items"]:
        if item["exists"] != (item["name"] not in missing):
            problems.append(f"{item['name']}: review exists={item['exists']} but the matcher says missing={item['name'] in missing}")
        if item["recognizable"] and not item["exists"]:
            problems.append(f"{item['name']}: recognizable but does not exist")
    exists = sum(i["exists"] for i in review["items"])
    recognizable = sum(i["recognizable"] for i in review["items"])
    if (exists, recognizable) != (review["exists_by_name"], review["recognizable"]):
        problems.append(f"totals in the file {(review['exists_by_name'], review['recognizable'])} != counted {(exists, recognizable)}")
    print(f"review v{review['version']}: exists {exists}/{len(names)}, recognizable {recognizable}/{len(names)}")
    for p in problems:
        print("PROBLEM", p)
    raise SystemExit(1 if problems else 0)


if __name__ == "__main__":
    main()
