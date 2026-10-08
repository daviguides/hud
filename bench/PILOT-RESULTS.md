# hud pilot results

Cross-candidate results of the first full pilot, under the corrected harness (`bench/CHANGELOG.md`). Reference: Python Rich 15.0.0, corpus version 1, machine and toolchain in `pilot/machine.txt` (Apple M5 Pro, 18 logical CPUs, 48 GiB, macOS 26.7, rustc 1.94.0, cargo 1.94.0). Thresholds, gates and decision rules are those of `foundation/evaluation.md`, unchanged (Amendments: none). Every number here is computed from `pilot/<candidate>/` by `scripts/pilot_report.py`; raw outputs are in `results/` (git-ignored) and regenerate with `scripts/pilot.py`.

## What changed in the harness before this pass

Six changes, each logged with its rationale in `CHANGELOG.md` and applied to every candidate before any re-run: (1) correctness goldens are rendered one fresh process per case, because Rich caches a `Style`'s ANSI codes on first use and 28 of 210 goldens depended on render order; (2) `api_surface.py` and `docs_coverage.py` use the lib target and run doctests from the published crate source; (3) the t06 statement matches its target; (4) the t08 statement matches its check (bold stays under `NO_COLOR`); (5) `speed.py verify S3` requires at least 900 of the 1 000 expected frames, read through a terminal emulator; (6) tests for 1 and 5. The per-candidate `reference_quirk` exclusions are dropped; only `documented_deviation` entries survive (3, all `rich-rs` table `no_wrap`). Self-test: 16 tests pass (Rich 210/210 on its own corpus from fresh renders, regeneration deterministic, the S3 frame check separates a full redraw and a changed-cells-only redraw from half the frames and from the final frame alone). The speed-reproducibility test (ratio of two runs within 10%) failed once at 1.118 while other sessions loaded the machine and passed on the rerun.

## Verdicts

Strictly by the rules of `evaluation.md`. A gate failure on axis 1 or 2 puts a candidate out as engine (Engine rule 1). The fuzz criterion (zero panics, 1 000 inputs per feature) and first-try success need harness parts that are not built, so those criteria are INCONCLUSIVE for every candidate; neither changes a verdict below, because each NO-GO rests on a deterministic gate that failed.

| candidate | axis 1 correctness gate | axis 2 assertiveness gate | engine verdict | S1 at most 0.10x Python Rich | API adopted as is |
|---|---|---|---|---|---|
| rich_rust 0.2.3 | fail: 101/210 = 48.1% | fail: width 63.6%, splits 797 / 623, 22 misaligned rows, capability 31/40 | **NO-GO** (rule 1) | fail: 0.933 [0.918, 0.952] | NO-GO on the deterministic thresholds |
| rs-rich 0.0.9 | **pass: 210/210 = 100%**, style and markup 100% (fuzz: INCONCLUSIVE) | fail: splits 176 / 119, capability 31/40 (width 500/500 and table alignment pass) | **NO-GO** (rule 1, axis 2) | pass: 0.087 [0.085, 0.092] | NO-GO on the deterministic thresholds |
| richrs 0.2.1 | fail: 33/210 = 15.7%, style and markup below 100% | fail: width 87.8%, fold unsupported, 11 misaligned rows, capability 5/40 | **NO-GO** (rule 1) | workload discarded (informational 0.156) | NO-GO on the deterministic thresholds |
| rich-rs 1.3.0 | fail: 66/207 = 31.9% | fail: width 87.8%, splits 797 / 217, 6 misaligned rows, capability 22/40 | **NO-GO** (rule 1) | workload discarded (informational 0.201) | NO-GO on the deterministic thresholds |
| composed (owo-colors + comfy-table + indicatif) | fail: 127/210 = 60.5% | fail: width 87.8%, 6 misaligned rows, capability 22/40 (splits 0 / 0 pass) | **NO-GO** (rule 1) | workload discarded (informational 0.072) | NO-GO on the deterministic thresholds |

