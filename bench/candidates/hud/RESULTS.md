# hud (own engine), results of v0.1, v0.2 and v0.3

Candidate adapters in `src/bin/`: `width_runner` and `cap` (v0.1), `cases_runner`, `t06_markup`,
`t08_env`, `bench` (S4) and `fuzz` (v0.2), `t01_table`, `t07_pipe`, `s1_table` and S2 in `bench` (v0.3), built from the workspace crates by path. Everything below
was produced by the shared scripts of `bench/scripts/` under the corrected harness
(`bench/CHANGELOG.md` entries 1 to 15). Raw outputs: `bench/pilot/hud/`. hud claims only what it
implements: width, fold, truncate and capability (v0.1), style and markup (v0.2), table (v0.3). Every other
file is missing, which means unsupported, never a pass.

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
