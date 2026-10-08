# Composed baseline: results

Candidate: `owo-colors` 4.4.0 + `comfy-table` 8.0.1 + `indicatif` 0.18.6, with the helper crates the tasks required: `console` 0.16.6 (colour switch for the progress frame), `supports-color` 3.0.2 (colour level, used by `cap`), `unicode-width` 0.2.2 and `unicode-segmentation` 1.13.3 (width, fold, panel wrap), `serde_json` 1.0.145 (case files). Glue written by hand where the three crates have no equivalent: see `NOTES.md`. Measured 2026-10-08 against Python Rich 15.0.0 goldens. Thresholds are the pre-registered ones in `foundation/evaluation.md`; nothing here changes them.

Machine: Apple M5 Pro, 18 logical CPUs, 48 GiB, macOS 26.7 (25G229), rustc 1.94.0, cargo 1.94.0, uv 0.12.23, Python Rich pinned by `uv.lock`. Every timing run checked `pgrep -x cargo|rustc|rustdoc|ld` before and after: empty. Load average was 4 to 16 from other sessions during the day, so numbers are one machine, one session.

## Verdict by gate

| Axis | Gate | Result | Status |
|---|---|---|---|
| 1 Correctness | style and markup 100% byte-identical | style 16/29 (55.2%), markup 25/30 (83.3%) | FAIL |
| 1 Correctness | at least 98% byte-identical over the corpus | 122/182 (67.0%) after excluding 28 `reference_quirk` cases | FAIL |
| 1 Correctness | zero panics on 1 000 fuzz inputs per feature | 0 panics in 222 corpus cases; fuzz mode not built | NOT MEASURED |
| 2 Assertiveness | at least 99% width agreement | 439/500 (87.8%) | FAIL |
| 2 Assertiveness | zero grapheme splits (fold, truncate) | 0 in 20 000 folds and 20 000 truncations | PASS |
| 2 Assertiveness | zero misaligned table rows | 6 of 97 rows in 5 of 12 tables | FAIL |
| 2 Assertiveness | 100% of the capability matrix | 20/40 (one-liner), 22/40 (colour-level aware), 40/40 with a 29 line hand-written resolver | FAIL by default, PASS with glue |
| 3 Speed | S1 at most 0.10x Python Rich | 0.073 (CI 0.072 to 0.075) first byte; output differs from golden | PASS on an unverified workload |
| 3 Speed | S2 to S4 ratios | S2 0.020, S3 0.011, S4 0.049 vs Python Rich; S3 verified, S2 and S4 not | S3 verified, others informational |
| 4 DX | first-try success at least 80% | agent protocol not run | NOT MEASURED |
| 4 DX | LOC at most 1.5x Python Rich | median 37 vs 12 (3.1x) | FAIL |
| 4 DX | name parity at least 70% | 5/40 (12.5%), union of the three crates | FAIL |
| 4 DX | zero positional `None` padding | 0 candidates in 825 public functions | PASS |
| 4 DX | adoption: at most 15 s, 1.5 MB, 60 deps | +1.50 s, +311 296 B, 25 transitive deps (hello project) | PASS |
| 4 DX | docs 100%, doctests compile | 80.0% / 88.3% / 100.0% documented; doctests run for owo-colors only (65 pass) | FAIL |

The speed ratios are against Python Rich, which every candidate shares as reference. The decision rule compares candidates to each other (own engine at most 0.80x the best Rust candidate); this file gives the composed side only.

## 1 Correctness

Corpus: 210 cases in `cases/correctness.jsonl`. `compare.py results/composed/correctness` with the `classifications.json` produced by `scripts/reference_quirks.py` (also committed as `quirks.json`).

| Feature | Byte-identical | Excluded as `reference_quirk` | Failed |
|---|---|---|---|
| style | 16/29 | 1 | 13 |
| markup | 25/30 | 0 | 5 |
| table | 1/27 | 3 | 26 |
| panel | 19/26 | 4 | 7 |
| tree | 28/28 | 2 | 0 |
| progress | 3/12 | 18 | 9 |
| error | 30/30 | 0 | 0 |
| overall | 122/182 (67.0%) | 28 | 60 |

Unicode tables (`cases/table_unicode.jsonl`, 12 cases): 2/12.

### The 28 excluded cases are a defect of the goldens

`scripts/reference_quirks.py` re-renders every case with the pinned Rich in a fresh process and compares with `golden/correctness/`. 28 of 210 goldens differ from their own fresh render. Cause: Rich caches the ANSI codes of a `Style` the first time any console renders it, so a style first rendered at truecolor keeps its truecolor codes in a later 256-colour or standard case (`bar.complete` appears as `38;2;249;38;114` inside a `256` case; `#808080` appears as `37` inside a `truecolor` case). No stateless renderer reproduces those bytes. They are classified `reference_quirk` with this evidence and left out of the denominator, which is how `compare.py` treats that class. The shared goldens should be regenerated with one process per case; the 12 Unicode table goldens were not checked.

