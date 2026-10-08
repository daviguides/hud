# richrs 0.2.1: results

Candidate adapter run, 2026-10-08. Pinned `richrs = "=0.2.1"` (`Cargo.lock` committed). Thresholds and decision rules are in `~/work/projects/daviguides/hud/foundation/evaluation.md` and were not touched. Raw evidence: `evidence/`. Adapter notes, API friction and bugs: `NOTES.md`.

## Gate outcomes

| Gate (evaluation.md) | Threshold | Result | Outcome |
|---|---|---|---|
| 1 Correctness, byte-identical | at least 98% over claimed features | **32/210 = 15.2%** | FAIL |
| 1 Style and markup | 100% | style 12/30 (40.0%), markup 16/30 (53.3%) | FAIL |
| 1 Fuzz, zero panics | 1000 inputs per feature | not run (adapter mode not built yet) | not measured |
| 2 Cell width agreement | at least 99% | **439/500 = 87.8%** | FAIL |
| 2 Grapheme splits in wrap/truncate | zero | truncate: 0 splits in 20 000; fold: no public primitive, unsupported | FAIL (fold unsupported) |
| 2 Table alignment | zero misaligned rows | **11 of 82 rows misaligned**, 7 tables; 1 of 12 tables unsupported (`heavy_head`) | FAIL |
| 2 Capability matrix | 100% | **5/40 cells** | FAIL |

Engine decision, rule 1: richrs fails axes 1 and 2, so it is out as an engine and is not ranked. The failures are properties of the crate (see attribution below), not of the harness, and rules 1 to 6 held for these two axes (corpora are deterministic and at or above the minimum sample; no concurrent run can change a byte output). The unmeasured fuzz sub-criterion does not change this, because the byte-identical gate already fails.

## Axis 1: correctness by feature

| Feature | pass | fail | unsupported | rate |
|---|---|---|---|---|
| style | 12 | 17 | 1 (`grey50` unknown color) | 40.0% |
| markup | 16 | 13 | 1 (`grey50`) | 53.3% |
| table | 0 | 27 | 3 (`heavy_head` box does not exist) | 0.0% |
| panel | 2 | 14 | 14 (body is a panel or table; `Panel::new` takes `Into<Text>`) | 6.7% |
| tree | 2 | 28 | 0 | 6.7% |
| progress | 0 | 30 | 0 | 0.0% |
| error | 0 | 30 | 0 | 0.0% |

`missing` is 0: every case id has an output or an `.unsupported` marker (rule 2 and 3). All 210 cases were classified in `evidence/runs.jsonl` (rule 5); every failure is `candidate_failure`, because richrs documents no deliberate deviation from Rich, so nothing is excluded as `documented_deviation`. No `harness_error` or `task_defect` was found in the run.

Root causes (a case can carry more than one tag; `analysis/classify.py`):

| Cause | Cases |
|---|---|
| text layout differs from the golden (any of the below) | 125 |
| no wrapping inside panel, error, text, table | 46 |
| progress layout and default styles differ | 30 |
| `Column::justify` accepted but ignored | 27 |
| tree indents nested levels by 2 columns, Rich by 4 | 25 |
| SGR spelling only (the visible screen is identical, bytes differ) | 23 |
| no 256 or 16 color downgrade (`38;2` kept at lower depth) | 17 |
| panel title and subtitle left-aligned, Rich centers | 13 |
| table title or caption differs | 11 |
| `show_lines` accepted but ignored | 5 |

23 failures are byte-only: richrs emits one escape per attribute (`\x1b[39m\x1b[9m...`), Rich merges them (`\x1b[9;39m...`). If SGR spelling were free, style would reach 21/30 and markup 27/30, still under 100%.

## Axis 2: assertiveness

| Measure | Result |
|---|---|
| Width agreement | 439/500. By category: ascii 30/30, cjk 70/70, emoji 61/61, emoji_zwj 50/50, emoji_tone 40/40, flags 30/30, keycaps 15/15, ambiguous 40/40, mixed 47/54, zero_width 25/30, combining 31/50, **control 0/30**. Contested (Mc) strings: 0/13 |
| Truncate (`Text::truncate`, widths 1 to 40) | 0 grapheme splits, 0 not-a-prefix, 589 valid but not maximal (a first cluster wider than `w` yields an empty string) |
| Fold | unsupported: no public fold or wrap function |
| Tables | 11 of 82 rows misaligned in 7 tables: cell widths come from `unicode-width` and rows are never wrapped, so wide or combining cells shift borders |
| Capability | 5/40. The default console emits truecolor SGR on pipes, on `TERM=dumb`, under `NO_COLOR` and `CLICOLOR=0`; `FORCE_COLOR` and `CLICOLOR_FORCE` are not read; depth 256 and 16 never downgrade. `evidence/capability.txt` |

## Axis 3: speed

| Workload | Output verified against the golden | Counted |
|---|---|---|
| S1 | DIFFERENT (855 vs 943 bytes) | discarded for this candidate |
| S2 | DIFFERENT (1 088 573 vs 1 088 681 bytes) | discarded |
| S3 | EQUAL (final screen, 8 completed bars) | yes |
| S4 | DIFFERENT (48 230 vs 46 698 bytes) | discarded |

