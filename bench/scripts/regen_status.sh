#!/usr/bin/env bash
# Regenerate the corpus 5 fixtures (Spinner, Status). Deterministic: a clean tree shows no diff.
# Separate from regen.sh and regen_live_layout.sh so no earlier fixture is touched by this track.
set -euo pipefail
cd "$(dirname "$0")/.."
uv sync --quiet
PY=.venv/bin/python
$PY scripts/gen_status_cases.py
rm -rf golden/status
nice -n 19 $PY scripts/render_status_reference.py cases/status.jsonl golden/status
