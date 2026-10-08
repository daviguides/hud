# rich_rust 0.2.3: results

Candidate `rich_rust =0.2.3` (crates.io; unicode-width 0.2.2, crossterm 0.29.0), Python Rich 15.0.0 as reference. Corpus version 1. Measured 2026-10-08 on one machine (see [evidence/machine.txt](evidence/machine.txt)). Thresholds and decision rules are those of `foundation/evaluation.md`, unchanged. Every number below can be recomputed from [evidence/](evidence/) and the commands at the end. How the adapters were built and what the crate does: [NOTES.md](NOTES.md).

## Verdict

**Engine: NO-GO (decision rule Engine.1: fails gate 1 and gate 2).** The gates do not depend on timing, the corpora have the minimum sizes (210 cases, 500 strings, 40 capability cells) and every failure is classified, so pilot rules 1 to 6 hold for these two axes. The same result holds if the 28 goldens affected by the harness defect below are left out (54.4% on the 182 unaffected cases, against a 98% gate).

**API: not adopted as is (API rule 1 not met).** LOC, name parity, API friction, adoption size and docs miss their thresholds; first-try success was not run (the DX agent runner does not exist yet), so the DX axis is partial.

**Speed (axis 3): pipeline-quality only.** The candidate is out as an engine, so it is not ranked; the numbers are kept as data. Session 2 is the reference session (no sibling `rustc`/`cargo`, load average 3.1 to 6.7); session 1 ran under load up to 16.5 and is kept only to show the run-to-run spread.

## Gates at a glance

| Axis | Metric | rich_rust | Threshold | Result |
|---|---|---|---|---|
| 1 Correctness | byte-identical to Rich, 210 cases | 99/208 = **47.6%** (1 `reference_quirk` excluded in style, 1 in table; vs fresh-process goldens 101/210 = 48.1%) | at least 98% | fail |
| 1 | style + markup | 59/59 (100%) | 100% | pass |
| 1 | zero panics, 1 000 fuzz inputs per feature | fuzz mode not built; 0 panics in the 222 corpus runs | zero | not measured |
| 2 Assertiveness | cell width agreement, 500 strings | 318/500 = **63.6%** | at least 99% | fail |
| 2 | grapheme splits in fold / truncate (20 000 each) | **797** / **623** | zero | fail |
| 2 | misaligned table rows (12 tables, 98 rows) | **22** rows in 9 tables | zero | fail |
| 2 | capability matrix (40 cells) | **31/40** (Rich: 34/40) | 100% | fail |
| 3 Speed | S1 first byte vs Python Rich | **0.96x** (default console), 0.71x (explicit size) | at most 0.10x | fail |
| 4 DX | LOC median | **20** (7 passing tasks) vs Python 12: 1.67x (1.54x on the same 7 tasks) | at most 1.5x | fail |
| 4 | name parity | 27/40 = **68%** | at least 70% | fail |
| 4 | None-padding public functions | **4** | zero | fail |
| 4 | adoption: compile / size / deps | +6.5 s / **+2 037 072 B** / 60 | 15 s / 1.5 MB / 60 | fail (size) |
| 4 | docs | **88.1%** documented, **2.9%** with an example; 30 doctests pass, 37 ignored | 100% / 100% | fail |
| 4 | first-try success | not run | at least 80% | not measured |
| 4 | the 8 tasks | 7 of 8 pass; t08 run `no_color_tty` fails | all pass | 1 failure |

## Axis 1: correctness (`cases/correctness.jsonl`, `compare.py`)

Run: `adapters/cases-runner` writes 210 `.ansi` files, 0 unsupported markers, 0 panics. `compare.py` with the classification file `evidence/classifications.json`.

| feature | pass | denominator | rate | failures: visible text identical / layout differs |
|---|---|---|---|---|
| style | 29 | 29 | 100% | 0 / 0 (1 excluded `reference_quirk`) |
| markup | 30 | 30 | 100% | 0 / 0 |
| table | 2 | 29 | 6.9% | 1 / 26 (1 excluded `reference_quirk`) |
| panel | 5 | 30 | 16.7% | 10 / 15 |
| tree | 26 | 30 | 86.7% | 4 / 0 |
| progress | 0 | 30 | 0% | 1 / 29 |
| error | 7 | 30 | 23.3% | 23 / 0 |
| **overall** | **99** | **208** | **47.6%** | 39 / 70 |

