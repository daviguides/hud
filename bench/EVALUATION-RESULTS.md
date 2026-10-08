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

---

# v0.7: the evaluation repeated on the changed API

Candidate: `hud` workspace at the commits `2d56cdc` and `68a5453` (v0.7: API hardening, merge of the Windows and adoption tracks), same corpus, thresholds, runner, models and instrument as v0.6 (`evaluation.md` is unchanged). Instrument changes of this round are changelog 33 to 38; none touches a golden, a threshold or a pass criterion. Raw records: `pilot/hud/` (v0.6 records are in git history at `56f73e8`), `pilot/hud/dx_v07/`, `pilot/hud/ab_v06_v07/`, `pilot/*/speed/` (same-session re-measure of every arm), `spec/name_parity_signatures_v07.json`.

## Verdict

| Entry | v0.6 | v0.7 | Rule |
|---|---|---|---|
| Engine: own engine | GO | **GO** | engine rule 3: gates 1 and 2 pass, S1 to S4 thresholds hold |
| API: hud's own API | NO-GO on name parity only | **GO** | API rule: first-try, LOC, name parity by name and by recognizable signature, friction, adoption and docs all meet their thresholds |

## What changed in the API, and why (causes from the v0.6 decomposition)

| Cause (v0.6) | Change | Evidence |
|---|---|---|
| C2: same name, different signature or role | `Text::stylize(style, range)` (style first, open ranges); `Progress::update(&task)` returns a `TaskUpdate` builder with `total`, `completed`, `advance`, `description`, `visible`, `refresh`; `Padding::new(renderable, pad)` is Rich's wrapper and the spacing value is `Pad` | `tests/padding.rs` (9 cases byte-identical to Rich), the Rich progress oracle now drives every event through `Progress::update` (all vectors pass), `tests/api.rs` |
| C3: missing names Rust can express | `Console::{width, is_terminal, color_system, no_color, force_terminal}`, `Color::parse`, `Text::{truncate, wrap}`, `cell_len` | `tests/api.rs`; `Text::truncate` matches Rich on the three cases checked against the pinned Rich |
| C1: measurement | fixed in v0.6 (changelog 30), unchanged | n/a |
| Not changed | `box`, `box.ROUNDED` stay misses (Rust reserved word) | maximum reachable 38/40 |

Decisions kept visible: `Text::wrap(width)` has no console argument (styles are typed, D-044), so the review counts it as NOT recognizable rather than bending the API to pass; `Progress::update` does not support Rich's `**fields` (D-045).

## Name parity (unchanged instrument)

| | v0.6 | v0.7 | Threshold |
|---|---|---|---|
| By existence of the name (matcher, changelog 30) | 29/40 = 72.5% | **38/40 = 95.0%** (missing: `box`, `box.ROUNDED`) | at least 70% |
| By recognizable signature (criterion of `name_parity_signatures.json`, review v2) | 26/40 = 65.0% | **37/40 = 92.5%** | at least 70% (target 90%) |
| Sensitivity | needed 2 contested items | 34/40 = 85.0% with every contested item rejected (`Text.stylize`, `Progress.update`, `Console.force_terminal`) | at least 70% |

The review of version 2 was written before the DX run and applies the same criterion text to all 40 names; `scripts/parity_review_check.py` confirms it covers exactly the 40 names and agrees with the matcher on existence.

## Axis 4: DX (real agents, same protocol as v0.6)

| Metric | Result | Threshold |
|---|---|---|
| First-try success, primary model `claude-sonnet-5-5`, 8 tasks x 5 | **40/40 = 100%** (95% CI over tasks [1.0, 1.0]) | at least 80% |
| Stress model `claude-haiku-5-5`, 8 tasks x 3 | 24/24 = 100% (reported, adds no threshold) | none |
| LOC median over tasks (primary) | 11 (t01 15, t02 8, t03 23, t04 14, t05 8, t06 5, t07 30, t08 3) | at most 12, and at most 0.80x rs-rich's 15 |
| API friction | 324 public functions; 0 with more than 3 positional parameters; 0 with `Option` parameters (0 padded `None`) | zero padded `None` |
| Adoption (hello table) | +2.31 s compile, +218 KB, 6 transitive crates | at most 15 s, 1.5 MB, 60 |
| Docs | 132/132 items documented, 32/32 with an example, 45 doctests pass, 0 ignored | 100% / 100% compiling doctests |
| Fuzz | 13 features x 15 000 inputs (3 seeds): 0 panics, 0 violated properties | zero panics, at least 1000 per feature |

