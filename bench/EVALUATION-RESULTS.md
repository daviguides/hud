# hud: first full evaluation, results

Candidate: `hud` workspace, commit `d033748` (v0.6 part A plus the runner fixes of changelog 28 to 32), Rust 1.94.0 (`rustc 4a4ef493e 2026-03-02`). Reference: Python Rich 15.0.0. Corpus: bench corpus version of `bench/CHANGELOG.md` entry 27. Thresholds and decision rules: `~/work/projects/daviguides/hud/foundation/evaluation.md`, untouched (Amendments: none). Method for non-GO items: `~/work/projects/daviguides/hud/foundation/no-go-investigation.md`.

Machine: Apple M5 Pro, 18 cores, macOS (Darwin 25.6), shared with a live desktop (Chrome, a VM, another project's Python job). Raw outputs: `pilot/hud/`, `pilot/<candidate>/speed/`, `pilot/hud/dx/` (runs, report, transcripts and solutions of all 64 runs).

## Verdict

| Entry | Verdict | Rule | Why |
|---|---|---|---|
| **Engine: own engine as a candidate** | **GO** | Engine rule 3 (passes gates 1 and 2, speed thresholds met) | Correctness 210/210, assertiveness all four gates, S1 to S4 thresholds hold with their CI bounds, measured sequentially with no concurrent build or benchmark arm |
| **API: hud's own API** | **NO-GO** on one threshold, all others hold | API rule 2 (own API stays; the failing metric is name parity) | Name parity by recognizable signature 26/40 = 65.0% against 70% (72.5% by name only). First-try 40/40 = 100%, LOC, friction, adoption and docs hold |

Rules 1 to 6 of "Pilot validity" held for the rows above (same conditions, same work, the pilot exercised what it measures, harness self-tests green, every excluded run classified, minimum samples met), so the NO-GO is valid. The NO-GO is on one metric and is investigated below as a starting point, not as a conclusion.

## Axis 1: correctness (gate): passes

| Metric | Result | Threshold |
|---|---|---|
| Byte-identical over the claimed features | 210/210 = 100% (style 30, markup 30, table 30, panel 30, tree 30, progress 30, error 30); `table_unicode` 12/12 | at least 98% |
| Style and markup | 60/60 = 100% | 100% |
| Fuzz | 80 000 inputs per feature on nine features, 0 panics, 0 violated properties | zero panics, at least 1000 per feature |
| Unsupported | none counted as passing | never counted |

`documented_deviation` entries are in `DEVIATIONS.md` (D-001 to D-043); none is used to reach the numbers above, which are over all 210 goldens. Differences found by the differential oracle against Rich are explained there and are outside the 210.

## Axis 2: assertiveness (gate): passes

| Metric | Result | Threshold |
|---|---|---|
| Cell width agreement with Rich | 496/500 = 99.2% (13 contested strings resolved by hand: 13/13) | at least 99% |
| Grapheme splits in fold and truncate | 0 over 20 000 fold and 20 000 truncate cases | zero |
| Misaligned table rows | 0 of 97 | zero |
| Capability matrix | 40/40 | 100% |

The 4 width misses are the ZWJ-between-letters case D-001, where hud follows UAX #29 (UCD 17.0.0 `GraphemeBreakTest.txt` line 752) and Rich and `wcwidth` do not.

## Axis 3: speed: passes

`pilot.py timed` for hud, the five pilot candidates and Python Rich, one at a time, 30 iterations per workload, 5 warm-ups discarded; every hud workload output EQUAL to the golden (S3: 1000 of 1000 frames). No `cargo` or `rustc` process in any row, 0 lines `PROCEEDED UNDER LOAD` in `pilot/speed_conditions.log`. Reference for S2 to S4: the fastest existing Rust candidate whose output is verified equal for that workload (no pilot candidate passed both gates, `PILOT-RESULTS.md`).

| Workload | hud median | Python Rich | hud vs Python (95% CI) | best verified existing | hud vs best (95% CI) | Threshold | Result |
|---|---|---|---|---|---|---|---|
| S1 first byte | 2.989 ms | 37.86 ms | 0.079 [0.077, 0.084] | rs-rich 2.96 ms | 1.010 [0.983, 1.078] | at most 0.10x Python; no regression above 25% | pass |
| S2 10 000-row table | 31.643 ms | 1608.81 ms | 0.020 [0.019, 0.020] | rich_rust 133.73 ms | 0.237 [0.231, 0.242] | at most 0.80x best, CI upper at most 1.00 | pass |
| S3 100 000 progress updates | 2.175 ms | 1051.00 ms | 0.002 [0.002, 0.002] | composed 11.12 ms | 0.196 [0.191, 0.199] | same | pass |
| S4 1 000 markup lines | 2.529 ms | 59.98 ms | 0.042 [0.041, 0.043] | rs-rich 8.72 ms | 0.290 [0.283, 0.300] | same | pass |

Computed by `scripts/hud_eval.py speed` (`pilot/hud_speed.json`). Conditions (`pilot/speed_conditions_v06b_start.txt`, `pilot/speed_conditions.log`, `pilot/idle_poll.log`): the load average was never below 2.27 in 56 polling samples over about 20 minutes (maximum 5.29, mean 3.25) and averaged 3.4 across the 70 recorded row boundaries of the timed pass (maximum 9.05, during the adoption builds that precede the timings), with 92% of the CPU idle on 18 cores. A quieter window does not exist on this desktop; the driver's own limit (load at most 4.0 and no build running) held at every timing row. The thinnest margin is S1 at 0.079 against 0.10 (CI upper 0.084). The same-session ratio against Python Rich was 0.064 in the v0.3 record taken at lower load, so load moves S1 by a quarter and still leaves it under the threshold. S1 against rs-rich is 1.01: the cold start of both is the same process-start floor.

### Re-measurement of the pilot candidates in the same session

`pilot.py timed` re-measured every pilot candidate under the conditions of the hud rows, so the ratios compare arms of one session. Absolute times ran about 1.3 to 1.5x higher than in the pilot session (a busier desktop), while the ratios against Python Rich reproduced: S1 rich_rust 0.933 to 0.910, rs-rich 0.087 to 0.078; S2 rich_rust 0.084 to 0.083, rs-rich 0.126 to 0.125; S3 composed 0.011 to 0.011, rich-rs 4.15 to 3.88, rs-rich 0.130 to 0.138, richrs 0.012 to 0.013; S4 rich_rust 0.298 to 0.334, rs-rich 0.135 to 0.145 (one outlier: S3 rich_rust 0.388 to 0.256). The speed thresholds are read on same-session ratios, so the load level moves the absolute medians and leaves the verdict where it was. The earlier tables are kept as written (`PILOT-RESULTS.md`, `pilot/tables.md`, `pilot/summary.json`); this session's are in `pilot/tables_v06b.md` and `pilot/summary_v06b.json`; the raw `pilot/<candidate>/speed/` files hold this session, the previous ones are in git history.

## Axis 4: DX

### First-try success (real agents)

`dx_runner.py run --execute`: fresh agent per run (`--safe-mode`, tools Read, Grep, Glob), run directory outside any checkout holding a copy of the docs mirror and the README, one attempt, compile and golden check, transcript audit.

| Model | Role | Runs | First-try | 95% CI over tasks | Excluded | Reading |
|---|---|---|---|---|---|---|
| `claude-sonnet-5-5` | primary, the gate | 8 tasks x 5 = 40 | **40/40 = 100%** | [1.00, 1.00] | 0 harness errors, 0 protocol violations | pass (at least 80%) |
| `claude-haiku-5-5` | stress, reported only | 8 tasks x 3 = 24 | 24/24 = 100% | [1.00, 1.00] | 0, 0 | informs v0.7, no threshold |

Every task passed 5/5 and 3/3. Cost measured by the runner: primary 1.35M input and 32.9k output tokens, US$5.57; stress 4.51M input and 156k output tokens, US$0.53 (the pre-run estimate of 15.2M and 9.1M input tokens counted every turn without prompt caching). The run directory, transcripts and solutions are in `pilot/hud/dx/evidence.tar.gz`; `runs.jsonl` and `report.json` next to it.

Validity of the 100% (protocol step 1, below): no run solved a task by copying the README (the closest solution to any README code block has similarity 0.47; the median per task is 0.14 to 0.46), the agents read the docs (primary: 3.3 doc pages and 1 README read per run over 6.8 assistant turns; stress: 9.9 pages over 20.1 turns), and the literal-output check rejects solutions that print the target text. The 100% therefore measures docs plus API, with the README as one of the two inputs the protocol defines.

### Remaining metrics

| Metric | Result | Threshold | Result |
|---|---|---|---|
| LOC median, agent solutions (primary) | 11.0 (t01 15, t02 6, t03 24, t04 14, t05 8, t06 5, t07 32, t08 3); the hand-written solutions also give 11 | at most 1.5x Python Rich's 12 (18), at most 0.80x rs-rich's 15 (12) | pass: 0.92x and 0.73x. The rs-rich 15 comes from a hand-written solution, not from an agent run, so the second ratio is not paired |
| Name parity | **by name 29/40 = 72.5%; by recognizable signature 26/40 = 65.0%** | at least 70% | **fails by signature** (see the investigation) |
| API friction | 285 public functions, 0 with more than 3 positional parameters or `Option` meant as `None` | zero | pass |
| Adoption (hello table) | +2.31 s, +218 336 bytes, 6 transitive dependencies | 15 s, 1.5 MB, 60 | pass (targets 3 s, 400 KB, 10: pass) |
| Docs | 129/129 items documented, 29/29 public types with an example, 37 doctests passed, 0 ignored (`hud-width`: 16/16, 8 doctests) | 100% and 100% | pass |

Not run: the paired first-try comparison against the best existing candidate (evaluation.md, DX thresholds): the runner supports only hud, and API rule 1 cannot hold for any pilot candidate because each fails another DX threshold (`PILOT-RESULTS.md`). The absolute threshold (at least 80%) is met.

## Investigation of the non-GO item: name parity (API)

Method: `foundation/no-go-investigation.md`. Result to explain: 26 of 40 names recognizable against a threshold of 28.

### 1. Instrument validation

| Check | Finding | Class |
|---|---|---|
| Same count in every arm | The matcher in `api_surface.py` resolved a dotted name `a.b` only as member `b` of `a`. `markup.escape` is a Python module function; hud exposes it as `hud::escape` (`pub use services::markup::escape`) and the matcher called it missing. | measurement |
| Fix, applied uniformly | A lowercase prefix is a Python module: the leaf now also matches a crate-root item. Re-run for all six candidates (`pilot/<c>/api_parity_v2.json`): hud 28 to 29; rich_rust 27, rs-rich 35, richrs 27, rich-rs 38, composed 5, all unchanged. Logged as changelog 30; thresholds untouched. | measurement |
| Arms differ in one thing | Existence by name is computed the same way for all six arms. The signature review was done for hud only, because every pilot candidate already fails another DX threshold; it is a review of the 29 existing names against a written criterion (`spec/name_parity_signatures.json`), so the 40-name list and the 70% threshold are the same for everyone. Asymmetry recorded here. | setup |
| Feature exercised / harness errors | The review reads signatures from the source; no run is involved. All 64 DX runs were valid. | n/a |
| Every failing item read | The 14 items that do not count were all read (below). | n/a |
| Reviewer judgement | The criterion (same operation, same argument roles, same positional order, after the mechanical Rust adaptations) was written before the final count, after reading the signatures. It is stated in the artifact so it can be challenged; the sensitivity is below. | measurement |

### 2. Ceiling of gain

The metric is a static count over a fixed list of 40 Rich names. The ceiling is the list itself: 38 of 40 if every renamable name is added, because `box` and `box.ROUNDED` cannot have the same name in Rust (`box` is a reserved word). Moving from 26 to 28 needs two items; all names below are reachable with small changes.

### 3. Behavior, run by run

The vocabulary gap did not cost a first-try success: 0 failures in 64 runs. The agents solved the tasks through the docs without needing the missing names (transcripts: `pilot/hud/dx/evidence.tar.gz`). The metric exists for Rich users arriving from Python, a population the agent runs do not sample, so the gap is real for that user and is invisible in the first-try number.

### 4. The feature

What is missing at the point of use for a Rich user:

| Item | What a Rich user types | What hud has |
|---|---|---|
| `Console.width`, `is_terminal`, `color_system`, `no_color`, `force_terminal` | `console.width`, `console.is_terminal`, ... | the data is in `Console::capabilities()`; `width` exists only on the builder |
| `Color.parse` | `Color.parse("red")` | `Style::parse`; the `Color` enum has no parse |
| `Text.truncate`, `Text.wrap` | `text.truncate(20)` | the behavior exists (`hud_width::truncate`, `fold`, console wrapping) without those names on `Text` |
| `cell_len` | `cell_len(s)` | `hud::cell_width` |
| `box`, `box.ROUNDED` | `box.ROUNDED` | `BoxStyle`; `box` is a Rust reserved word |
| `Text.stylize` | `stylize(style, start, end)` | `stylize(range, style)`: order and type differ |
| `Progress.update` | `update(task, total=..., completed=..., description=...)` | `update(task, completed)` only |
| `Padding` | `Padding(renderable, pad)` wraps a renderable | `Padding` is a four-field spacing value |

### 5. Causal decomposition

| # | Cause | Evidence | Class | Effect on the count |
|---|---|---|---|---|
| C1 | The matcher missed a module-qualified name | `markup.escape` missing, `hud::escape` public; `api_surface.py` `parity()` | measurement | +1 existing, fixed uniformly (28 to 29) |
| C2 | Same name, different signature or role on three items | `Text::stylize(range, style)`, `Progress::update(task, completed)`, `Padding` four-field struct (`spec/name_parity_signatures.json`) | feature | 3 items exist but do not count: 29 to 26 |
| C3 | Eleven names missing | list above; five Console accessors, `Color.parse`, `Text.truncate`, `Text.wrap`, `cell_len`, `box`, `box.ROUNDED` | feature (9 renamable, 2 structural) | the headroom: 9 more names |
| C4 | Reviewer criterion | sensitivity: the 70% line holds (28/40) only if both `Text.stylize` and `Progress.update` are accepted as recognizable and `Padding` is not; with all three accepted it is 29/40 = 72.5% | measurement | the verdict depends on 2 judgements, written down |

Discarded: the task cause (the 40 names are all public in Rich 15.0.0, and no task statement mentions a name); the behavior cause (0 first-try failures, docs read in every run); the setup cause for the 64 runs (smoke test and fixes of changelog 29 preceded them). The shortfall is therefore a feature cause (C2, C3) measured by an instrument that had one false negative (C1) and a reviewer criterion that decides two items (C4). No threshold moved.

## What this changes for v0.7

Driven by C2 and C3 only; the plan is in `foundation/features.md` (v0.7). No other API change is demanded by any measurement: first-try, LOC, friction, adoption, docs and every gate hold.

## Harness and instrument changes made during this evaluation

All logged in `bench/CHANGELOG.md` (28 to 32), applied to every arm, with no threshold, gate, golden or verdict rule changed: DX runner without confirmation (28); `--safe-mode` isolation, copied docs, run directory outside a checkout, `HUD_DX_MODELS` (29); module-qualified name matching (30); `hud_eval.py` (31); the signature review artifact (32).

Self-test of the harness after the changes: `pytest -q` in `bench/` (result in the last section of this file).

## Pilot validity checklist

| Rule | Held |
|---|---|
| 1 Same conditions, no concurrent build or benchmark arm | yes (no `cargo`/`rustc` in any timing row); machine shared with an idle-ish desktop, recorded |
| 2 Same work | every hud workload EQUAL to the golden; each DX run had exactly the docs mirror, the README and the statement |
| 3 Exercises what it measures | all seven features at 30 cases; S3 replays 1000 frames; every DX agent read docs and attempted the task |
| 4 Harness self-test | pytest green (below); Rich scores 100% on its own corpus; every task has a hand-written solution passing the check |
| 5 Failure attribution | 0 harness errors, 0 protocol violations in the 64 runs; the 64 accidental calls with a broken invocation (all "Not logged in", 0 tokens) were test traffic and are not in the data |
| 6 Minimum sample | 8 x 5 DX runs, 210 cases, 500 strings, 30 speed samples |
| 7 Verdicts | GO and NO-GO issued with the rule that produced them |

## Harness self-test after all changes

`.venv/bin/python -m pytest -q` in `bench/`: 32 passed. The test that exercises `dx_runner.py run --execute` now points `HUD_DX_MODELS` at an unpinned copy of the models file, so a pinned file can never start a real run from a test.