By color depth the picture is the same: with `none` (no escapes) tree 7/7, error 7/7, panel 3/7, table 1/7, progress 0/7; with a color system every colored panel, table, error and progress case fails because of how segments are styled and split (see NOTES.md). "Visible text identical" means the text after stripping SGR sequences equals the reference: the defect is in styling or segment splitting, not in layout.

Failure causes (all `candidate_bug`, details and first diffs in `evidence/correctness.json`):

- **Table** under width pressure: columns are allocated differently from Rich and collapse (header `Sigma` rendered as `Si` + `gm`); 26 cases.
- **Panel**: title and subtitle are not styled with the border style, bottom border with subtitle splits differently, `expand=false` panels size from the widest wrapped line rather than the available width.
- **Progress**: `99%` for 999/1000 (truncation, Rich 100%), leading glyph `╺` instead of `╸`, no styling of percentage and counts.
- **Error**: same title and segment styling difference as panels (the layout is right in every case).
- **Tree**: `bold` guide style keeps the bold attribute on the heavy guides (Rich drops it, 4 cases; 2 of them also affected by the golden defect).

Sensitivity to the default print path: with `Console::print(markup)` and no width option (what the README shows) markup is 28/30 and style 26/30, so the style+markup 100% condition fails; the adapter uses `print_with_options(.., with_width(w))`.

## Axis 2: assertiveness

Width agreement (`width_check.py`, reference = Rich `cell_len`, 13 contested strings resolved per `spec/width_resolutions.json`):

| category | agree | | category | agree |
|---|---|---|---|---|
| ascii | 30/30 | | emoji | 28/61 |
| cjk | 70/70 | | emoji_zwj | 2/50 |
| ambiguous | 40/40 | | emoji_tone | 0/40 |
| flags | 30/30 | | keycaps | 0/15 |
| control | 27/30 | | combining | 34/50 |
| zero_width | 25/30 | | mixed | 32/54 |

Contested (Mc spacing marks): 0/13. Per-code-point summation (`unicode-width` 0.2.2, no grapheme clustering) explains the emoji, ZWJ, skin-tone and keycap rows.

Fold and truncate (widths 1 to 40, 500 strings): fold 797 grapheme splits, 0 concatenation mismatches, 127 lines too wide, 255 valid but not greedy; truncate 623 grapheme splits, 355 valid but not maximal. The adapter folds with the crate's `cells::chop_cells` (it has no grapheme-aware primitive); `Text::wrap` was not usable because it word-wraps and drops the whitespace at the break.

Tables with Unicode cells: 22 of 98 rows misaligned, in 9 of 12 tables.

Capability matrix (`capability.py check -- adapters/cap`): 31/40. The 9 mismatches: `NO_COLOR` on a terminal in 3 depths (the crate emits no SGR at all, the expectation keeps bold), `CLICOLOR=0` on a terminal in 3 depths and `CLICOLOR_FORCE=1` on a pipe in 3 depths (not read; Rich fails the same 6 and the written expectation stands).

## Axis 3: speed

Environment: `FORCE_COLOR=1 COLORTERM=truecolor TERM=xterm-256color COLUMNS=100`, output to a pipe. All four workloads verified equal to the goldens before timing (S1 943 B, S2 1 088 681 B, S4 46 698 B byte-identical; S3 final screen equal). 30 samples after 5 warm-ups, median and 95% bootstrap CI (milliseconds). Session 2 (reference):

| workload | rich_rust (explicit size) | Python Rich | ratio (CI) | variants |
|---|---|---|---|---|
| S1 first byte | 24.17 [24.04, 24.39] default console; 17.77 [17.63, 18.08] explicit size | 25.14 [24.85, 25.37] | 0.96 [0.95, 0.98]; 0.71 [0.70, 0.72] | process-spawn floor (`/usr/bin/true`) 1.66 |
| S2 10 000-row table | 91.11 [90.44, 91.94] | 1104.86 [1097.03, 1115.65] | 0.082 [0.082, 0.083] | buffered writer 82.90 (0.91x); default size detection 101.56 (1.12x) |
| S3 100 000 progress updates | 265.39 [264.94, 265.45] | 700.29 [689.56, 708.52] | 0.379 [0.374, 0.385] | default size detection **10 391 [10 178, 10 879] (39.2x slower)** |
| S4 1 000 markup lines | 13.22 [13.18, 13.27] | 42.69 [41.95, 42.89] | 0.310 [0.308, 0.316] | buffered writer 12.97 (0.98x); default size detection 13.15 |