Cost of the run: US$6.51 (Sonnet US$5.96 over 41 runs, Haiku US$0.55 over 25 runs, counting the two excluded attempts).

Exclusions, all classified and rerun as the protocol says: Sonnet `t03-progress #1` called a tool named `Rust` that does not exist (the CLI answered "No such tool available") and Haiku `t07-pipe #2` tried a `Write` tool; both are `protocol_violation`, excluded, and their reruns succeeded (`run_resume.log`). The runner documented the rerun but did not implement it (v0.6 had no exclusions), so changelog 37 adds `--resume`, which reruns exactly the runs without a counted result and keeps the excluded rows and their transcripts.

Reading the DX result honestly: first-try success was already 100% in v0.6, so the metric is at its ceiling and cannot show an improvement; it shows that the changes (and the new README and docs of the adoption track, changelog entry 36) did not regress it. The name-parity gain is a vocabulary gain for Rich users arriving from Python, which the agent runs do not sample (the same limit as in v0.6).

## Axes 1 to 3: no regression

| Axis | v0.7 | v0.6 | Threshold |
|---|---|---|---|
| Correctness, 210 goldens | 210/210 = 100% (style and markup 100%); `table_unicode` 12/12 | same | 98% (target 100%) |
| Width | 496/500 = 99.2% (the 4 misses are D-001) | same | 99% |
| Grapheme splits, fold and truncate | 0 of 20 000 and 0 | same | 0 |
| Misaligned table rows / rows wider than the terminal | 0/97, 0/704 | same | 0 |
| Capability matrix | 40/40 | same | 100% |
| Tasks t01 to t08 | 8/8 pass | same | all pass |
| S1 first byte vs Python Rich (CI) | 0.070 [0.068, 0.071] | 0.079 | at most 0.10x |
| S2 vs best existing Rust (CI) | 0.238 [0.232, 0.244] (rich_rust) | 0.237 | at most 0.80x, CI at most 1.00 |
| S3 vs best existing Rust (CI) | 0.198 [0.194, 0.203] (composed) | 0.196 | same |
| S4 vs best existing Rust (CI) | 0.311 [0.297, 0.317] (rs-rich) | 0.290 | same |

Speed conditions: one session, every arm (Python Rich, hud and the five pilot candidates) measured one at a time by `pilot.py speed S1,S2,S3,S4`, 1-minute load average between 2.75 and 4.00 at the starts, no `cargo` or `rustc` running, 0 lines `PROCEEDED UNDER LOAD`, every hud output EQUAL to its golden (S3 1 000 of 1 000 frames). The machine is a shared desktop, so this is not an idle window: the thresholds are read on same-session ratios, as in v0.6, and the guard of the driver (load at most 4.0) held at every start.

Version to version, interleaved on the same session (`pilot/hud/ab_v06_v07/`, 60 samples per arm and workload; v0.7 over v0.6): S1 0.976 [0.958, 1.007], S2 1.012 [0.989, 1.038], S3 1.034 [1.013, 1.050], S4 0.986 [0.974, 1.017]. The 3% on S3 is real (the display now skips hidden tasks, one atomic read per task per frame) and small against the 25% regression rule.

## What the evidence says about the next step

The next step comes from what was measured, not from a list: the API gate that failed in v0.6 is closed by the changes its causes asked for, and nothing else measured is below its threshold. The open items are not API gates: `Text::wrap` has no console argument (D-044), `box` and `box.ROUNDED` cannot exist in Rust, and first-try success is at its ceiling, so the DX metric will not detect a regression smaller than one failed run in 40 until the task suite is made harder. v0.8 (structured output) starts from here.

## v0.8: structured output (criteria S1 to S12, pre-registered in `foundation/structured-output-criteria.md`)

