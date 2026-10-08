#!/usr/bin/env bash
# Regenerate every fixture from the declarative sources. Deterministic: a clean tree must show no diff.
set -euo pipefail
cd "$(dirname "$0")/.."
uv sync --quiet
PY=.venv/bin/python
$PY scripts/gen_cases.py
$PY scripts/gen_width_corpus.py
$PY scripts/gen_speed_inputs.py
rm -rf golden/correctness golden/table_unicode
$PY scripts/render_reference.py cases/correctness.jsonl golden/correctness
$PY scripts/render_reference.py cases/table_unicode.jsonl golden/table_unicode
$PY scripts/width_ref.py
$PY scripts/capability.py write-matrix
mkdir -p golden/capability
$PY scripts/capability.py check --out golden/capability/rich_cross_check.json -- $PY reference/python/cap.py || true
$PY scripts/tasks.py make-golden
$PY scripts/gen_task_specs.py
$PY scripts/loc.py --python-baseline
$PY scripts/speed.py make-golden
{
  echo "rich=$($PY -c 'from importlib.metadata import version as v; print(v("rich"))')"
  echo "python=$($PY --version | cut -d' ' -f2)"
  echo "wcwidth=$($PY -c 'from importlib.metadata import version as v; print(v("wcwidth"))')"
  echo "regex=$($PY -c 'from importlib.metadata import version as v; print(v("regex"))')"
  echo "pyte=$($PY -c 'from importlib.metadata import version as v; print(v("pyte"))')"
} > /tmp/hud_bench_versions.txt
$PY - <<'PYEOF'
import json, pathlib
v = dict(l.strip().split("=", 1) for l in open("/tmp/hud_bench_versions.txt"))
manifest = {
    "corpus_version": "1",
    "reference": {"rich": v["rich"], "python": v["python"], "wcwidth": v["wcwidth"], "regex": v["regex"], "pyte": v["pyte"]},
    "regenerate": "bash scripts/regen.sh  (uv pins rich==15.0.0 in pyproject.toml; uv.lock pins the rest)",
    "counts": {
        "correctness_cases": sum(1 for _ in open("cases/correctness.jsonl")),
        "unicode_table_cases": sum(1 for _ in open("cases/table_unicode.jsonl")),
        "width_strings": sum(1 for _ in open("cases/width_corpus.jsonl")),
        "capability_cells": len(json.load(open("cases/capability_matrix.json"))),
        "dx_tasks": len(json.load(open("spec/tasks.json"))["tasks"]),
    },
}
pathlib.Path("golden/manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(json.dumps(manifest, indent=2))
PYEOF
