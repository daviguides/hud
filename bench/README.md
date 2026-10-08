# hud bench

Fixtures, reference renderer and harness for the hud evaluation. The thresholds and decision rules are pre-registered in `~/work/projects/daviguides/hud/foundation/evaluation.md`; how each metric is measured is in `~/work/projects/daviguides/hud/references/studies/bench-design.md`. Nothing here changes a threshold.

```bash
bash scripts/regen.sh     # regenerate every fixture; a clean tree shows no diff
uv run pytest -q          # harness self-tests (Rich scores 100% on its own corpus, mutants are caught)
```

Reference: Python Rich 15.0.0 (pinned in `pyproject.toml`, `uv.lock`), recorded in `golden/manifest.json`.

## Layout

| path | content |
|---|---|
| `spec/` | language-neutral definitions: `case-schema.md`, `width.md`, `capability.md`, `speed.md`, `tasks.json` and `tasks/<id>.md` (statements with the target embedded), `name_parity.json` |
| `cases/` | generated inputs: `correctness.jsonl` (210), `table_unicode.jsonl` (12), `width_corpus.jsonl` (500), `capability_matrix.json` (40), `speed/` |
| `golden/` | Rich output: `correctness/`, `table_unicode/`, `width/width_ref.jsonl`, `capability/`, `tasks/`, `speed/`, `manifest.json` |
| `reference/python/` | Rich solutions of the 8 DX tasks (the LOC baseline), the capability one-liner, speed adapters |
| `scripts/` | generators, comparison, checkers, timing, adoption cost, docs and API metrics |
| `results/` | raw candidate runs (git-ignored) |
| `pilot/` | committed per-candidate results of the corrected pass (`pilot/<candidate>/*.json`), `tables.md`, `summary.json`, `machine.txt`, speed conditions logs |

## What a candidate provides

Each candidate has its own crate and adapters under `candidates/<name>/` (rich_rust, rs_rich, richrs, rich_rs, composed). Results of the first pilot: [PILOT-RESULTS.md](PILOT-RESULTS.md); harness changes after the first candidate pass: [CHANGELOG.md](CHANGELOG.md).

| adapter | contract | checked by |
|---|---|---|
| `cases-runner <cases.jsonl> <outdir>` | one `<id>.ansi` per case, or an empty `<id>.unsupported`; optional `support.json`, `classifications.json`. Run on `cases/correctness.jsonl` and `cases/table_unicode.jsonl`, in a clean environment, configuring the console through the crate's public API as the schema says | `scripts/compare.py <outdir>` |
| `width-runner <outdir>` | `width.jsonl`, `fold.jsonl`, `truncate.jsonl`, `tables/` as in `spec/width.md` | `scripts/width_check.py <outdir>` |
| `cap` | prints bold `#ff8800` `x` and a newline through the default console | `scripts/capability.py check -- <cmd>` |
| `s1`, `bench` | speed protocol in `spec/speed.md` | `scripts/speed.py verify` and `time` |
| `tNN` solutions | 8 hand-written programs, one per DX task | `scripts/tasks.py check <task-id> -- <cmd>` |
| minimal `hello table` crate | for adoption cost, docs and API metrics | `adoption.py`, `docs_coverage.py`, `api_surface.py` |

## Scripts

| script | use |
|---|---|
| `gen_cases.py`, `gen_width_corpus.py`, `gen_speed_inputs.py` | seeded generators |
| `render_reference.py` | declarative case to ANSI bytes with Rich |
| `width_ref.py` | width reference from Rich, wcwidth and `spec/width_resolutions.json` |
| `compare.py` | byte-identical rate per feature, first diff, classifications |
| `width_check.py` | width agreement, grapheme splits in fold and truncate, table alignment |
| `capability.py` | 40 cells against the written expectation |
| `tasks.py`, `gen_task_specs.py`, `loc.py` | DX tasks: run, goldens, statements, LOC rule |
| `speed.py` | output verification, S1 whole-process timing, S2-S4 in-process timing, bootstrap CI, ratios |
| `adoption.py`, `docs_coverage.py`, `api_surface.py` | compile time / binary size / dependencies; rustdoc coverage and doctests; name parity and API friction |
| `parity_review_check.py` | checks that a name-parity signature review covers exactly the 40 names and agrees with the matcher |
| `pilot.py`, `pilot_report.py` | run every candidate through the same code (`static`, `timed`, `speed`, `unverified`) and build the cross-candidate tables |
| `docs_mirror.py` | docs.rs-equivalent mirror (no `src/` pages) for the DX first-try test; prompt in `spec/dx-prompt.md` |
