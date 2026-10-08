# rs-rich 0.0.9: results

Candidate `rs-rich =0.0.9` (crates.io, default features; lib name `rich`), measured with the shared harness in `bench/scripts/` against Python Rich 15.0.0 (`golden/manifest.json`). Adapters are in this directory (`src/bin/`), hand-written against the public API, the library is not patched. No threshold or spec file was changed. Raw reports: `results/*.json`, speed samples and conditions: `results/speed/`.

Machine (identical for every number here): Apple M5 Pro, 48 GiB, macOS 26.7, `rustc 1.94.0`, `cargo 1.94.0` (rustdoc metrics: `cargo 1.100.0-nightly`), Python 3.14.3 + Rich 15.0.0 from `uv.lock`. Load average was 3 to 10 during the run because other forks used the machine; every timed block was guarded by `pgrep -x cargo|rustc|cc|ld|rustdoc|clang` empty before and after (see Speed).

## Summary against the pre-registered gates

| Axis / gate | Result | Verdict |
|---|---|---|
| Correctness, style + markup 100% | raw: style 29/30 (1 order-dependent golden), markup 30/30; 100% after classification | pass (see golden note) |
| Correctness, >= 98% byte-identical on >= 200 cases | 182/210 raw (86.7%); 182/182 (100%) after classifying 28 failures; 222/222 against isolated Rich | pass (see golden note) |
| Correctness, zero panics on 1000 fuzz inputs per feature | not run (fuzz mode not built in the harness) | open |
| Assertiveness, width agreement >= 99% on 500 strings | 500/500 (100%), 13/13 contested | pass |
| Assertiveness, zero grapheme splits in fold and truncate | 176 splits in 20 000 folds, 119 in 20 000 truncations (flags and Indic conjuncts) | **fail** |
| Assertiveness, zero misaligned table rows | 0 of 97 rows in 12 tables | pass |
| Assertiveness, capability matrix 100% of 40 cells | 31/40 with the default console, 34/40 with a 3-line `FORCE_COLOR` adapter (Rich: 34/40) | **fail** |
| Speed S1 <= 0.10x Python Rich (time to first byte) | 0.088x, CI [0.087, 0.091] | pass |
| DX, LOC <= 1.5x Python Rich | median 15 vs 12 = 1.25x | pass |
| DX, name parity >= 70% | 35/40 = 88% (existence by name only) | pass |
| DX, zero positional `None` padding | the 8 tasks needed none; 11 public signatures take 2 or more `Option` positionals | see friction |
| DX, <= 15 s added compile, <= 1.5 MB, <= 60 deps | 6.1 s, **2.41 MB**, 50 deps (default); 4.4 s, **2.26 MB**, 21 deps (no default features) | **fail** (size) |
| DX, 100% documented, doctests on every public type | 84.3% documented, 4 items with an example (0.5%), 4 doctests pass | **fail** |
| DX, first-try success >= 80% | protocol only, not run | open |

## Correctness (axis 1)

`cases_runner` renders each case through `Console::builder().width(w).height(24).force_terminal(true).color_system(..).legacy_windows(false).emoji(false).highlight(false).no_color(false)` and writes `Console::render_export` (exactly what `print` writes). 210 cases plus the 12 Unicode tables, clean environment.

| feature | pass | fail (raw) | after classification |
|---|---|---|---|
| style | 29 | 1 | 29/29, 1 reference_quirk |
| markup | 30 | 0 | 30/30 |
| table | 27 | 3 | 27/27, 3 reference_quirk |
| panel | 26 | 4 | 26/26, 4 reference_quirk |
| tree | 28 | 2 | 28/28, 2 reference_quirk |
| progress | 12 | 18 | 12/12, 18 reference_quirk |
| error | 30 | 0 | 30/30 |
| table_unicode (12) | 12 | 0 | 12/12 |

**Golden defect found (affects every candidate).** Rich caches a `Style`'s ANSI codes on first use whatever the color system, so a golden rendered after another case that used the same style carries that other case's color system. Example: `style-008` is `truecolor` with `#808080`; its golden is `\x1b[37m` (standard downgrade, left over from `style-002`, `standard`), while Rich on a fresh process prints `\x1b[38;2;128;128;128m`. `scripts/golden_isolation_audit.py` renders all 222 cases with Rich in a fresh process each: **28 goldens differ** (style 1, table 3, panel 4, tree 2, progress 18), `results/golden_isolation_audit.json`. rs-rich's 28 failures are exactly those 28 and in every one its bytes equal the isolated Rich render (`scripts/isolate_check.py`, `results/classifications.json`, class `reference_quirk`). So rs-rich is byte-identical to Rich on 222/222 cases, and the harness self-test ("Rich scores 210/210") only holds because it renders in the generation order. `compare.py` treats `reference_quirk` as excluded, so the classified rate is 182/182 with 13.3% of the corpus excluded and `progress` left with 12 cases. The goldens need regenerating with a per-case cache reset or a fresh process per case; I did not touch them.