"Explicit size" is `Console::builder().width(COLUMNS).height(24)`. Without it the crate asks `crossterm::terminal::size()` on every `width()`/`height()` call, which spawns `tput cols` and `tput lines` whenever stdout is not a terminal; `Live` does that on every refresh (S3: 1 000 refreshes), and S1 pays it once (6.4 ms of the 24.2). The explicit-size S1 still sits about 16 ms above the process-spawn floor (1.66 ms); that part is not process creation and was not profiled.

Reproducibility (rule 4, within 10%): primary S3 session 1 / session 2 = 1.005 [1.004, 1.009]; S2 = 1.336 and S4 = 1.362, outside the bound because session 1 ran at load average 12 to 16.5 while sibling forks built and benchmarked and `syspolicyd` was using several hundred percent CPU. Session 2: no `rustc`/`cargo` process before or after any measurement ([evidence/speed.run.log](evidence/speed.run.log), no WARN lines), load 3.1 to 6.7 from unrelated user processes. Session 1 log has WARN lines ([evidence/speed_run1.run.log](evidence/speed_run1.run.log)). A clean re-measurement of all candidates one after another, with the machine idle, is `bash candidates/rich_rust/run_speed.sh`.

## Axis 4: DX

LOC (`loc.py`, rustfmt edition 2024, blanks and comments dropped), hand-written solutions in `solutions/src/bin/`:

| task | t01 | t02 | t03 | t04 | t05 | t06 | t07 | t08 |
|---|---|---|---|---|---|---|---|---|
| rich_rust | 16 | 20 | 40 | 13 | 25 | 7 | 32 | 7 (fails run a) |
| Python Rich | 13 | 8 | 22 | 11 | 14 | 9 | 18 | 2 |
| ratio | 1.23 | 2.50 | 1.82 | 1.18 | 1.79 | 0.78 | 1.78 | n/a |

Median over the 7 passing solutions: 20 (Python 12 over all 8 tasks: 1.67x; 13 over the same 7 tasks: 1.54x). These are one author's solutions, not agent solutions, and the author read the crate source; the first-try test is the real LOC population and has not run.

`tasks.py check`: 9 of 10 runs pass. The failure is t08 `no_color_tty`: the program is `Console::builder().highlight(false).build().print("[bold red]error[/] [green]ok[/] plain")`, and under `NO_COLOR=1` on a terminal the crate removes every attribute including bold, while the target keeps `error` bold. It is a `candidate_failure`; working around it means reimplementing the variable check and choosing a color system by hand, which is not the unchanged program the task asks for.

Name parity: 27/40 = 68% (`api_surface.py parity`). Missing: `Console.no_color`, `Console.force_terminal` (both exist on `ConsoleBuilder`, which would make 29/40 = 72.5% under a manual review that counts the builder), `Text.from_markup`, `Panel.fit`, `Tree.add`, `Progress.add_task`, `Progress.advance`, `Progress.update`, `BarColumn`, `TextColumn`, `TaskProgressColumn`, `MofNCompleteColumn`, `SpinnerColumn`.

API friction: 1 355 public functions; 10 with more than 3 positional parameters; 27 with `Option` parameters; 4 None-padding candidates, all low level (`segment::apply_style(.., style: Option, post_style: Option)`, `segment::adjust_line_length(.., style: Option, ..)`, `Measurement::clamp(min: Option, max: Option)` and the free `clamp`). None is on the path of the 8 tasks.

Adoption (`adoption.py`, hello table against an empty program, 3 clean release builds, `strip = "symbols"`): +6.54 s (candidate 6.69 s, baseline 0.15 s), +2 037 072 bytes (2 372 608 against 335 536), 60 transitive dependencies. Size is over the 1.5 MB threshold; compile time and dependency count pass.

Docs (`rustdoc --show-coverage`, run with `--lib` by hand): 1 016 items documented = 88.1%, 24 items with an example = 2.9%. Doctests (`cargo test --doc` on a pristine copy of the published source): 30 passed, 0 failed, 37 `ignore`d.

First-try success: protocol written in `bench-design.md`, runner not built; not measured.

## Classification (pilot rule 5)

Every failed case, run and cell has a record in [evidence/runs.jsonl](evidence/runs.jsonl) (260 records: 210 correctness, 10 task runs, 40 capability cells).