### Why the other 60 fail

Supplementary view, not a gate: `scripts/screen_compare.py` compares golden and candidate by what a terminal shows (pyte, cell by cell with attributes), separating SGR spelling from layout.

| Feature | Screen-equal | Text equal, attributes differ | Layout differs |
|---|---|---|---|
| style | 29/29 | 0 | 0 |
| markup | 30/30 | 0 | 0 |
| error | 30/30 | 0 | 0 |
| tree | 28/28 | 0 | 0 |
| panel | 23/26 | 3 | 0 |
| progress | 3/12 | 4 | 5 |
| table | 1/27 | 3 | 23 |
| overall | 144/182 (79.1%) | 10 | 28 |

- **22 failures are SGR spelling only** (style 13, markup 5, panel 4). `owo_colors::Style` writes colour parameters before effects (`31;1`), Rich writes effects first (`1;31`). A single attribute or a single colour matches; any combination does not. The crate has no switch for it.
- **10 failures differ in attributes** (panel 3, table 3, progress 4): `comfy-table` pads a styled cell outside the style while Rich styles the padding too (nested tables in panels, tables with ANSI cells); `console` writes bright colours as `38;5;n` and has no 16-colour mode, so standard-depth progress bars differ.
- **28 failures differ in layout** (table 23, progress 5): `comfy-table` shrinks columns with its own algorithm (different wrap points, e.g. `table-001` wraps the first column at 55 cells where Rich wraps at 48), centres with the odd cell on the left where Rich puts it on the right, and trims whitespace inside cells (`"regex "` right-aligned loses its trailing space). `indicatif` draws a head glyph `╸` at every bar boundary and has no `╺`, while Rich draws a half cell only for odd half-steps, so bars at 25% and 50% differ.

## 2 Assertiveness

`width-runner` writes the corpora; `width_check.py` judges them (`results/composed/width_report.json`).

| Check | Result |
|---|---|
| Width agreement | 439/500 (87.8%); contested Mc strings 0/13 |
| By category | ascii 30/30, cjk 70/70, emoji 61/61, emoji_zwj 50/50, emoji_tone 40/40, flags 30/30, keycaps 15/15, ambiguous 40/40, mixed 47/54, zero_width 25/30, combining 31/50, control 0/30 |
| Fold (20 000) | 0 grapheme splits, 0 concatenation mismatches, 30 lines wider than `w`, 393 valid but not greedy |
| Truncate (20 000) | 0 grapheme splits, 0 not-a-prefix, 288 valid but not maximal |
| Table alignment | 6 of 97 rows misaligned, in 5 of 12 tables |

The 30 control strings all miss by exactly one cell per control character: `unicode_width::UnicodeWidthStr::width` counts C0 and C1 controls as one cell, the reference counts zero. Those 30 are 49% of the 61 misses. The others are spacing combining marks, soft hyphen and zero-width joiner runs among ASCII.

Capability matrix (`capability.py check`, 40 cells):

