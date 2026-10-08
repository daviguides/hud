# rich-rs 1.3.0: bench results

Candidate `rich-rs` 1.3.0 (crate `mrsaraiva/rich-rs`, pinned `=1.3.0`) against Python Rich 15.0.0 goldens. Measured 2026-10-08 on Apple M5 Pro, 48 GiB, macOS 26.7, rustc 1.94.0, cargo 1.94.0. Thresholds are the pre-registered ones in `foundation/evaluation.md`; this file reports numbers against them and issues no verdict (rule 7 belongs to the report that compares candidates). Method: `references/studies/bench-design.md`. Adapters and notes: `NOTES.md`. Raw reports: `evidence/`.

## Summary against the pre-registered thresholds

| Axis | Metric | rich-rs | Threshold | Meets |
|---|---|---|---|---|
| 1 Correctness | byte-identical, 206 cases after exclusions (210; 3 `documented_deviation`, 1 `reference_quirk`) | **65/206 = 31.6%** | ≥98% | no |
| 1 | style + markup | style 15/29 = 51.7%, markup 12/30 = 40.0% | 100% | no |
| 1 | Unicode table set (12) | 4/12 | (same gate) | no |
| 1 | fuzz, 1 000 inputs per feature, zero panics | not run (harness mode not built) | 0 panics | n/a |
| 2 Assertiveness | width agreement, 500 strings | **439/500 = 87.8%** (contested Mc: 0/13) | ≥99% | no |
| 2 | grapheme splits in fold, 20 000 folds | **797** | 0 | no |
| 2 | grapheme splits in truncate (`Text::truncate` / first `chop_cells` line) | 217 / 623, plus 826 non-prefix outputs for `Text::truncate` | 0 | no |
| 2 | misaligned table rows | 6 of 97 rows, 5 of 12 tables | 0 | no |
| 2 | capability matrix | **22/40** (Rich 15.0.0: 34/40) | 100% | no |
| 3 Speed | S1 median vs Python Rich, time to first byte / to exit | 4.93 ms vs 25.84 ms = **0.191×** [0.188, 0.193] / 0.172× [0.171, 0.174] | ≤0.10× | no |
| 3 | S2 / S3 / S4 vs Python Rich (informational; the threshold is against the best Rust candidate) | 0.560× / **3.99×** / 0.407× | n/a here | n/a |
| 4 DX | LOC, median over the 7 passing tasks | 16 vs Python Rich median 12 = 1.33× | ≤1.5× Rich; ≤0.80× best Rust (not computed here) | vs Rich yes |
| 4 | name parity, existence by name | 38/40 = 95% (manual signature review: 30/40 = 75%) | ≥70% | yes |
| 4 | API friction, public fns padded with positional `None` | 91 candidates (1 838 public fns; 85 with >3 positionals; 180 with `Option` params) | 0 | no |
| 4 | adoption cost | +8.69 s compile, +392 288 bytes stripped, **82** transitive deps | ≤15 s, ≤1.5 MB, ≤60 deps | deps: no |
| 4 | docs | 82.2% documented, 10.0% with an example; 107 doctests pass, 0 fail, 35 ignored | 100% / 100% | no |
| 4 | first-try success (fresh agent) | not run (protocol only) | ≥80% | n/a |

## Axis 1: correctness (`scripts/compare.py`)

| feature | pass / denominator | classes of the failures |
|---|---|---|
| style | 15/29 | 14 `candidate_bug` (SGR spelling); 1 `reference_quirk` excluded (style-008) |
| markup | 12/30 | 18 spelling |
| table | 5/27 | 10 spelling, 9 attributes only on blank padding cells, 3 visible attribute differences; 3 `documented_deviation` excluded (no_wrap) |
| panel | 12/30 | 14 spelling, 4 visible attribute differences |
| tree | 7/30 | 19 spelling, 4 visible attribute differences |
| progress | 7/30 | 3 spelling, 14 visible attribute differences, 6 layout/text |
| error | 7/30 | 23 spelling |
| table_unicode | 4/12 | 6 spelling, 2 layout |

By color depth: without color (`none`) 46 of 49 cases are byte-identical, so layout and text are mostly right; with color 19 of 161. The dominant cause is SGR spelling: rich-rs closes attributes one by one (`ESC[22m`, `ESC[39m`, `ESC[49m`) and orders attributes differently, where Rich writes `ESC[0m`. `analysis/screen_equiv.py` shows 101 of the 145 byte-different cases have identical screen cells and 44 differ on screen (some of those only because of the contaminated goldens below).

Failure attribution (`analysis/classify.py`, written to `evidence/classifications_cases.json`): 141 `candidate_bug` (101 `sgr_spelling_only`, 25 `attrs_differ_text_same`, 9 `attrs_on_blank_cells_only`, 6 `text_differs`), 3 `documented_deviation` (`Column` docs: "no_wrap ... not yet fully implemented"), 1 `reference_quirk`. Visible differences worth knowing: progress bar half-cell rounding and width fitting (`progress-001`, `progress-008`), tree guide stays bold when `guide_style` is bold (`tree-006`), `no_wrap` columns wrap instead of ellipsizing (`table-007`), table title italic does not cover the padding of the title line.