| class | count | where |
|---|---|---|
| `candidate_bug` / `candidate_failure` | 109 correctness cases, 1 task run, 9 capability cells | the crate |
| `reference_quirk` (excluded from the rate) | 2 correctness cases | style-008 and table-020: the stored golden depends on Rich's style cache and the candidate equals the fresh-process render |
| `harness_error` | 0 | |
| `task_defect` | 28 goldens differ from a fresh-process Rich render | see below; they count for no candidate once fixed |
| `documented_deviation` | 0 | `FEATURE_PARITY.md` documents no deviation for any of the failing behaviours |

### Harness defect: 28 of 210 goldens are order-dependent (task_defect)

Python Rich caches ANSI codes on `Style` objects (`Style.parse` is `lru_cache`d and `Style._ansi` is filled on the first render). A style string rendered at one color depth earlier in the process comes out at that depth later, whatever the console says. `scripts/render_reference.py` renders all cases in one process, so 28 stored goldens differ from a fresh-process render of the same case: 18 of 30 progress, 4 panel, 3 table, 2 tree, 1 style (`style-008`: golden `\x1b[37m`, fresh `\x1b[38;2;128;128;128m`). The harness self-test passes because a second render in the same process repeats the same pollution. `golden_pollution.py` lists them ([evidence/golden_pollution.json](evidence/golden_pollution.json)); the fix is one process per case or `Style.parse.cache_clear()` plus clearing `_ansi`, then `regen.sh`. Until the goldens are regenerated, the progress feature is mostly unscorable for any candidate. Effect on this candidate: no change to the verdict (gate fails either way).

## Pilot validity checklist

| rule | status |
|---|---|
| 1 same conditions | axes 1, 2, 4: deterministic, unaffected. Speed: session 2 clean of sibling builds, load 3.1 to 6.7; session 1 not clean (WARN lines, load up to 16.5). Toolchain and machine in `evidence/machine.txt` |
| 2 same work | `speed.py verify` EQUAL for S1 to S4 before every timing; `cases-runner` produced an output for every one of the 210 + 12 cases |
| 3 exercises what it measures | at least 30 cases per claimed feature, 0 missing; S2 10 000 rows, S3 1 000 frames and 8 completed bars, S4 1 000 lines (golden equality) |
| 4 self-test | hand-written solution for all 8 tasks, 9 of 10 runs pass (the failure is the crate's); `cases-runner`, `width-runner`, `cap`, `s1`, `bench` run end to end; speed reproducibility within 10% holds for S3 and fails for S2 and S4 between the loaded and the clean session; harness defect above found |
| 5 attribution | every failure classified in `runs.jsonl` |
| 6 minimum sample | 210 cases, 500 strings, 30 speed samples; DX 8 tasks but no repeats (first-try not run): DX is a pipeline check |
| 7 verdict | engine NO-GO valid (axes 1 and 2); speed and first-try issue no verdict |

## Not measured, and why

- First-try success (agent runner not built).
- Zero-panics fuzz gate (fuzz adapter mode not built; `cases-runner` caught 0 panics in 210 cases).
- The ratio of LOC against "the best existing Rust candidate": needs the other candidates' solutions.
- S2 to S4 against the best Rust candidate: needs the other candidates.

## Reproduce

```bash
cd ~/work/sources/hud/bench
for d in adapters solutions hello; do (cd candidates/rich_rust/$d && cargo build --release --locked); done
B=candidates/rich_rust/adapters/target/release
$B/cases-runner cases/correctness.jsonl results/rich_rust/correctness
python3 candidates/rich_rust/analyze_correctness.py results/rich_rust/correctness results/rich_rust/fresh_golden results/rich_rust/correctness   # after golden_pollution.py
uv run python scripts/compare.py results/rich_rust/correctness --out results/rich_rust/correctness.json
$B/width-runner results/rich_rust/width . && uv run python scripts/width_check.py results/rich_rust/width
uv run python scripts/capability.py check -- $B/cap
for t in t01-table t02-panel t03-progress t04-tree t05-error t06-markup t07-pipe t08-env; do
  uv run python scripts/tasks.py check $t -- candidates/rich_rust/solutions/target/release/${t%%-*}; done
bash candidates/rich_rust/run_speed.sh     # one session, sequential, writes results/rich_rust/speed/
uv run python candidates/rich_rust/make_evidence.py
```
