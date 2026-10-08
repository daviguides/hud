#!/usr/bin/env bash
# Regenerate the corpus 3 fixtures (Columns, Layout, Live). Deterministic: a clean tree shows no diff.
# Separate from regen.sh so the corpus 1 and 2 fixtures are never touched by this track.
set -euo pipefail
cd "$(dirname "$0")/.."
uv sync --quiet
PY=.venv/bin/python
$PY scripts/gen_live_layout_cases.py
rm -rf golden/live_layout
nice -n 19 $PY scripts/render_live_layout_reference.py cases/live_layout.jsonl golden/live_layout