`console` in the adapters: width, terminal on, color depth, emoji off, highlight off, set through `ConsoleOptions`; table title and caption styled with the Rich default theme through `with_title_style`/`with_caption_style` (`HUD_BENCH_CRATE_DEFAULTS=1` leaves the crate default; the pass count was 65 both with and without it).

## Axis 2: assertiveness

`width.jsonl` (`cell_len`): agreement by category: ascii 30/30, cjk 70/70, emoji 61/61, emoji_zwj 50/50, emoji_tone 40/40, flags 30/30, keycaps 15/15, ambiguous 40/40, mixed 47/54, zero_width 25/30, combining 31/50, **control 0/30** (rich-rs counts control characters that the reference treats as zero cells). `cell_len` is `unicode-width` 0.2.2 per string; emoji sequences are handled, combining marks and controls are not.

`fold.jsonl` (`chop_cells`, which iterates `chars()`): 797 grapheme splits, 127 over-wide lines, 255 valid-but-not-greedy, 0 concatenation errors. `truncate.jsonl`: `Text::truncate(w, Crop, false)` gives 217 splits and 826 outputs that are not a prefix of the input (it pads with a space when it cuts a double-width cell); first line of `chop_cells` gives 623 splits, 0 non-prefix (`--truncate-via chop`).

Capability matrix (`capability.py`, 18 mismatches): 3 cells lose bold under `NO_COLOR` on a terminal (truecolor, 256, 16); 6 cells are `CLICOLOR=0` on a terminal and `CLICOLOR_FORCE=1` on a pipe, which neither rich-rs nor Rich reads (Rich has exactly these 6, `reference_quirk`, expectation unchanged); 4 cells in the 16-color row (`TERM=xterm`, no `COLORTERM`) report truecolor on a terminal and 256 when forced; 5 cells in the `TERM=dumb` row emit `ESC[?7l`/`ESC[?7h` on a terminal.

## Axis 3: speed (`analysis/run_speed.py`)

Environment `FORCE_COLOR=1 COLORTERM=truecolor TERM=xterm-256color COLUMNS=100`, stdout is a pipe the harness drains, 5 warm-ups then 30 measured runs, median with 95% bootstrap CI (5 000 resamples), rich-rs then Python Rich one after the other. Quiet gate before each workload and after each series: no other `cargo`, `rustc`, `bench`, `s1`, `speed.py` or `adoption.py` process (the first S2 to S4 run was repeated: see below). All series below are clean.

| workload | output vs golden | rich-rs median [CI] | Python Rich median [CI] | ratio of medians [CI] |
|---|---|---|---|---|
| S1 first byte | 877 vs 943 bytes: DIFFERENT, same visible cells | 4.93 ms [4.89, 4.99] | 25.84 ms [25.76, 26.04] | 0.191 [0.188, 0.193] |
| S1 exit | same | 5.20 ms [5.15, 5.26] | 30.17 ms [30.07, 30.29] | 0.172 [0.171, 0.174] |
| S2 10 000-row table | 1 098 592 vs 1 088 681 bytes: DIFFERENT, same visible cells | 620.9 ms [608.9, 625.4] | 1 109.0 ms [1 097.1, 1 133.6] | 0.560 [0.543, 0.567] |
| S3 100 000 updates, 1 000 frames | final screen EQUAL | 2 849.1 ms [2 805.6, 2 913.3] | 714.3 ms [708.9, 727.9] | **3.989** [3.892, 4.092] |
| S4 1 000 markup lines | 45 914 vs 46 698 bytes: DIFFERENT, identical cells | 17.5 ms [17.3, 17.7] | 43.0 ms [42.1, 43.3] | 0.407 [0.402, 0.417] |

S1, S2 and S4 outputs differ from the golden in bytes (SGR spelling; S1 and S2 also italicize only the title text, not its padding). `sgr_equiv.py --visible` shows the same visible text and the same visible style on every cell for all three, so the work is the same on screen; per `speed.md` they are `documented_deviation` and comparable with candidates that show the same deviation, and the ratios to Python Rich are informational. `speed.py time` refuses a DIFFERENT workload, so `run_speed.py` imports its timing functions and labels the result.

S3 needed one adapter change that the harness did not catch. rich-rs treats a piped stdout as a non-terminal even with `FORCE_COLOR=1`, so its `Live` printed only the final frame (1 265 bytes, 7.8 ms): `speed.py verify S3` still said EQUAL because it compares only the final screen. The adapter now calls the crate's `set_force_terminal(Some(true))` when `FORCE_COLOR` is set (what Python Rich does), after which rich-rs emits the same 7 028 cursor-up repaints as the Python reference (176 664 bytes against 1 562 913: it rewrites less per frame). The 2.85 s above is the corrected workload. The first S2 to S4 run (before this change) is discarded; `evidence/speed_s1_run.json` still contains it next to the valid S1 and adoption entries, and only those two are used.

## Axis 4: DX

Solutions are `tasks/src/bin/t01.rs` to `t08.rs`, hand-written for the crate API, checked by `tasks.py check` in the harness environment.