S3, 30 iterations after 5 warm-ups: richrs median **8.54 ms**, CI95 [8.18, 8.67]; Python Rich 15.0.0 median 689.73 ms, CI95 [684.94, 694.83]; ratio **0.012**, CI95 [0.012, 0.013]. S3 frames overwrite each other with `ControlType::CursorUp` through `Console::write_segment`.

Rule 1 caveat: `pgrep -x cargo rustc` was empty before and after each timing, but other forks of this evaluation and graphite daemons were active (load average 2.6 to 3.9). Speed ranks only candidates that pass the gates and richrs does not, so this affects no decision; re-measure S3 alone before using the number in a ranking. S1, S2 and S4 stay without numbers; the comparison against candidates with the same deviation is for the final report.

## Axis 4: DX

### Task suite (hand-written solutions in `src/bin/t01.rs` to `t08.rs`)

| Task | Result | Class | Reason |
|---|---|---|---|
| t01 table | FAIL | candidate_failure | no heavy-head box (the table separator reuses the box chars); `Column::justify` ignored, so Version is not centered and Downloads not right-aligned |
| t02 panel | FAIL | candidate_failure | `Panel` does not wrap; the line overflows the border; title left-aligned |
| t03 progress | PASS | | needs manual padding of the descriptions and a final `write_segments` of `Progress::render` (nothing draws live) |
| t04 tree | FAIL | candidate_failure | nested levels indent 2 columns instead of 4 (`│ ├──` against `│   ├──`) |
| t05 error | FAIL | candidate_failure | only difference: title left-aligned, Rich centers it |
| t06 markup | PASS | | |
| t07 pipe | FAIL | candidate_failure | same box and alignment as t01 |
| t08 env | PASS (3 of 3 runs) | | `NO_COLOR` and `FORCE_COLOR` are not read by the crate; the program implements them itself |

3 of 8 solutions pass. Rule 4 asks for a passing solution per task; for the five failing tasks the Python Rich reference passes, so the task is solvable and the gap is the crate's. Outputs of the failing byte tasks: `evidence/dx_outputs/`.

### Metrics

| Metric | Threshold | Result | Outcome |
|---|---|---|---|
| First-try success | at least 80% | not run (protocol written, runner not built) | not measured |
| LOC | at most 1.5x Python Rich median (12) | per task 19, 15, 20, 19, 18, 7, 29, 14 (Python 13, 8, 22, 11, 14, 9, 18, 2). Median over the 3 passing solutions: 14 (1.17x). Median over all 8: 18.5 (1.54x) | passes on the 3 solutions the rule counts; 3 of 8 is a thin basis. The 0.80x test against the best Rust port waits for the other candidates |
| Name parity | at least 70% | **27/40 = 68%**. Missing: `Console.no_color`, `Console.force_terminal`, `Text.from_markup`, `Text.wrap`, `box`, `box.ROUNDED`, `BarColumn`, `TextColumn`, `TaskProgressColumn`, `MofNCompleteColumn`, `SpinnerColumn`, `Group`, `cell_len` (exists as `richrs::measure::cell_len`, module name differs) | FAIL |
| API friction | zero `None`-padded public functions | 729 public functions, 6 with more than 3 positional parameters, 19 with `Option` parameters, 1 padded: `Progress::update(id, completed, advance, total, visible)` | FAIL |
| Adoption cost | 15 s, 1.5 MB, 60 deps | +4.15 s, +155 264 bytes, 44 transitive dependencies | pass |
| Docs | 100% documented, 100% with compiling doctest | 616 items, 100% documented; 0 items with an example; 23 doctests, all `ignore`, 0 passed | FAIL |

DX gate summary: richrs fails name parity, API friction and docs; passes adoption cost; first-try success is not measured.

## Pilot validity check

| Rule | Status |
|---|---|
| 1 same conditions | one machine (Apple M5 Pro, 48 GiB, macOS 26.7, rustc 1.94.0, cargo 1.94.0; `evidence/measurements.txt`); concurrent activity affects only the S3 timing (above) |
| 2 same work | cases-runner covers all 222 case ids; speed output verified before timing; DX first-try inputs not applicable yet |
| 3 exercises what it measures | 30 cases per feature; S3 emitted 1 000 frames and the final screen matched |
| 4 self-test | hand-written solution per task exists and was checked; width-runner and cases-runner ran end to end |
| 5 attribution | `evidence/runs.jsonl` and `evidence/classifications.json` |
| 6 minimum sample | 210 cases, 500 strings, 30 speed samples met; DX first-try not run |

## Harness issues found (listed, not fixed here)

These are `harness_error`s: they do not count against the crate, and the workaround is recorded.

1. `docs_coverage.py` and `api_surface.py json` fail on richrs: the package has a library and a demo binary (`src/main.rs`), so `cargo rustdoc -p richrs -- ...` is rejected. Run by hand with `--lib`; the same total row and the same rustdoc JSON were then fed to the scripts' own parsing (`api_surface.py parity` and `friction` ran unmodified).
2. `docs_coverage.py` and the spec's `cargo test --doc -p richrs` fail for a registry crate: "requires dev-dependencies and is not a member of the workspace". Doctests were run in a scratch copy of the crate source with `[workspace]` appended.
3. `bench-design.md` says `bench/results/` is git-ignored; evidence needed for the report is copied to `evidence/`.

## Not measured

Fuzz mode; first-try success (agent runner); S1, S2, S4 timings (outputs differ from the golden); the paired bootstrap against other candidates; the 0.80x LOC comparison with the best Rust port.
