"""Input tokens per task from a DX runs file: the median over the counted runs of each task, per model.

  dx_tokens.py RUNS.jsonl [RUNS2.jsonl ...]

Reported as an extra column of the evaluation, never as a gate: it was not pre-registered (the proposal is in
EVALUATION-RESULTS.md, v0.8). Excluded runs (harness_error, protocol_violation) are not counted.
"""

import json
import statistics
import sys
from collections import defaultdict


def counted(path):
    for line in open(path):
        if not line.strip():
            continue
        row = json.loads(line)
        if row["status"] in ("success", "candidate_failure") and row.get("usage"):
            yield row


def main():
    for path in sys.argv[1:]:
        per = defaultdict(list)
        for row in counted(path):
            per[(row["model"], row["task"])].append(row["usage"]["input_tokens"])
        print(f"== {path}")
        for model in sorted({m for m, _ in per}):
            tasks = sorted(t for m, t in per if m == model)
            cells = "  ".join(f"{t.split('-')[0]}:{statistics.median(per[(model, t)]):,.0f}" for t in tasks)
            allv = [v for (m, _), vs in per.items() if m == model for v in vs]
            print(f"{model}  runs {len(allv)}  median over runs {statistics.median(allv):,.0f}  max {max(allv):,}\n  {cells}")


if __name__ == "__main__":
    main()