**Engine decision, rule 3: no candidate passes axes 1 and 2, so the rule says GO: own engine**, which then enters this same evaluation as a candidate before release. **API decision:** no existing native API meets every DX threshold (every candidate fails at least one of the deterministic thresholds: name parity, None padding, adoption cost, docs), so API rule 2 applies (hud's own API, designed against the failing metrics). The DX axis is still a pipeline check under pilot rule 6 (8 tasks, one author solution each, no repeats, first-try success not run), so the LOC figures and the "at most 0.80x the best Rust candidate" criterion are INCONCLUSIVE until the agent runner exists. Speed does not rank anyone: it ranks candidates that pass both gates, and none does.

## Pilot validity (evaluation.md, rules 1 to 7)

| rule | status |
|---|---|
| 1 same conditions | one machine, one toolchain, Python and Rich pinned by `uv.lock`, crates pinned by committed lockfiles. Axes 1, 2 and 4 are deterministic and unaffected by load. Speed and adoption ran sequentially, one process at a time, each only after a check that no `cargo`, `rustc`, `rustdoc`, `cc`, `ld` or `clang` process existed and the 1 minute load average was at most 4.0; the load at every start (3.18 to 3.99, from other sessions on a 28-user machine, never from this pilot) and the busy list (always empty) are in `pilot/speed_conditions*.log` |
| 2 same work | `speed.py verify` runs before every timing; a workload whose output differs from the golden is discarded for that candidate. Discarded: S1, S2 and S4 for richrs, rich-rs and composed. Their informational timings are labelled "not comparable" and never ranked |
| 3 exercises what it measures | 30 cases per feature (minimum 10), 500 width strings, 40 capability cells, 12 table cases; S2 emits 10 000 rows, S3 shows 1 000 of 1 000 expected frames for all six implementations, S4 emits 1 000 lines |
| 4 self-test | harness self-test passes (above); every task has at least one passing solution (rs-rich and composed solve 8 of 8), so no task is a harness bug |
| 5 attribution | every failure is `candidate_failure`, except: 28 `harness_error`/`task_defect` goldens (fixed, no longer present), 3 `documented_deviation` (rich-rs table `no_wrap`, `Column` docs say not fully implemented), 2 `task_defect` statements (t06, t08, fixed). No remaining `reference_quirk` |
| 6 minimum sample | correctness, assertiveness and speed meet the minimums (30 timing samples after 5 warm-ups). **DX is a pipeline check**: 8 tasks, one author-written solution each, no repeats, no agent runs |
| 7 verdicts | each NO-GO above rests on a deterministic gate under rules 1 to 6. Nothing was withdrawn |

## Axis 1: correctness (210 cases, byte-identical to Rich 15.0.0)

| candidate | style | markup | table | panel | tree | progress | error | overall | style+markup 100% | gate 98% |
|---|---|---|---|---|---|---|---|---|---|---|
| rich_rust 0.2.3 | 30/30 | 30/30 | 3/30 | 5/30 | 26/30 | 0/30 | 7/30 | 101/210 = 48.1% | pass | FAIL |
| rs-rich 0.0.9 | 30/30 | 30/30 | 30/30 | 30/30 | 30/30 | 30/30 | 30/30 | 210/210 = 100.0% | pass | pass |
| richrs 0.2.1 | 13/30 | 16/30 | 0/30 | 2/30 | 2/30 | 0/30 | 0/30 | 33/210 = 15.7% | FAIL | FAIL |
| rich-rs 1.3.0 | 16/30 | 12/30 | 5/27 | 12/30 | 7/30 | 7/30 | 7/30 | 66/207 = 31.9% | FAIL | FAIL |
| composed (owo-colors + comfy-table + indicatif) | 17/30 | 25/30 | 1/30 | 21/30 | 30/30 | 3/30 | 30/30 | 127/210 = 60.5% | FAIL | FAIL |

Zero-panic fuzz is not built: INCONCLUSIVE for all. Classification of every failure is in `pilot/<candidate>/correctness.json` (first differing byte and context).

## Axis 2: assertiveness

| candidate | width agreement (gate 99%) | grapheme splits fold / truncate (gate 0) | misaligned table rows (gate 0) | capability (gate 40/40) |
|---|---|---|---|---|
| rich_rust 0.2.3 | 318/500 = 63.6% (FAIL) | 797 / 623 (FAIL) | 22/98 rows (FAIL) | 31/40 (FAIL) |
| rs-rich 0.0.9 | 500/500 = 100.0% (pass) | 176 / 119 (FAIL) | 0/97 rows (pass) | 31/40 (FAIL) |
| richrs 0.2.1 | 439/500 = 87.8% (FAIL) | unsupported / 0 (FAIL) | 11/82 rows, 1 of 12 tables unsupported (FAIL) | 5/40 (FAIL) |
| rich-rs 1.3.0 | 439/500 = 87.8% (FAIL) | 797 / 217 (FAIL) | 6/97 rows (FAIL) | 22/40 (FAIL) |
| composed (owo-colors + comfy-table + indicatif) | 439/500 = 87.8% (FAIL) | 0 / 0 (pass) | 6/97 rows (FAIL) | 22/40 (FAIL) |

Capability matrix reference: Rich 15.0.0 itself scores 34/40 against the written expectation (it does not read `CLICOLOR` or `CLICOLOR_FORCE`), and its own `chop_cells` splits flags and Indic conjuncts. The gate is therefore stricter than the reference it measures against.

## Axis 3: speed (ratio of medians, 95% bootstrap CI over 30 samples, 5 warm-ups discarded)

Environment `FORCE_COLOR=1 COLORTERM=truecolor TERM=xterm-256color COLUMNS=100`, output to a pipe. S1 is time to first byte of a spawned process; S2 to S4 are in process. "vs best verified Rust candidate" is shown for information: the decision rule compares against the best candidate that passed both gates, and none did.

| workload | candidate | verified | median ms | vs Python Rich (CI) | vs best verified Rust candidate (CI) |
|---|---|---|---|---|---|
| S1 | Python Rich 15.0.0 | reference | 26.16 | 1 | n/a |
| S1 | rich_rust 0.2.3 | yes | 24.42 | 0.933 [0.918, 0.952] | 10.69 [10.19, 10.94] |
| S1 | rs-rich 0.0.9 | yes | 2.28 | 0.087 [0.085, 0.092] | (best) |
| S1 | richrs 0.2.1 | NO: output differs from the golden, workload discarded; informational timing of different work | 4.09 | 0.156 [0.153, 0.162] (not comparable) | not ranked |
| S1 | rich-rs 1.3.0 | NO: output differs from the golden, workload discarded; informational timing of different work | 5.26 | 0.201 [0.182, 0.218] (not comparable) | not ranked |
| S1 | composed (owo-colors + comfy-table + indicatif) | NO: output differs from the golden, workload discarded; informational timing of different work | 1.88 | 0.072 [0.070, 0.075] (not comparable) | not ranked |
| S2 | Python Rich 15.0.0 | reference | 1059.80 | 1 | n/a |
| S2 | rich_rust 0.2.3 | yes | 89.44 | 0.084 [0.083, 0.085] | (best) |
| S2 | rs-rich 0.0.9 | yes | 133.96 | 0.126 [0.125, 0.127] | 1.50 [1.48, 1.51] |
| S2 | richrs 0.2.1 | NO: output differs from the golden, workload discarded; informational timing of different work | 29.82 | 0.028 [0.028, 0.028] (not comparable) | not ranked |
| S2 | rich-rs 1.3.0 | NO: output differs from the golden, workload discarded; informational timing of different work | 600.51 | 0.567 [0.561, 0.572] (not comparable) | not ranked |
| S2 | composed (owo-colors + comfy-table + indicatif) | NO: output differs from the golden, workload discarded; informational timing of different work | 21.41 | 0.020 [0.020, 0.020] (not comparable) | not ranked |
| S3 | Python Rich 15.0.0 | reference | 690.10 | 1 | n/a |
| S3 | rich_rust 0.2.3 | yes | 267.55 | 0.388 [0.381, 0.393] | 35.85 [35.46, 36.34] |
| S3 | rs-rich 0.0.9 | yes | 90.03 | 0.130 [0.128, 0.132] | 12.06 [11.91, 12.19] |
| S3 | richrs 0.2.1 | yes | 8.44 | 0.012 [0.012, 0.012] | 1.13 [1.12, 1.15] |
| S3 | rich-rs 1.3.0 | yes | 2866.25 | 4.153 [4.066, 4.260] | 384.06 [378.37, 394.94] |
| S3 | composed (owo-colors + comfy-table + indicatif) | yes | 7.46 | 0.011 [0.011, 0.011] | (best) |
| S4 | Python Rich 15.0.0 | reference | 43.55 | 1 | n/a |
| S4 | rich_rust 0.2.3 | yes | 12.97 | 0.298 [0.295, 0.306] | 2.21 [2.12, 2.31] |
| S4 | rs-rich 0.0.9 | yes | 5.87 | 0.135 [0.130, 0.141] | (best) |
| S4 | richrs 0.2.1 | NO: output differs from the golden, workload discarded; informational timing of different work | 3.40 | 0.078 [0.077, 0.080] (not comparable) | not ranked |
| S4 | rich-rs 1.3.0 | NO: output differs from the golden, workload discarded; informational timing of different work | 17.14 | 0.394 [0.387, 0.403] (not comparable) | not ranked |
| S4 | composed (owo-colors + comfy-table + indicatif) | NO: output differs from the golden, workload discarded; informational timing of different work | 1.85 | 0.043 [0.042, 0.043] (not comparable) | not ranked |

S1 for `rich_rust` is the default console (it spawns `tput` to size the terminal); its candidate report measured 0.71x with an explicit size, still above 0.10x.

## Axis 4: DX and adoption

LOC are medians over the author's passing solutions (one per task), formatted with `rustfmt --edition 2024`, blanks and comments dropped; Python Rich median 12. Name parity is existence by exact normalized name of the 40 Rich names (`spec/name_parity.json`); the manual signature review required by the metric has not been redone here (the `rich-rs` report found 30/40 = 75% by manual review). Tasks failing: richrs t01, t02, t04, t05, t07 (and t03, t06, t08 pass); rich_rust and rich-rs t08 (`NO_COLOR` drops bold, which the check requires).

| candidate | tasks passing (runs) | LOC median (passing) / ratio vs Python 12 | name parity (existence by name) | None-padding public fns | first-try |
|---|---|---|---|---|---|
| rich_rust 0.2.3 | 7/8 (9/10 runs) | 20 / 1.67x (FAIL) | 27/40 = 67.5% (FAIL) | 4 (FAIL) | not measured (agent runner not built) |
| rs-rich 0.0.9 | 8/8 (10/10 runs) | 15.0 / 1.25x (pass) | 35/40 = 87.5% (pass) | 39 (FAIL) | not measured (agent runner not built) |
| richrs 0.2.1 | 3/8 (5/10 runs) | 14 / 1.17x (pass) | 27/40 = 67.5% (FAIL) | 2 (FAIL) | not measured (agent runner not built) |
| rich-rs 1.3.0 | 7/8 (9/10 runs) | 16 / 1.33x (pass) | 38/40 = 95.0% (pass) | 91 (FAIL) | not measured (agent runner not built) |
| composed (owo-colors + comfy-table + indicatif) | 8/8 (10/10 runs) | 37.0 / 3.08x (FAIL) | 5/40 = 12.5% (FAIL) | 0 (pass) | not measured (agent runner not built) |

### Adoption cost (clean release build of a minimal `hello table` against an empty program, stripped)

| candidate | project | added compile s (gate 15) | added stripped bytes (gate 1 500 000) | transitive deps (gate 60) | pass |
|---|---|---|---|---|---|
| rich_rust 0.2.3 | hello | 4.90 | 2,037,072 | 60 | FAIL |
| rs-rich 0.0.9 | hello | 5.96 | 2,414,976 | 50 | FAIL |
| rs-rich 0.0.9 | hello_nodefault | 4.32 | 2,264,704 | 21 | FAIL |
| richrs 0.2.1 | hello | 4.17 | 155,264 | 44 | pass |
| rich-rs 1.3.0 | hello | 7.99 | 392,288 | 82 | FAIL |
| composed (owo-colors + comfy-table + indicatif) | hello | 1.56 | 311,296 | 25 | pass |

### Docs (`rustdoc --show-coverage`, lib target; doctests run from the published crate source)

| candidate | crate | documented items | with an example | doctests (passed / failed / ignored) |
|---|---|---|---|---|
| rich_rust 0.2.3 | rich_rust | 1016 items = 88.1% | 24 = 2.9% | 30 / 0 / 37 |
| rs-rich 0.0.9 | rs-rich | 984 items = 84.3% | 4 = 0.5% | 4 / 0 / 0 |
| richrs 0.2.1 | richrs | 616 items = 100.0% | 0 = 0.0% | 0 / 0 / 23 |
| rich-rs 1.3.0 | rich-rs | 996 items = 82.2% | 80 = 10.0% | 107 / 0 / 35 |
| composed (owo-colors + comfy-table + indicatif) | comfy-table | 88 items = 80.0% | 4 = 100.0% | 33 / 0 / 0 |
| composed (owo-colors + comfy-table + indicatif) | indicatif | 53 items = 88.3% | 5 = 21.7% | 14 / 0 / 7 |
| composed (owo-colors + comfy-table + indicatif) | owo-colors | 138 items = 100.0% | 20 = 5.5% | 62 / 0 / 0 |

Thresholds: 100% of public items documented, 100% of public types with a compiling doctest. The "with an example" column is the closest measure the tool gives to the second.

## Numbers that differ from the candidate reports

| candidate | metric | candidate report | this pass | reason |
|---|---|---|---|---|
| rich_rust | correctness | 99/208 = 47.6% | 101/210 = 48.1% | 2 cases excluded as `reference_quirk` now count and pass |
| rs-rich | correctness | 182/210 raw, 100% after 28 exclusions | 210/210 = 100% | the 28 excluded cases all pass against the corrected goldens |
| richrs | correctness | 32/210 = 15.2% | 33/210 = 15.7% | style-008 matches the corrected golden |
| rich-rs | correctness | 65/206 = 31.6% | 66/207 = 31.9% | 1 `reference_quirk` case now counts and passes |
| composed | correctness | 122/182 = 67.0% | 127/210 = 60.5% | **23 of the 28 cases excluded as `reference_quirk` fail against the corrected goldens**; the exclusion held for 5 only |
| composed | doctests | owo-colors only, 65 pass | all three crates: 109 pass, 7 ignored | doctests now run from the published source for every crate |
| rich-rs | S3 | 3.99x [3.89, 4.09] | 4.153x [4.066, 4.260] | re-measured in the same session as Python Rich, S3 verified by frames |

## Evidence for each strategy

The decision between an own engine, wrapping one existing engine and a facade over existing pieces belongs to the user. The rules and the numbers:

- **Own engine.** Rule 3 as written selects it: no candidate passes both gates. The two axis 2 items the closest candidate fails are properties of the reference itself: Rich splits flags and Indic conjuncts in `chop_cells` and scores 34/40 on the capability matrix, so any faithful Rich port fails the pre-registered gate by construction. Only an engine built to the hud definition (UAX #29 clusters, a resolver that reads `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR` and `CLICOLOR_FORCE`) can pass it.
- **Wrap or embed one engine.** `rs-rich` is the only candidate that comes close: 210/210 byte-identical, width 500/500, 0 misaligned rows, S1, S2 and S4 byte-equal to the golden and S3 verified by final screen and frames, S1 0.087x and S2 to S4 0.126x to 0.135x of Python Rich. It fails axis 2 on two items, 176 / 119 grapheme splits and 31/40 capability (`FORCE_COLOR` not read; its report says a 3-line workaround reaches 34/40, as Rich, not re-measured here), and the pre-registered adoption and API thresholds (+2.41 MB, 39 None-padding functions, 84.3% documented). It is a 0.0.x crate that the landscape study flagged as agent-run (15 crates at 0.0.x in two months). Wrapping needs fixes inside the engine for both gate failures, and a wrapper must stay within 1.10x of the engine alone.
- **Facade over existing pieces.** The composition has the best startup, the cheapest adoption (+1.56 s, +311 296 bytes, 25 dependencies) and 0 grapheme splits, and 40/40 on capability with a 29-line hand-written resolver. It fails width (87.8%), table alignment, progress and panel parity, and costs 3.08x Python Rich in lines (37 against 12) because panel, tree, error and markup glue is written by the user. Closing width and layout in the glue is most of an engine.

The numbers favor an own engine, using `rs-rich` as the byte-parity oracle (it reproduces Rich 210/210) and the composition as the speed and adoption-cost target.

## What hud must do better than every candidate

Derived only from the failures measured above.

1. **Cluster-correct width and grapheme-safe wrapping together.** No candidate has both: rs-rich has width 500/500 but 176 fold and 119 truncate splits; the composition has 0 splits but 87.8% width and 6 misaligned rows; rich_rust is at 63.6% width with 797 / 623 splits.
2. **A capability resolver on by default that reaches 40/40.** The best default is 31/40 (rich_rust, rs-rich); Rich is 34/40. rich_rust and rich-rs drop bold under `NO_COLOR` (t08 fails); rs-rich ignores `FORCE_COLOR`; the composition needs 29 lines of user code to reach 40/40.
3. **Byte parity of the ANSI stream, including progress, panel and table layout.** Only rs-rich passes all seven features. Progress is 0/30 for rich_rust and richrs, 3/30 for the composition, 7/30 for rich-rs; table is 3/30, 0/30, 1/30 and 5/27.
4. **Zero positional `None` padding.** rs-rich has 39, rich-rs 91, rich_rust 4, richrs 2.
5. **Adoption inside 1.5 MB and 60 dependencies.** rs-rich +2.41 MB (+2.26 MB without default features, 21 dependencies), rich_rust +2.04 MB, rich-rs 82 dependencies.
6. **Docs: every public item documented and every public type with a compiling doctest.** Best items documented among Rich ports is 100% (richrs, with 0 examples and 23 ignored doctests); best examples is rich-rs at 10.0%; ignored doctests: rich_rust 37, rich-rs 35, richrs 23.
7. **Startup at or under 0.10x of Python Rich on the default console.** rs-rich 0.087x; rich_rust 0.933x (default console sizing spawns `tput`). The three discarded workloads measured 0.072x to 0.201x informationally, so the bar is reachable but only rs-rich reaches it with Rich-identical output.
8. **The 8 tasks in about Python Rich's line count.** rs-rich 15 against 12 (author solutions), composition 37, rich_rust 20, rich-rs 16. The "at most 0.80x the best Rust candidate" criterion puts hud's API at 12 lines (0.80 x 15), equal to Python Rich, and richrs solves only 3 of 8 tasks (t01, t02, t04, t05, t07 fail).

## Not measured, INCONCLUSIVE

- Zero panics on 1 000 fuzz inputs per feature (fuzz mode not built).
- First-try success by a fresh agent (runner not built; protocol in `bench-design.md`), and with it the paired comparison against the best Rust candidate and the DX LOC population (agent solutions).
- Speed against the best candidate that passes both gates (none exists), and the "no workload regresses by more than 25%" check.
- Name parity "recognizable signature" review (existence by name only).
- Indic conjunct widths against a real terminal (open in `spec/width_resolutions.json`).
- Compile time and binary size were measured with other sessions on the machine at load up to 4; they are stable to well under the thresholds they are compared with, but not isolated.

## Reproduce

```bash
cd ~/work/sources/hud/bench
bash scripts/regen.sh                                  # goldens, one fresh process per correctness case
uv run python scripts/pilot.py static                  # axes 1, 2 and the deterministic part of 4, every candidate
# docs: scripts/docs_coverage.py candidates/<c>/hello <crate>   (pilot/<c>/docs*.json)
uv run python scripts/pilot.py timed                   # adoption and S1-S4, sequential, guarded by load and builds
uv run python scripts/pilot.py speed S3                # speed for chosen workloads only
uv run python scripts/pilot.py unverified              # informational timing of discarded workloads
uv run python scripts/pilot_report.py                  # writes pilot/tables.md and pilot/summary.json
```

## hud (appended 2026-10-08, v0.6 part B)

hud entered the same evaluation as a candidate under the rules above; the pilot numbers in this file are unchanged. Full results, the verdict and the investigation of the one failing threshold are in `EVALUATION-RESULTS.md`.

| Axis | hud | Best pilot candidate |
|---|---|---|
| Correctness | 210/210 = 100% | rs-rich 210/210 |
| Width / splits (fold, truncate) / capability | 99.2% / 0, 0 / 40 of 40 | rs-rich 100% / 176, 119 / 31 of 40 |
| S1 vs Python Rich; S2, S3, S4 vs the best verified existing | 0.079; 0.237, 0.196, 0.290 (all CI upper bounds under 0.31) | n/a |
| First-try (primary model, 8 x 5) | 40/40 = 100% | not run for pilot candidates |
| LOC median / name parity (by name, by signature) | 11 / 72.5%, 65.0% | rs-rich 15 / 87.5% by name |
| Adoption (hello table) / docs | +2.31 s, +218 KB, 6 deps / 129 of 129 | composed +0.31 MB, 25 deps |
| Verdict | Engine GO; API NO-GO on name parity by signature only | none passes both gates |


## Appended 2026-10-08: same-session re-measurement with hud 1.0 (no pilot record rewritten)

The five pilot candidates were timed again in the same session as hud (`pilot/tables.md`, `pilot/speed_conditions.log`; loads 2.04 to 3.94, none under load). Their static records (correctness, assertiveness, capability, tasks, API) are those above and did not change; their verdicts stand: all five are NO-GO as engines. Speed against Python Rich in this session: `rs-rich` S1 0.083x, S2 0.127x, S3 0.130x, S4 0.139x; `rich_rust` S1 0.975x, S2 0.086x, S3 0.394x, S4 0.313x; the others differ from the golden in S1, S2 and S4 and are not ranked there. hud against the best verified candidate: S2 0.227x, S3 0.175x, S4 0.295x; hud's own verdict and the final tables are in `EVALUATION-RESULTS.md`.