First attempt of the adapter had 20 more failures (error panel): `Text::styled(msg, "bold")` makes bold the base style of everything appended afterwards (`\x1b[1;2m`). Fixed in the adapter (and in `t05`) by building the body with `Text::new("")` plus styled `append`; this was an adapter bug, not a crate defect.

## Assertiveness (axis 2)

`width_runner`: `rich::cells::cell_len` for width, `rich::cells::chop_cells` for fold, first chunk of the fold for truncate (the spec definition), `cases_runner` for the tables.

| check | result |
|---|---|
| width agreement | 500/500 (100%); contested Mc strings 13/13; every category 100% |
| fold | 20 000 cases: **176 grapheme splits**, 0 concat mismatches, 0 too wide, 262 valid but not greedy |
| truncate (first chunk of fold) | **119 grapheme splits**, 0 not-a-prefix, 271 not maximal |
| truncate via `cells::truncate` (variant, not primary) | 119 splits, 924 not-a-prefix (a cut wide cluster becomes a space), 14 not maximal |
| table alignment | 0 misaligned rows of 97 in 12 tables; 12/12 byte-identical to the Rich goldens |

Where the splits are: `chop_cells` uses Rich's own cluster rule (ZWJ swallows the next code point, U+FE0F widens), not UAX #29. It cuts regional-indicator pairs (flags `🇧🇷` becomes `🇧`, `🇷`; 61 cuts) and Indic conjuncts (`क्षत्रिय` becomes `क्`, `ष`, `त्`, `रि`, `य`; 38 cuts); the `mixed` category repeats those two (77). Python Rich 15.0.0's own `chop_cells` returns the same pieces for both strings (checked: `['🇧', '🇷']`, `['क्', 'ष', 'त्', 'रि', 'य']`), so these are Rich-faithful behaviors that the UAX #29 reference of the gate rejects, not rs-rich deviations. `chop_cells` also keeps Rich's quirk (checked in Python) of an empty leading chunk for a grapheme wider than the width (`chop_cells("宽", 1) == ["", "宽"]`), which accounts for the "not greedy" count.

Capability (`cap`, default console, no overrides): **31/40**. The 9 mismatches: `FORCE_COLOR` on a pipe (3 depths), `CLICOLOR=0` on a terminal (3), `CLICOLOR_FORCE=1` on a pipe (3). The core crate never reads `FORCE_COLOR` (it only reads `NO_COLOR`, `COLORTERM`, `TERM`, `COLUMNS`, `LINES`), although Rich does. `cap_env` (3 extra lines: if `FORCE_COLOR` is set, `force_terminal(true)`) scores **34/40**, equal to Rich; the 6 left are the `CLICOLOR*` reference quirks. `results/capability.json`, `results/capability_env_variant.json`.

## Speed (axis 3)

`speed.py verify`: S1 943 B, S2 1 088 681 B, S3 final screen, S4 46 698 B, all EQUAL to the goldens before any timing. 30 measured samples after 5 discarded (S1) or 5 warm-ups (S2 to S4), median and 95% bootstrap CI, rs-rich and Python Rich measured back to back under identical conditions. `FORCE_COLOR=1` is injected by the adapter through `force_terminal(true)` because the crate does not read it (output equality shows the styling path is the same).

| workload | rs-rich median [CI95] | Python Rich median [CI95] | ratio rs-rich / Python [CI95] |
|---|---|---|---|
| S1 first byte | 2.20 ms [2.18, 2.22] | 24.9 ms [24.1, 25.3] | **0.088** [0.087, 0.091] |
| S1 exit | 2.38 ms [2.37, 2.40] | 29.2 ms [28.3, 29.5] | 0.082 [0.080, 0.084] |
| S2 10 000-row table | 132.4 ms [131.5, 132.9] | 1080 ms [1068, 1134] | 0.123 [0.117, 0.124] |
| S3 100 000 progress updates | 94.8 ms [93.8, 95.8] | 720.6 ms [712.7, 729.1] | 0.132 [0.129, 0.134] |
| S4 1 000 markup lines | 5.78 ms [5.70, 5.90] | 41.9 ms [41.2, 42.7] | 0.138 [0.135, 0.142] |