| task | result | LOC (rustfmt, no blanks or comments) | Python Rich LOC |
|---|---|---|---|
| t01 table | PASS | 17 | 13 |
| t02 panel | PASS (reads `COLUMNS` itself; `t02_naive`, without it, depends on whether a controlling terminal exists) | 12 | 8 |
| t03 progress | PASS | 29 | 22 |
| t04 tree | PASS | 14 | 11 |
| t05 error | PASS (one `Text` with `\n`; the `Group` version `t05_group` fails) | 16 | 14 |
| t06 markup | PASS | 12 | 9 |
| t07 pipe | PASS | 34 | 18 |
| t08 env | **FAIL** `no_color_tty` (bold lost under `NO_COLOR`); `force_color_pipe`, `plain_pipe` pass | 6 | 2 |

Runs: 9 of 10 pass. Median LOC over the 7 tasks that passed completely: 16; Python Rich median over its 8: 12; ratio 1.33. The `no_color_tty` statement says bold "is allowed to remain" while the check requires it; that wording may be a `task_defect` of the statement, left for the parent to rule on (rich-rs also fails capability rule 5 independently, three matrix cells).

Name parity (`api_surface.py parity`): 38 of 40 exist (missing `Console.no_color`, `Console.force_terminal`; the setter `set_force_terminal` exists). Manual signature review of the 38: 30 recognizable (same shape as Rich), 8 different: `Console.print` (six positionals), `Text.stylize` (argument order), `Text.wrap` (positional `Option`s), `Table.add_column` and `add_row` (take `Column`/`Row`), `Tree.add` (boxed label), `Progress.add_task` (five positionals), `Progress.update` (eight, `Option<Option<f64>>`). This review is reviewer judgement.

API friction (`api_surface.py friction`): 1 838 public functions, 85 with more than 3 positional parameters, 180 with `Option` parameters, 91 flagged as `None`-padding candidates by the script's proxy (the manual pass that separates real padding is not done).

Adoption (`adoption.py`, 3 clean release builds, quiet): candidate hello table 8.80 s median against 0.11 s for the empty program, +8.69 s; stripped binary 727 824 bytes against 335 536, +392 288; 82 unique crates in `cargo tree -e normal`. `syntect` brings `onig` (C), which accounts for most of the compile time.

Docs (`docs_coverage.py`): 996 documented items, 82.2%; 80 items with an example, 10.0%. The script's doctest step did not report for a registry dependency (`cargo test --doc -p rich-rs` has no workspace member to run), so the crate's doctests were run from its registry source with a separate target directory: 107 passed, 0 failed, 35 ignored.

## Pilot validity checks (bench-design.md)

| Rule | State for rich-rs |
|---|---|
| 1 same conditions | machine above; quiet gate on adoption and every speed series (clean); `Cargo.lock` committed for both crates; Python and Rich pinned by `uv.lock` |
| 2 same work | S3 EQUAL; S1, S2, S4 DIFFERENT in bytes but identical visible cells (above); `cases-runner` writes an output for all 222 case ids, `missing` is 0 |
| 3 exercises the measured thing | S2 emits 10 000 rows; S3 emits 7 028 repaints like the reference (after the `set_force_terminal` fix); S4 emits 1 000 lines; correctness has 30 cases per feature |
| 4 self-test | hand-written solution for each of the 8 tasks runs end to end (7 pass fully); `cases-runner`, `width-runner`, `cap`, `s1`, `bench` run end to end |
| 5 failure attribution | every failing case has a class in `evidence/classifications_cases.json`; the task and capability failures are attributed in `NOTES.md` |
| 6 minimum sample | corpora at or above 200 cases and 500 strings; speed 30 samples; first-try and fuzz not run |
| 7 verdict | not issued here |

## Findings about the harness (not changed here)

1. **28 of 210 correctness goldens depend on Rich's per-process `Style` ANSI cache.** Rendering each case in its own fresh process differs from the committed golden for style 1, table 3, panel 4, tree 2, progress 18 (truecolor 9, 256 13, standard 6); example: `style-008` (`#808080`, truecolor) has `ESC[37m` in the golden but `ESC[38;2;128;128;128m` in a fresh render, because an earlier case parsed the same style at 16 colors. List: `evidence/golden_fresh_process_diff.json`. Only `style-008` could be marked `reference_quirk` for rich-rs; the other contaminated cases also differ in spelling and stay `candidate_bug` either way.
2. **`speed.py verify S3` is blind to frame emission**: it compares the final screen only, so a candidate that never animates passes (rich-rs did until the adapter forced terminal mode). The sentence "golden equality implies it" in `bench-design.md` rule 3 does not hold for S3.
3. **`docs_coverage.py` does not produce the doctest result for a dependency** (see Axis 4, Docs).
4. **`t08` `no_color_tty`** statement and check disagree on bold (above).
5. The `control` width category (30 strings, tab and BEL among them) scores 0/30 because rich-rs counts a control character as 1 cell (`a\tb` is 3) where Rich and `wcwidth` count 0 (2). The corpus value follows Rich; whether controls should occupy a cell is a design choice the corpus makes for hud.