| Variant | Lines | Cells |
|---|---|---|
| `cap-naive`: `"x".if_supports_color(Stdout, \|t\| t.style(Style::new().bold().truecolor(255, 136, 0)))` | 8 | 20/40 |
| `cap`: colour level from `supports_color::on`, pick truecolor, 256 or 16 | 17 | 22/40 |
| `cap-full`: hand-written resolver for `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, `TERM=dumb`, `COLORTERM`, `TERM` depth | 29 | 40/40 |

The misses of the first two come from `supports-color`: `FORCE_COLOR=1` forces level 1 (16 colours) whatever `COLORTERM` says, `NO_COLOR` removes bold as well as colour, `CLICOLOR=0` is ignored, `TERM=dumb` loses to a forced colour.

## 3 Speed

Method: `scripts/speed_run.py` calls `speed.py`'s own functions (`verify`, `time_s1`, `time_inproc`, `summarize`), same session, Python Rich first and the candidate second for each workload, 5 warm-ups then 30 measured samples, median with 95% bootstrap CI (5 000 resamples), environment `SPEED_ENV`. The only change from `speed.py time` is that an unverified workload is timed and flagged instead of refused.

| Workload | Python Rich median [CI95] | Composed median [CI95] | Ratio (CI95) | Output vs golden |
|---|---|---|---|---|
| S1 first byte | 26.41 ms [26.28, 26.54] | 1.927 ms [1.903, 1.981] | 0.073 (0.072, 0.075) | DIFFERENT, 912 vs 943 bytes |
| S1 exit | 31.25 ms [31.07, 31.38] | 2.106 ms [2.085, 2.171] | 0.067 (0.067, 0.070) | same |
| S2 | 1103.3 ms [1102.1, 1109.2] | 21.98 ms [21.88, 22.12] | 0.020 (0.020, 0.020) | DIFFERENT, 1 131 298 vs 1 088 681 bytes |
| S3 | 702.6 ms [700.7, 704.5] | 7.478 ms [7.449, 7.492] | 0.011 (0.011, 0.011) | EQUAL (final screen) |
| S4 | 39.66 ms [39.50, 39.81] | 1.931 ms [1.878, 1.945] | 0.049 (0.047, 0.049) | DIFFERENT, 46 698 bytes both |

By the pilot-validity rule 2 only S3 is a counted workload for this candidate. S1, S2 and S4 measure different work (SGR order on S1 and S4, layout on S2) and are shown as informational. S3 renders 1 028 frames (1 000 forced with `force_draw` plus a few rate-limited ones at 1 Hz), which is the protocol's frame count; the raw `inc` loop is not what the 7.5 ms measures.

## 4 DX

### Tasks

`tasks.py check` on the 10 runs of the 8 tasks: 10/10 pass (`t01`, `t02`, `t03`, `t04`, `t05`, `t06`, `t07`, `t08` with its three runs).

| Task | t01 | t02 | t03 | t04 | t05 | t06 | t07 | t08 | Median |
|---|---|---|---|---|---|---|---|---|---|
| LOC, composed | 38 | 36 | 28 | 43 | 47 | 21 | 54 | 18 | 37 |
| LOC, Python Rich | 13 | 8 | 22 | 11 | 14 | 9 | 18 | 2 | 12 |

Ratio 3.1x against the 1.5x gate. Counting rule: `loc.py` (rustfmt 2024, blank and comment lines dropped). t02, t04 and t05 are hand-rolled because no panel, tree or error layout exists in the three crates; their glue is inside the file, so it counts.

Task defect found, not corrected here: the statement of `t06` and its embedded target disagree. Line 1 target is `Deploy ok in 3.2s` (the word "in" is plain text) while the statement lists only the three styled words; line 3 target has two spaces between `FAIL` and `retry` while the statement says `" FAIL "` then `retry`. The solution follows the target, which is what "the output must equal the target" requires. Class `task_defect` (evaluation rule: fixed and rerun, counting for no candidate until then).

### Other DX metrics

| Metric | Result | Threshold |
|---|---|---|
| Name parity | 5/40 (12.5%): `Color`, `Column`, `Style`, `Table`, `Table.add_row` (comfy-table 5, owo-colors 2, indicatif 1; union) | at least 70% |
| API friction | 825 public functions: 3 with more than 3 positional parameters, 2 with `Option` parameters, 0 `None`-padding candidates | zero padding |
| Adoption, `hello` project | +1.50 s clean release build (candidate 1.62 to 2.31 s, empty 0.11 s), +311 296 stripped bytes, 25 transitive dependencies | 15 s, 1.5 MB, 60 |
| Adoption, full candidate crate | 38 transitive dependencies (adds `console`, `supports-color`, `unicode-*`, `serde_json`) | 60 |
| Docs coverage | documented items (share): comfy-table 88 (80.0%), indicatif 53 (88.3%), owo-colors 138 (100.0%) | 100% |
| Doctests | owo-colors 65 passed, 0 failed; comfy-table and indicatif cannot run under the shared script (`cargo test --doc -p` refuses crates with dev-dependencies that are not workspace members) | compile |

Not run: the fresh-agent first-try protocol (the runner is not built).

## Reproduce

```bash
cd bench/candidates/composed && cargo build --release
cd ../..
B=candidates/composed/target/release
$B/cases-runner cases/correctness.jsonl results/composed/correctness
cp candidates/composed/quirks.json results/composed/correctness/classifications.json
uv run python scripts/compare.py results/composed/correctness
uv run python candidates/composed/scripts/screen_compare.py results/composed/correctness
$B/width-runner cases results/composed/width && uv run python scripts/width_check.py results/composed/width
uv run python scripts/capability.py check -- $B/cap          # also cap-naive, cap-full
for t in t01-table:t01 t02-panel:t02 t03-progress:t03 t04-tree:t04 t05-error:t05 t06-markup:t06 t07-pipe:t07 t08-env:t08; do
  uv run python scripts/tasks.py check ${t%%:*} -- $B/${t##*:}; done
uv run python scripts/loc.py candidates/composed/src/bin/t0*.rs
uv run python candidates/composed/scripts/speed_run.py
uv run python scripts/adoption.py candidates/composed/hello
uv run python scripts/api_surface.py json candidates/composed/hello comfy-table   # indicatif, owo-colors
uv run python scripts/docs_coverage.py candidates/composed/hello comfy-table
uv run python candidates/composed/scripts/reference_quirks.py quirks.json         # evidence for the 28
```

## Not built, not measured

- Fuzz mode (1 000 random inputs per feature).
- The first-try agent runner and the paired bootstrap over tasks.
- A cross-candidate decision: this file is the composed side only.