Conditions (`results/speed/conditions.txt`, S1, S2, S4 from the first pass, S3 rerun): S1, S2, S4 ran with no `cargo|rustc|cc|ld` process before or after (load 3.5 to 6.2). The first S3 pair was discarded as contaminated (a compiler process from another fork appeared during it; 89.3 ms / 767.7 ms); the rerun above had none before or after (load 5.3 to 5.7). The first pass log was overwritten by the rerun script header; its content is quoted here. Ratios against the best existing Rust candidate need the other candidates' numbers and are not computed here. The S3 adapter drives `Progress::start(console, stdout, 1.0)` and calls `refresh()` every 100th update; the background tick (1 per second) is longer than the run.

## DX (axis 4)

All 8 task programs build and pass `tasks.py check` (10 runs): t01 t02 t03 t04 t05 t06 t07 t08 (no_color_tty, force_color_pipe, plain_pipe) PASS. Written against the crate's own README and rustdoc.

| task | rs-rich LOC | Python Rich LOC | ratio |
|---|---|---|---|
| t01 table | 24 | 13 | 1.85 |
| t02 panel | 6 | 8 | 0.75 |
| t03 progress | 25 | 22 | 1.14 |
| t04 tree | 12 | 11 | 1.09 |
| t05 error | 18 | 14 | 1.29 |
| t06 markup | 9 | 9 | 1.00 |
| t07 pipe | 24 | 18 | 1.33 |
| t08 env | 10 | 2 | 5.00 |
| median | 15 | 12 | 1.25 |

t08 is 10 lines because the crate does not honor `FORCE_COLOR`; the program reads the variable itself. t01 and t07 carry a `match` for status colors where Python uses a dict.

- **Name parity**: 35/40 (88%). Missing by name: `Console.force_terminal` (it is on `ConsoleBuilder`), `Text.wrap`, `TaskProgressColumn` (enum variant `ProgressColumn::TaskProgress`), `MofNCompleteColumn` (variant `ProgressColumn::MofN`), `Group` (`containers::Renderables`, built from `Arc<dyn Renderable + Send + Sync>`). Existence by name only; the manual signature review is not done.
- **API friction** (`results/friction.txt`): 1336 public functions, 49 with more than 3 positional parameters, 128 with `Option` parameters, 39 proxy candidates. Manual review: 11 distinct signatures take two or more `Option` positionals (`Spinner::update`, `Status::update` with four, `LogRender::render` with four, `LiveProgress::open` and `wrap_read`, `Style::from_color`, `Syntax::line_range`, `Measurement::clamp`, `update_screen`, `highlight_range`); the rest are single-`Option` (`Progress::reset(total)`, `render_lines_*`). The calls the 8 tasks use (`Table`, `Panel`, `Tree`, `Console`, `Progress`) need no `None` padding (`add_task` takes `impl Into<Option<f64>>`); only `Text::append(text, None)` passes one `None` for an unstyled run.
- **Adoption** (`results/adoption_default.json`, `adoption_nodefault.json`; hello-table crate vs empty program, 3 clean release builds each, stripped): default features +6.13 s, **+2 414 976 B**, 50 transitive crates; `default-features = false` +4.38 s, **+2 264 704 B**, 21 crates. Compile times were taken at load average 8 to 10, so treat them as upper bounds; size and dependency count are deterministic. The first default-feature run (8.96 s) was discarded because compiler processes from other forks appeared during it.
- **Docs** (`docs_coverage.py`): 984 public items, 84.3% documented, 4 items with an example (0.5%), `cargo test --doc` 4 passed, 0 failed.
- **First-try (agent) success**: protocol is written in `bench-design.md`, not run.

## Not measured

Fuzz (1000 inputs per feature, zero panics); first-try success by a fresh agent; recognizable-signature review for name parity; the comparison against the other candidates (speed ratio to the best Rust candidate, LOC ratio to the best Rust port).

## Reproduce

```bash
cd bench/candidates/rs_rich && cargo build --release          # Cargo.lock committed, rs-rich pinned =0.0.9
cd ../.. 
candidates/rs_rich/target/release/cases_runner cases/correctness.jsonl candidates/rs_rich/results/cases
uv run python scripts/compare.py candidates/rs_rich/results/cases
.venv/bin/python candidates/rs_rich/scripts/isolate_check.py <report.json> candidates/rs_rich/results/cases cases/correctness.jsonl
candidates/rs_rich/target/release/width_runner candidates/rs_rich/results/width && uv run python scripts/width_check.py candidates/rs_rich/results/width
uv run python scripts/capability.py check -- $PWD/candidates/rs_rich/target/release/cap
uv run python scripts/tasks.py check t01-table -- $PWD/candidates/rs_rich/target/release/t01_table   # t01..t08
candidates/rs_rich/scripts/run_speed.sh                         # sequential, guarded, rs-rich and Python Rich
candidates/rs_rich/scripts/run_adoption.sh candidates/rs_rich/hello candidates/rs_rich/results/adoption_default.json
```
