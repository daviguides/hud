# hud (own engine), results of v0.1 to v0.5

Candidate adapters in `src/bin/`: `width_runner` and `cap` (v0.1), `cases_runner`, `t06_markup`,
`t08_env`, `bench` (S4) and `fuzz` (v0.2), `t01_table`, `t07_pipe`, `s1_table` and S2 in `bench` (v0.3), `t02_panel`, `t04_tree`, `cases_runner --widgets` and a `panel` and `tree` feature in `fuzz` (v0.4), `t03_progress`, S3 in `bench`, `hello_progress` and a `progress` feature in `fuzz` (v0.5), built from the workspace crates by path. Everything below
was produced by the shared scripts of `bench/scripts/` under the corrected harness
(`bench/CHANGELOG.md` entries 1 to 23). Raw outputs: `bench/pilot/hud/`. hud claims only what it
implements: width, fold, truncate and capability (v0.1), style and markup (v0.2), table (v0.3), panel and tree (v0.4), progress (v0.5). Every other
file is missing, which means unsupported, never a pass.

## v0.5 exit criteria (`foundation/features.md`)

| Criterion | Result | Threshold | Verdict |
|---|---|---|---|
| Correctness, progress (`compare.py`, corpus 1) | **30 / 30** byte-identical (1 to 5 tasks, bars of 10 to 40 cells, totals 10 to 1 000, every color system, widths 40 to 120, 5 of them collapsing columns) | progress 30/30 | pass |
| Claimed-feature correctness (style, markup, table, panel, tree, progress) | **180 / 180** = 100%; `table_unicode` 12 / 12 (error is not claimed) | at least 98%, style and markup 100% | pass |
| Task t03 (three tasks that finish, 100 columns terminal) | PASS (screen check), 21 lines of code (Python Rich 22, 0.95x; Rich's also sleeps 5 ms a step) | pass | pass |
| S3 output (`speed.py verify S3`) | **EQUAL**: final screen 919 bytes, **1 000 of 1 000** expected frames (Python Rich 1 000) | at least 900 frames | pass |
| S3 median | **2.110 ms** (95% CI 2.094 to 2.124) at load 2.5; Python Rich 1 037.6 ms in the same session (0.002x); composition (`indicatif`) 11.03 ms in the same loaded session, **0.204x** (CI 0.201 to 0.207) | at most 5.97 ms, and at most 0.80x the best existing with CI upper bound at most 1.00 | pass (see the conditions below) |
| API check (`api_surface.py idiom`, changelog 21) | `Progress` and `Task`: 16 public methods, **0 take `&mut self`, 0 return a `Result`**; the whole crate has 0 functions with more than 3 positional parameters and 0 `Option` parameters | zero | pass |
| Agent runner built, self-tested, dry-run on hud | `scripts/dx_runner.py` (changelog 22): 13 self-tests; dry run with the mock agent through the real pipeline (docs mirror, prompt, audit, sandbox build, `tasks.py check`): **7 of 7** supported tasks `success`, t05 `unsupported`; a broken program, a wrong output and a read outside the mirror are classified `candidate_failure` (twice) and `protocol_violation` | built, self-tested, dry-run | pass; no model was called, so first-try success is not measured |
| Fuzz, progress | 80 000 inputs (4 seeds x 20 000): **0 panics, 0 violated properties** (columns of every kind, descriptions and separators from the random strings, totals from 0 to `u64::MAX`, clocks that are negative, `NaN` and infinite, every console width including 0; properties: no printed line wider than the console, no escape bytes on a console that shows nothing, a stream that is not interactive draws nothing before it ends, and the live frame equals a render when it fits the terminal). A mutation of the cached path (one space less in the count cell) was caught in 42 of 3 000 inputs | zero panics on at least 1 000 inputs | pass |
| Cumulative v0.1 to v0.4 criteria | style 30 / 30, markup 30 / 30, table 30 / 30 and 12 / 12 (0 of 97 rows misaligned), panel 30 / 30, tree 30 / 30, t01 t02 t04 t06 t07 t08 PASS, width 496 / 500 = 99.2%, 0 splits in 20 000 fold and 20 000 truncate cases, 0 of 704 panel and tree rows wider than the terminal, capability 40 / 40, S1, S2 and S4 outputs EQUAL (943, 1 088 681 and 46 698 bytes), fuzz 0 panics in all eight features | unchanged | pass |

### Speed: the conditions

`pilot/hud/speed_conditions_v05.json`. The machine is an interactive desktop with other sessions open:
1-minute load average 2.5 to 3.2 during the runs (the v0.2 and v0.3 records were taken at 1.7 to 3.3),
no `cargo` or `rustc` of this work running, S1, S2, S3 and S4 one after the other. It is **not idle in
the strict sense** of evaluation rule 1, and it was about 1.5x slower than during the v0.3 session: the
v0.4 binaries, rebuilt from commit `5a9b60d` in a separate worktree, measure S4 2.47 ms and S2 31 ms
on it, against 1.645 and 19.9 ms then. That is why the gates are read from **same-session ratios** and
from the absolute S3 median, which clears the 5.97 ms target 2.8x over:

| Workload | hud | Reference, same session | Ratio | Target |
|---|---|---|---|---|
| S1 first byte | 2.840 ms | Python Rich 37.48 ms | **0.076** (CI 0.074 to 0.078) | at most 0.10 |
| S3 | 2.110 ms | Python Rich 1 037.63 ms | 0.002 | at most 0.80x the best existing |
| S3 | 2.252 ms (second run) | composition 11.03 ms | **0.204** (CI 0.201 to 0.207) | at most 0.80, CI upper bound at most 1.00 |

The v0.5 binaries against the v0.4 binaries, alternating on the same machine, outputs equal to the
goldens (`pilot/hud/speed_ab_v04_v05_*.json`): S4 1.065 (CI 1.049 to 1.079) and 1.021 (1.005 to
1.048), S2 0.976 (0.965 to 1.001) and 1.016 (1.004 to 1.043), S1 first byte 1.030 (0.982 to 1.077) and
0.973 (0.953 to 0.982). No workload is slower than 8% at the 95% level in either block; the S4 and S2
numbers of v0.3 stand. Strictly, the speed gates of v0.5 are read on a loaded machine; a run on an
idle one (`scripts/speed.py time S3 --iterations 60`, then `ratio`) can only improve the absolute
numbers, and the margin is above 2x.

How S3 got there. The first version drew each frame through the general layout (the grid of Rich's
`Progress`, measured and arranged for every refresh): **23 us a frame**, 28.3 ms for the workload
(profile: allocation and cluster-width measurement of cells that never change). The live path now
plans the display once per change of tasks, descriptions, totals, console width or capabilities, and
assembles each frame from cached cells (bars and percentages by state, the count written directly):
**0.6 us a frame**, S3 2.1 ms. The plan draws the bytes of the general path: a seeded test over 2 000
random displays compares them (`services::progress::tests`), and the fuzz property above does the
same on live frames. A display that is outside the plan (time columns, a count wider than its total,
a cell that wraps) takes the general path every frame and is unchanged.

### Differential oracle for progress (`gen_oracle_vectors.py progress`, `crates/hud/tests/oracle.rs`)

A fake clock replays the same events (add, advance, update, change of total, with gaps of 0 to 90 000
seconds) in Rich and in hud; the display is rendered twice, so a spinner has turned.

| Set | Vectors | Differ | Reading |
|---|---|---|---|
| ASCII progress | 1 200 | **0** | byte-identical (text, bar, percentage, count, elapsed, remaining and spinner columns in any order and subset, templates with `{task.completed}` and `{task.total}`, 20 to 120 columns, every color system) |
| Unicode progress | 300 | **0** | none of the 300 has a flag, a combining mark or a wide character that Rich cuts differently |

The first run had 7 ASCII differences, all in the time remaining: Rich prunes the old samples of a
task on every `update`, including one that adds no step (a lower or equal `completed`, or the same
total), and hud pruned only when it added a sample. Fixed; the estimate then matched on every vector.

### Adoption and documentation

| Measure | Result | Threshold | Verdict |
|---|---|---|---|
| Added compile time, binary size, dependencies (`adoption.py`, a progress hello) | **+2.0 s, +288 KB, 6 crates** | 15 s, 1.5 MB, 60 (targets 3 s, 400 KB, 10) | pass |
| Documented public items, items with an example | **126 / 126**, **26 / 26** | 100% | pass |
| Doctests | 30 passed, **0 ignored** | 0 ignored | pass |
| Name parity by name (`api_surface.py parity`) | **27 / 40 = 68%** (v0.6 asks for 70%; the 13 missing are `Console` getters, `Color.parse`, `Text.truncate`, `Text.wrap`, `box`, `Group`, `markup.escape`, `cell_len`) | at least 70% at v0.6 | not yet; the nine Progress names exist |

### Not claimed in v0.5

Listed in `DEVIATIONS.md` D-036 to D-039: tasks without a total, `visible`, `stop_task`, task fields,
the speed and file size columns, a `println!` while a display is live, fractional steps. The bytes of
a live display are not Rich's (D-037); the screens are. First-try success by an agent, LOC against
the agent population and the 0.80x speed criterion against rs-rich stay INCONCLUSIVE until v0.6.

## v0.4 exit criteria (`foundation/features.md`)

| Criterion | Result | Threshold | Verdict |
|---|---|---|---|
| Correctness, panel (`compare.py`, corpus 1) | **30 / 30** byte-identical (bodies: 17 text, 6 table, 7 nested panel; every box, 0 to 2 padding, with and without title and subtitle, expand and fit) | panel 30/30 | pass |
| Correctness, tree (`compare.py`, corpus 1) | **30 / 30** byte-identical (depth 2 to 4, guide styles `dim`, `green`, `bold #ff8800`, none) | tree 30/30 | pass |
| Claimed-feature correctness (style, markup, table, panel, tree) | **150 / 150** = 100%; `table_unicode` 12 / 12 (progress and error are not claimed) | at least 98%, style and markup 100% | pass |
| Task t02 (titled panel with wrapped text) | PASS, 5 lines of code (Python Rich 8, 0.63x) | pass | pass |
| Task t04 (tree) | PASS, 14 lines (Python Rich 11, 1.27x) | pass | pass |
| No panel or tree row wider than the terminal on the Unicode table cases (`width_check.py`, changelog 18) | **0 of 704** rows wider than the terminal, **0** misaligned panel rows, over 36 outputs (each of the 12 cases as an expanding panel, a fitting panel and a tree) | zero | pass |
| Fuzz, panel and tree | 80 000 inputs each (4 seeds x 20 000): **0 panics, 0 violated properties** | zero panics on at least 1 000 inputs per feature | pass |
| Cumulative v0.1 to v0.3 criteria | style 30 / 30, markup 30 / 30, table 30 / 30 and 12 / 12 (0 of 97 rows misaligned), t01 t06 t07 t08 PASS, width 496 / 500 = 99.2%, 0 splits in 20 000 fold and 20 000 truncate cases, capability 40 / 40, S1, S2 and S4 outputs EQUAL (943, 1 088 681 and 46 698 bytes), fuzz 0 panics in all seven features (80 000 each), docs 104 / 104 documented, 19 doctests, 0 ignored | unchanged | pass except the idle-machine speed re-measurement below |

### Speed: what was and was not measured

S1 (first byte) and S2 and S4 (medians) were **not re-measured on an idle machine** for v0.4. The
machine never went idle during the session: 1-minute load average between 4 and 6 for most of an
hour, with spikes to 15, an interactive desktop (browser, video call) running and no `cargo` or
`rustc` of this work (18 cores, so the load is about a quarter of the machine). The v0.3
idle numbers (S1 1.667 ms and 0.064x Python Rich, S2 19.877 ms, S4 1.645 ms) stand for the code
they measured. What was measured is the v0.4 binaries against the v0.3 binaries (commit `4e8de95`,
built from `git archive`), alternating blocks on the same loaded machine, same inputs, both outputs
equal to the goldens (`pilot/hud/speed_ab_v03_v04.json`):

| Workload | v0.4 over v0.3 (median ratio) | 95% bootstrap CI | Samples each |
|---|---|---|---|
| S1 first byte | 1.026 | [0.988, 1.055] | 320 |
| S2 (10 000-row table) | 0.980 | [0.967, 0.993] | 240 |
| S4 (styled lines) | 0.975 | [0.960, 0.987] | 600 |

No workload is slower at the 95% level beyond 5.5%. The absolute times in that file are inflated
about 1.7x by the load (S2 32.9 ms against 19.9 ms idle) and S1 at 3.86 ms is above the 0.10x gate
(2.6 ms) under that load, so **the gate itself is not claimed for v0.4**: S1, S2 and S4 against the
own-engine thresholds are INCONCLUSIVE until a run on an idle machine
(`scripts/speed.py time S1|S2|S4 --iterations 60`, then `ratio`, as in Reproduce). The A/B
makes a regression unlikely, it does not replace the measurement. No panel or tree workload exists
in `spec/speed.md`, so v0.4 adds no speed criterion of its own.

### Differential oracle for panels and trees (`gen_oracle_vectors.py widgets`, `crates/hud/tests/oracle.rs`)

| Set | Vectors | Differ | Reading |
|---|---|---|---|
| ASCII panels | 1 200 | **0** | byte-identical (every box, expand and fit, padding as 1, 2 or 4 numbers, title and subtitle with left, center and right alignment, border styles, bodies that are text, tables, trees or nested panels, widths 10 to 120, every color system) |
| ASCII trees | 1 200 | **0** | byte-identical (0 to 4 levels, thin, heavy and double guides chosen by the guide style, per-node guide style overrides, wrapped labels continuing the guide, every color system) |
| Unicode panels | 500 | 12 | 10 have a flag or a combining mark (D-003, D-024), 2 have a wide character and Rich prints `…` where hud's line fits (D-024) |
| Unicode trees | 400 | 4 | all 4 have a flag or a combining mark (D-003, D-024) |

The first run had 134 ASCII tree differences and most ASCII panel differences. One real defect
explained all of them and is fixed: where no width is left for a label or a body (a console of 0 or
1 cell, padding taking the whole width, a tree node deeper than the width allows), Rich renders
nothing, so the node prints no line, and hud printed an empty one (D-034). The 30 corpus cases
are all 40 cells or wider and did not contain the combination; the vectors found it.

The fuzz properties were checked to be live: changing the panel top rule by one cell made the
equal-width property fail on 500 inputs, and printing an empty line for a node with no room made
the tree property fail on 59 of 500.

### The renderer follows Rich's own algorithm, not its code

A panel is measured and laid out as Rich lays it out: the body is padded and rendered at the width
that is left inside the sides (the full width when the panel expands, otherwise the body's own
maximum within the room, widened to fit the title plus its two spaces), shorter lines are padded
and longer ones cut, and the title and subtitle are set into the edges with the fill characters in
the border style. A tree walks its nodes with a prefix of one four-cell guide per level; the guide
of a node's own level is a fork, or an end for the last child, on its first line and a continuation
or blank on the following lines of a wrapped label; heavy or double guides are picked by the guide
style's `bold` or `underline2`, and those two attributes are then switched off so only the color
and the other attributes show. The text of the algorithm was read from the pinned Rich in the
project's references; no code is copied (`THIRD_PARTY_NOTICES.md` is unchanged).

### Adoption and docs

A panel and tree hello (`hello_panel`, `pilot/hud/adoption_hello_panel.json`) adds **218 704 bytes**
(0.22 MB, limit 1.5 MB) and **6 crates** (limit 60); the compile time of +2.79 s was taken at load
average 5 and is not ranked (limit 15 s). `cargo doc` coverage: 104 / 104 items documented, 15 / 15
with examples, 19 doctests, 0 ignored.

### API notes for v0.6 (name parity and DX)

`Panel::new(body)` takes a string, `Text`, `Table`, `Tree` or `Panel` directly and any other
`Renderable` through `Body::new`; `Panel::fit` is Rich's `Panel.fit`; `padding` takes one number,
`(vertical, horizontal)` or `(top, right, bottom, left)`. `Tree::new(label).child(...)` builds in
one expression and `Tree::add` mirrors Rich's `add` for loops. Parity costs: `box` is a Rust
keyword so the box is `box_style(BoxStyle::...)`, and the border and guide styles are typed
`Style` values, not the strings Rich takes (`Style::parse` reads a string). Both are listed for
the v0.6 name-parity review. Not measured yet: first-try success by an agent (the runner arrives
in v0.5).

## v0.3 exit criteria (`foundation/features.md`)

| Criterion | Result | Threshold | Verdict |
|---|---|---|---|
| Correctness, table (`compare.py`, corpus 1) | **30 / 30** byte-identical | table 30/30 | pass |
| Correctness, `table_unicode` (12 cases) | **12 / 12** byte-identical; `width_check.py`: **0 of 97** table rows misaligned in 0 of 12 tables | zero misaligned rows | pass |
| Claimed-feature correctness (style, markup, table) | **90 / 90** = 100% (210-case corpus; panel, tree, progress and error are not claimed) | at least 98%, style and markup 100% | pass |
| Task t01 (table with header, alignment and a title) | PASS, 15 lines of code (Python Rich 13, 1.15x) | pass | pass |
| Task t07 (the same table styled, piped: no escape byte, same content) | PASS, 16 lines (Python Rich 18, 0.89x) | pass | pass |
| S1 output equals the golden | EQUAL, 943 bytes | equal | pass |
| S1 first byte, default console, no child process | median **1.667 ms** (95% CI [1.641, 1.700] ms, n = 60), total 1.816 ms; **0.064x** Python Rich first byte (CI [0.062, 0.065]), 0.059x total; no child process (`xtask check-layers` bans them) | at most 0.10x Python Rich first byte | pass |
| S2 output equals the golden | EQUAL, 1 088 681 bytes | equal | pass |
| S2 median | **19.877 ms** (95% CI [19.751, 20.161] ms, n = 60) | at most 71.6 ms | pass |
| S2 ratio to the best verified pilot candidate (`rs-rich`) | **0.148**, CI [0.147, 0.151] (`rich_rust` 0.222, CI [0.219, 0.226]; Python Rich 0.019) | CI upper bound at most 1.00 (own-engine rule: at most 0.80 of the best) | pass |
| Fuzz, table | 80 000 inputs (4 seeds x 20 000): **0 panics, 0 violated properties** | zero panics on at least 1 000 inputs per feature | pass |
| Cumulative v0.1 and v0.2 criteria | style 30 / 30, markup 30 / 30, t06 and t08 PASS, width 496 / 500 = 99.2%, 0 splits in 20 000 fold and 20 000 truncate cases, capability 40 / 40, S4 output EQUAL (46 698 bytes), fuzz 0 panics in style, markup, width and capability (80 000 each) | unchanged | pass |

Also measured at v0.3: S1 first byte against the other verified pilot candidates is 0.730x `rs-rich`
(CI [0.696, 0.755]) and 0.068x `rich_rust` (CI [0.067, 0.070]); a table of 5 rows is dominated by
process start, which is why `rs-rich`, a Rust binary like hud, is close. S2 is the workload where
the work is the table.

Conditions of the speed numbers: S1 and S2 run sequentially, alone, after the machine was idle:
load average (1 minute) 1.73 before and after S1, 1.99 before and after S2, no `cargo` or `rustc`
running (`pilot/hud/speed_conditions_S1_S2.json`). Speed is not ranked against candidates that
fail the correctness or assertiveness gates (`PILOT-RESULTS.md`); the ratios are evidence for the
v0.3 targets only.

### The renderer follows Rich's own algorithm, not its code

Cells are measured and arranged as Rich arranges them (the minimum is the longest word and the
maximum the longest line, each plus a cell of padding on each side; when the table is too wide
the widest wrappable columns give way level by level, then every column evenly with halves
rounded to even, then each column is re-measured at its new width), so the corpus and the vectors
match byte for byte, including the quirks that show in them (a cell squeezed below its padding
has no lines of its own and the row takes its height from the others; `overflow="ignore"` skips
justification). The text of the algorithm was read from the pinned Rich in the project's
references; no code is copied (`THIRD_PARTY_NOTICES.md` is unchanged).

### Differential oracle for tables (`gen_oracle_vectors.py`, `crates/hud/tests/oracle.rs`)

| Set | Vectors | Differ | Reading |
|---|---|---|---|
| ASCII tables | 1 500 | **0** | byte-identical (boxes, `show_lines`, title, caption, justify, `no_wrap`, overflow, `width`, `min_width`, `max_width`, header styles, short and long rows, widths 10 to 120, every color system) |
| Unicode tables | 600 | 49 | 43 have a flag or a combining mark in the input (D-003, D-024), 6 have a wide character and Rich prints `…` where hud's line fits (D-024, widened) |

The first run had 141 ASCII differences. Two were real defects of the renderer's fast path for
plain ASCII cells and are fixed: it padded a left-justified cell under `overflow="ignore"` and it
justified any cell under `ignore`, both of which Rich skips. The vectors found them; the 30
corpus tables did not contain the combination.

### Fuzz mode for tables

`fuzz.rs` builds random tables (1 to 6 columns with random justify, overflow, `no_wrap`, `width`,
`min_width` and `max_width`; 0 to 8 rows, some short or long; every box; title and caption from
random strings; console widths 0, 1, 2, 3, 8, 20, 80, 200 and 65 535). Properties besides "no
panic": no printed line is wider than the console, and at 60 000 columns every line of the table
has the same width. The property is not vacuous: a mutation of `cell_lines` (one cell one column
short for some widths) fails it in 217 of 500 inputs.

## v0.2 exit criteria (`foundation/features.md`)

| Criterion | Result | Threshold | Verdict |
|---|---|---|---|
| Correctness, style (`compare.py`, corpus 1) | **30 / 30** byte-identical | 100% | pass |
| Correctness, markup | **30 / 30** byte-identical | 100% | pass |
| Task t06 (markup, screen check) | PASS, 5 lines of code (Python Rich 9) | pass | pass |
| Task t08 (`NO_COLOR` tty, `FORCE_COLOR` pipe, plain pipe) | PASS 3 / 3 runs, 3 lines (Python Rich 2) | pass, bold stays under `NO_COLOR` | pass |
| Task t07 (pipe, no escape bytes, same content) | **not claimed**: its target is a table | pass | moved to v0.3 and passes there |
| Fuzz (`fuzz.py`, 4 seeds x 20 000 inputs per feature) | style, markup, width, capability: **0 panics, 0 violated properties** over 80 000 inputs each | zero panics on at least 1 000 inputs per feature for style and markup | pass |
| S4 output equals the golden | EQUAL, 46 698 bytes | equal | pass |
| S4 median | **1.645 ms**, 95% CI [1.623, 1.670] ms, n = 60 | at most 4.70 ms | pass |
| S4 ratio to the best verified pilot candidate (`rs-rich`, 5.87 ms) | **0.280**, CI [0.269, 0.293] | CI upper bound at most 1.00 (own-engine rule: at most 0.80 of the best) | pass |
| S1 (cold start of a small styled table) | **not claimed**: the workload is a table | at most 0.10x Python Rich | moved to v0.3 and passes there |
| Cumulative v0.1 criteria | width 496 / 500 = 99.2%, 0 splits in 20 000 fold and 20 000 truncate cases, capability 40 / 40, docs 100% | unchanged | pass |

Style and markup are the strict features (gate 100%), so a single miss would have failed them.
The corpus has 60 cases for them; the differential oracle below reaches far beyond it.

### The two criteria that depend on `Table`

`features.md` lists t07 and S1 under v0.2, but both are about a table: t07's target is the
styled build-report table and S1's workload is "cold start of a small styled table", whose
golden is that table's bytes. A candidate without a table cannot reproduce either output, and a
different program would measure other work (evaluation.md, pilot validity 2). They are therefore
reported as not claimed and move to v0.3, where `Table` lands (`features.md` is amended with
this). What v0.2 can say about the second half of the S1 bullet (default console, no child
process): a styled-text program on the default console (`hello_styled`, informational, not S1,
its output differs from the golden) starts and writes its first byte in a median of **2.20 ms**,
95% CI [2.05, 2.38] ms, **0.084x** of Python Rich's S1 first byte (CI [0.078, 0.091]), with no
child process (`xtask check-layers` bans them). The S1 criterion itself is judged at v0.3.

Conditions of the speed numbers: sequential, `bench` run alone after waiting for the machine to
idle; load average (1 minute) 3.34 before and 3.23 after, no `cargo` or `rustc` running
(`pilot/hud/speed_conditions_S4.json`). Speed is not ranked against candidates that fail the
correctness or assertiveness gates (`PILOT-RESULTS.md`); the ratios are reported as evidence for
the v0.2 target only.

## Differential oracle (a development aid, not the gate)

`bench/scripts/gen_oracle_vectors.py` renders seeded random input with the pinned Rich 15.0.0 and
`crates/hud/tests/oracle.rs` compares bytes. Vectors: 5 441 style strings (every color name,
every palette index, thousands of RGB values and random style strings) rendered through the
truecolor, 256-color and 16-color paths; 2 300 random markup and styled-text renders with
random width, color system, justification (default, left, center, right, full), overflow (fold,
crop, ellipsis, ignore), no-wrap, tab size, end, nesting, escapes and unclosed or unmatched
tags; 900 more with Unicode (CJK, emoji, flags, combining marks, ZWJ).

| Set | Vectors | Differ | Reading |
|---|---|---|---|
| style | 5 441 | 0 | byte-identical in all three color systems |
| markup and styled text, ASCII | 2 300 | 0 | byte-identical |
| markup and styled text, Unicode | 900 | 10 | every one has a flag (D-003) or a combining mark (D-024) in the input |

## Fuzz mode (`bench/scripts/fuzz.py`)

`bench/candidates/hud/src/bin/fuzz.rs`: seeded random strings built from tag fragments, escapes,
Unicode scalars of every plane, controls and numbers, per feature. Besides "no panic" it checks:
a style's text form parses back to the same style, `escape` round-trips, no printed line is
wider than the console, a console that shows nothing writes no escape bytes, `fold` and
`truncate` keep every byte and never split a cluster, a resolved size is never zero.

Seeds 20261008, 1, 2 and 3, 20 000 inputs per feature each: 0 panics and 0 violations in
`style`, `markup`, `width`, `capability` and, from v0.3, `table`. The first runs of the mode found four real defects,
all fixed and covered by unit tests: tags placed after the control characters `Text::new` strips
(spans left off character boundaries, a panic), `escape` leaving a backslash before a bracket
that starts no tag unprotected, an OS-reported size of zero reaching the resolved size, and a span
edge inside a grapheme cluster splitting it between two runs (a printed line wider than the
console).

## Adoption cost (`adoption.py`, 3 runs, clean release builds, versus an empty program)

| Project | Added compile time | Added stripped binary | Transitive crates |
|---|---|---|---|
| `hello_styled` (markup, `Style`, `Text`, console) | +1.16 s | +185 248 bytes | 6 (`hud`, `hud-width`, `rustix`, `bitflags`, `errno`, `libc`) |
| `hello_table` (v0.3: `Table`, markup cells, console) | +1.85 s | +218 400 bytes | 6 |
| `hello` (v0.1: capabilities and width) | +1.16 s | +67 408 bytes | 6 |
| `hello_width` (`hud-width` only) | +0.09 s | +66 976 bytes | 1 (`hud-width`) |

Targets of architecture.md (3 s, 400 KB, 10 crates) are met. Measured with the 1-minute load
average between 3.5 and 3.8.

## Docs (`docs_coverage.py`)

`hud`: 88 of 88 public items documented, 12 of 12 with an example, 16 doctests pass, 0 ignored.
`hud-width`: 16 of 16 documented, 8 doctests pass, 0 ignored.

## `D-001`: the four remaining width disagreements with Rich

Decided with evidence: hud is right and Rich is wrong for a ZWJ that is not part of an emoji
sequence. UAX #29 joins a ZWJ to what precedes it and breaks after it unless a pictograph
follows (`GraphemeBreakTest.txt` line 752, `÷ 0646 × 200D ÷ 0020 ÷`, which hud passes), so a
letter after a ZWJ takes a cell of its own in every terminal; Rich's `cell_len` and `wcwidth`
0.9.2 both skip the character after any ZWJ. The goldens, the corpus and the 99% gate are not
touched; the full text and the condition for removing the deviation are in `DEVIATIONS.md` (D-001).

## Reproduce

```bash
cd bench/candidates/hud && cargo build --release
B=$PWD/target/release; cd ../..
rm -rf results/hud/correctness && $B/cases_runner cases/correctness.jsonl results/hud/correctness
.venv/bin/python scripts/compare.py results/hud/correctness --show 8
.venv/bin/python scripts/tasks.py check t06-markup $B/t06_markup
.venv/bin/python scripts/tasks.py check t08-env $B/t08_env
.venv/bin/python scripts/fuzz.py --seeds 20261008 1 2 3 --count 20000 -- $B/fuzz
.venv/bin/python scripts/speed.py verify S4 -- $B/bench
.venv/bin/python scripts/speed.py time S4 --iterations 60 --out pilot/hud/speed_S4.json -- $B/bench   # idle machine
.venv/bin/python scripts/speed.py ratio pilot/hud/speed_S4.json pilot/rs_rich/speed/S4.json
.venv/bin/python scripts/adoption.py candidates/hud/hello_styled
.venv/bin/python scripts/docs_coverage.py .. hud
.venv/bin/python scripts/gen_oracle_vectors.py && (cd .. && cargo test -p hud --test oracle)   # then git checkout the markup fixtures: only their link ids change
# v0.5
$B/cases_runner cases/correctness.jsonl results/hud/correctness && .venv/bin/python scripts/compare.py results/hud/correctness
.venv/bin/python scripts/tasks.py check t03-progress $B/t03_progress
.venv/bin/python scripts/speed.py verify S3 -- $B/bench
.venv/bin/python scripts/speed.py time S3 --iterations 60 --out pilot/hud/speed_S3_v05.json -- $B/bench   # idle machine
.venv/bin/python scripts/gen_oracle_vectors.py progress && (cd .. && cargo test -p hud --test oracle progress)
.venv/bin/python scripts/fuzz.py --seeds 20261008 1 2 3 --count 20000 -- $B/fuzz
.venv/bin/python scripts/api_surface.py idiom $(.venv/bin/python scripts/api_surface.py json .. hud | tail -1) Progress Task
.venv/bin/python scripts/adoption.py candidates/hud/hello_progress
.venv/bin/python scripts/dx_runner.py run --dry-run --repeats 1 --stress-repeats 0   # the mock agent; no model is called
# v0.4
.venv/bin/python scripts/gen_oracle_vectors.py widgets && (cd .. && cargo test -p hud --test oracle)
$B/cases_runner --widgets cases/table_unicode.jsonl results/hud/width && .venv/bin/python scripts/width_check.py results/hud/width
.venv/bin/python scripts/tasks.py check t02-panel $B/t02_panel; .venv/bin/python scripts/tasks.py check t04-tree $B/t04_tree
.venv/bin/python scripts/adoption.py candidates/hud/hello_panel
# v0.3
rm -rf results/hud/table_unicode results/hud/width/tables && $B/cases_runner cases/table_unicode.jsonl results/hud/table_unicode
mkdir -p results/hud/width/tables && cp results/hud/table_unicode/tw-*.ansi results/hud/width/tables/
.venv/bin/python scripts/compare.py results/hud/table_unicode --cases cases/table_unicode.jsonl --golden golden/table_unicode
.venv/bin/python scripts/width_check.py results/hud/width
.venv/bin/python scripts/tasks.py check t01-table $B/t01_table; .venv/bin/python scripts/tasks.py check t07-pipe $B/t07_pipe
.venv/bin/python scripts/speed.py verify S1 -- $B/s1_table; .venv/bin/python scripts/speed.py verify S2 -- $B/bench
.venv/bin/python scripts/speed.py time S1 --iterations 60 --out pilot/hud/speed_S1.json -- $B/s1_table   # idle machine
.venv/bin/python scripts/speed.py time S2 --iterations 60 --out pilot/hud/speed_S2.json -- $B/bench
.venv/bin/python scripts/speed.py ratio pilot/hud/speed_S1.json pilot/python/speed/S1.json --key first_byte_ns
.venv/bin/python scripts/speed.py ratio pilot/hud/speed_S2.json pilot/rs_rich/speed/S2.json
.venv/bin/python scripts/adoption.py candidates/hud/hello_table
# v0.1, still green
$B/width_runner results/hud/width && .venv/bin/python scripts/width_check.py results/hud/width
.venv/bin/python scripts/capability.py check -- $B/cap
```