Written and committed before the implementation (changelog 43 to 46); two clarifications made before the criteria were measured are logged as 47 (a column's `justify` has no `default` value), 48 (S4 counts the cases that go through `render_as`: 282 of corpus 1, `table_unicode` and corpus 3, plus the 700 of corpus 4) and 50 (plain also drops the escape character and the C1 controls found in the data). Records in `pilot/hud/`: `structured.json`, `format_matrix.json`, `fuzz_v08.json`, `fuzz_live_layout_v08.json`, `ab_v07_v08/`, `dx_v08/`, `tasks_v08.json`, `docs_hud.json`, `adoption_hello_table.json`.

**Verdict: engine GO, API GO, all twelve criteria GO.** Nothing is NO-GO or INCONCLUSIVE, so the investigation protocol of `no-go-investigation.md` had no verdict to decompose; what it asked for before a number is trusted was done anyway (below).

| # | Criterion | Result | Gate | Verdict |
|---|---|---|---|---|
| S1 | JSON validity | corpus 4: 700 of 700 documents valid under `structured-json.schema.json` (independent `jsonschema` 4.26 validator) and 700 of 700 byte-equal to the document built in Python from the case (an oracle independent of the Rust writer); every node type appears at least 155 times (columns 161, error 221, group 161, layout 571, padding 177, panel 155, progress 204, table 232, text 214, tree 863); the schema shipped in the crate (`crates/hud/schema/hud-1.json`, constant `hud::JSON_SCHEMA`) is byte-identical to the spec's (a test) | 100% | GO |
| S2 | Determinism | 175 000 renders (700 cases x 5 widths x 50 repeats) with 0 mismatches; two processes produce the same bytes for 700 of 700 cases | 100% | GO |
| S3 | Canonical form and round trip | 700 of 700 equal `json.dumps(json.loads(doc), indent=2, ensure_ascii=False) + "\n"`; `Node::from_json` then `to_json` writes back the same bytes in 700 of 700 | 100% | GO |
| S4 | Plain | 982 of 982: corpus 4 700, corpus 1 210, `table_unicode` 12, corpus 3 `columns` and `layout` 60, each with no `ESC` byte and equal to the rich output with its escape sequences removed (colors and attributes forced on) | 100% | GO |
| S5 | Rich unchanged | correctness 210/210, `table_unicode` 12/12, corpus 3 102/102, width 496/500 = 99.2%, 0 splits in 20 000 fold and 20 000 truncate cases, capability 40/40, tasks t01 to t08 8/8 (10 of 10 runs); all as in v0.7 | zero regression | GO |
| S6 | Format selection | 30 of 30 cells (24 matrix cells and 6 override cells) through a real binary on a pseudo-terminal and a pipe | 30/30 | GO |
| S7 | Fuzz | 23 features (the 13 earlier and ten `structured_*`, one per value kind) x 15 000 inputs (3 seeds x 5 000), 0 panics, 0 violated properties; the 3 live-layout features likewise | 0 and 0, at least 1000 | GO |
| S8 | Docs | `hud` 149 of 149 items documented, 40 of 40 with an example, 61 doctests, 0 ignored; `hud-width` 16 of 16, 8 of 8, 8 doctests; the JSON shape is a table in the docs of `Node` and the schema a public constant | 100%, 0 ignored | GO |
| S9 | API and adoption | friction: 426 public functions, 0 with more than 3 positional parameters or `Option` parameters meant as `None`; name parity unchanged on the same instrument (review version 2): 38/40 by name, 37/40 by recognizable signature; adoption of the table hello +1.89 s, +252 KB, 6 dependencies (limits 15 s, 1.5 MB, 60); no `serde` | all hold | GO |
| S10 | Speed | v0.8 over v0.7, alternated in the same session (4 rounds of 30 samples per build and workload, loads 2.25 to 2.6, none under load): S1 **0.952** (95% CI 0.935 to 0.976), S2 **0.986** (0.976 to 0.998), S3 **0.961** (0.948 to 0.973), S4 **1.068** (1.052 to 1.084); JSON mode of the 10 000 row S2 input over rich mode of the same input: **0.438** (0.434 to 0.441), 8.54 ms against 19.51 ms | each CI upper bound at most 1.10; JSON at most as slow as rich | GO |
| S11 | DX first try (11 tasks) | `claude-sonnet-5-5`: **55 of 55 = 100%** (t01 to t08 40/40, t09 to t11 15/15); `claude-haiku-5-5`: **33 of 33** (t01 to t08 24/24, t09 to t11 9/9); US$9.40 in all (Sonnet 8.47, Haiku 0.93) | at least 80% on the primary model | GO |
| S12 | The format is discoverable | In all 8 t09 runs the agent searched the docs for `HUD_FORMAT` (it appears 1 to 5 times in each transcript) and none of the 8 programs reads the variable; no run read outside the mirror; the only protocol violation of the run was Haiku `t07-pipe #3` trying a `Write` tool (excluded, rerun with `--resume`, the rerun counted) | informational | read as part of S11 |

### What the instruments were checked for before the numbers were trusted

- **The oracle is independent.** The expected documents of S1 are built in Python from the case alone (`gen_structured_cases.py`); the Rust writer does not produce them.
- **The checks can fail.** With the library mutated and then restored: a layout `visible` always true is caught (656 of 700 documents equal the oracle), a three space indent in 0 of 700, an unescaped newline makes the document invalid JSON, an unescaped quote makes every structured fuzz feature fail (11 to 124 violations in 3 000 inputs), ignoring `plain` in `HUD_FORMAT` fails 3 cells of the matrix (changelog 49, 51).
- **A property found a real defect and the library was fixed, not the property** (changelog 50): the first structured fuzz run reported 126 to 679 violations per feature of "plain has an escape byte", every one an input that contained `U+001B` itself. Plain now drops the escape character and the C1 controls found in the data (D-052); `Format::Rich` is unchanged and stays what Rich writes.
- **Two arms measured under the same conditions.** The S10 builds are v0.7.0 (a worktree of the tag) and this checkout, both verified equal to the goldens for S1 to S4 before timing, alternating by round.
- **The DX suite is at the ceiling for first-try success, so success cannot show a regression.** Both models passed all 88 counted runs, including the three harder tasks added to make a regression detectable (changelog 44). The first-try rate is therefore not the metric that separates the tasks; the cost per run is: input tokens per Sonnet run are 10 015 to 33 730 for t09, 20 305 to 32 115 for t11 and **236 625 to 529 901 for t10** (Haiku 560 408 to 1 479 212), because an agent has to search the rustdoc HTML for how a progress display is placed in a layout. **Proposal for the next round, not applied now (a gate is never added after seeing the result):** register input tokens per task as a pre-registered DX metric with a per-task ceiling, so that a regression in discoverability shows up while success sits at 100%.

### What the new tasks show about the API

- **t09 (one call, three formats)**: all 8 runs wrote one `print` call on the standard output console and none reads `HUD_FORMAT` (the 4 outputs, including the JSON document, come from the library); LOC 9 in all 8 against 32 for Python Rich's reference, which has to write the JSON itself. Seven left the first column's justify unset (the document says `left` for it, changelog 47) and one set `Justify::Left`.
- **t11 (CJK and emoji in a table inside a panel)**: 8 of 8 first try, LOC 12 against 9.
- **t10 (a layout holding a table, a panel and a progress display)**: 8 of 8 first try, LOC 24 to 30 against 24. All eight runs used `Progress::builder().disable(true)` (a progress display that is only a region of a layout prints its own final frame when it is dropped otherwise: two extra lines, found writing the reference solution, changelog 53). Six of the eight placed it with `Body::new(progress)` (all 5 Sonnet runs and 1 Haiku run); the other two Haiku runs turned it into a string first (`render_to_plain` and `to_string`), which also matches here because the task runs with `NO_COLOR`. This is API friction that costs tokens and no failure; whether `Layout::new` should accept a `Progress` directly, and whether a progress display should say in its docs what it does inside a layout, is the next API question the cost metric above would track.

### Cumulative gates and cost

Every earlier gate holds (S5); the live-layout corpus (v0.9 scope, merged earlier) passes 102 of 102, its fuzz 3 x 15 000 with 0 violations, and the merged main is green on all CI jobs (`check`, `test` on Linux and macOS, `msrv` 1.85, `windows`, `windows-msrv`, `deny`, `api`, `fuzz`); the `api` job failed on three intermediate commits only, because the API snapshot was regenerated one commit after the code, and passes from `a349bdf`. Speed against the best existing Rust candidate, derived from the v0.7 same-session numbers and the S10 ratios (same method as v0.6), stays far inside the thresholds (S4, the largest ratio, 0.311 x 1.068 = 0.332 against 0.80).

Not claimed (D-049 to D-053): `Display` and `hud::println!` do not produce documents, the redraw frames of `Live` and `Progress` ignore the format, JSON has no styles (a new schema version would), the JSON writer is our own and reads only what it writes, a custom renderable without `node` is its plain text at width 80.

## Final evaluation (1.0 freeze): progress note

Done and recorded in `pilot/hud/final/` and `pilot/`: correctness on every corpus (210/210, `table_unicode` 12/12, corpus 3 102/102, corpus 5 182/182, structured 982/982), width 496/500 with 0 splits, capability 40/40, tasks t01 to t11, format matrix 30/30, fuzz 29 features x 15 000 with 0 panics, docs, harness self-tests (37 pass), adoption, name parity (38/40 by name, 37/40 by signature, instrument unchanged), speed S1 to S4 same session for hud and the five pilot candidates (loads 2.04 to 3.94, none under load), A/B v0.9 over v0.8.0, DX run A (Sonnet 55/55, Haiku 32/33). Open when this note was written: DX run B on the revised `Progress` docs (changelog 66), the final verdict section, the status lines of `features.md` and `architecture.md`.
