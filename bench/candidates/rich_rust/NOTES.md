# rich_rust 0.2.3: notes from writing the adapters and the 8 task solutions

Pinned `=0.2.3` (crates.io), edition 2024, rustc 1.94.0. No patch, no fork, no `unsafe` in any adapter. Numbers are in [RESULTS.md](RESULTS.md); this file records what the work showed about the crate's API and behaviour.

## Layout of this directory

| path | content |
|---|---|
| `solutions/` | 8 hand-written DX task programs (`src/bin/t01.rs` to `t08.rs`), depend on `rich_rust` only |
| `adapters/` | `cases-runner`, `width-runner`, `cap`, `s1`, `bench`; depend on `rich_rust` + `serde_json` (corpus parsing) |
| `hello/` | minimal `hello table` crate for adoption, docs and API metrics |
| `diffcase.py`, `analyze_correctness.py`, `golden_pollution.py`, `run_speed.sh` | analysis helpers (not adapters) |

## Decisions in the adapters (so the numbers can be audited)

1. **Console for cases**: `Console::builder().width(w).height(24).emoji(false).highlight(false).file(buffer)`; `force_terminal(true)` + `color_system(..)` for the three depths. `none` is `no_color()` + `force_terminal(false)`: the builder cannot express "terminal on, no color system" (`force_terminal(true)` without a color system re-detects the depth from the environment), so `none` is a non-terminal console. The bytes are the same (no SGR at all); only `is_terminal()` differs.
2. **Plain text with a style** goes through markup (`[style]plain[/]`) and `print_with_options(.., with_width(w))`. `print_styled` is the README's way, but it applies the style to the trailing newline segment too, so the output ends in `\x1b[..m\n\x1b[0m` (the newline is inside the style; with a background color this paints the line break). The markup route avoids it, so the style cases are not failed for that bug.
3. **Wrapping**: `Console::print` does not wrap at the console width unless a width-affecting option is set (`render_str_segments` only computes a width when `justify`, `overflow`, `no_wrap`, `crop`, `soft_wrap` or `width` is set). The adapter passes `with_width(console_width)`. Without it, `markup-021` style long lines overflow instead of wrapping.
4. **Panel body**: `Panel` takes pre-split lines (`Vec<Vec<Segment>>`), it does not wrap or measure its content (`adjust_line_length` crops to the inner width). The adapter wraps with `Text::wrap(inner_width)` and renders nested tables or panels to lines with `Renderable::render` + `segment::split_lines` before handing them to `Panel::new`.
5. **Tree guides**: Rich selects the heavy guide set when the guide style is bold; rich_rust needs `TreeGuides::Bold` explicitly. The adapter sets it when the guide style contains `bold`.
6. **Progress**: rich_rust 0.2.3 has `ProgressBar` (one bar) and no task manager, no columns pipeline, no `completed/total` column. The adapter and `t03` compose per-task `ProgressBar::render` output with a hand-added `" {done}/{total}"` segment (`BarStyle::Line`, `show_brackets(false)`, theme styles `bar.complete`, `bar.back`, `bar.pulse` read with `console.get_style`) inside a custom `Renderable`, and `Live` for the redraw.
7. **Fold and truncate** (width-runner): there is no grapheme-aware primitive in the crate. The adapter loops `cells::chop_cells(rest, w)` (the crate's own hard split by cells) and, when it returns an empty head (first character wider than `w`), emits that one `char` alone to guarantee progress. `Text::wrap` was not used because it word-wraps and drops the whitespace it breaks at, so concatenation would not equal the input.
8. **Speed adapters** use an explicit width (`COLUMNS`) and height (24). The default console (no explicit size) is measured as the variant `detect` because it behaves very differently (finding below).

## Behaviours worth knowing (each one cost a failed first attempt)

- **Terminal size detection spawns `tput`.** `Console::width()` and `height()` call `crossterm::terminal::size()` every time when no explicit size is set. crossterm 0.29 (`terminal/sys/unix.rs`) tries `ioctl` on stdout and, when that fails (any pipe, CI, redirect), runs `tput cols` and `tput lines` as child processes, two spawns per call. `Live` refresh reads the height every frame: S3 (1 000 frames) takes about 11 s with the default console and 0.27 s with `.height(24)`. `COLUMNS` is honoured only because `tput` reads it: t02 prints a 60-column panel with `COLUMNS=60`, 100 with `COLUMNS=100`, 80 with `COLUMNS` unset, and also 80 with `COLUMNS=100` when `tput` is not on `PATH`.
- **`NO_COLOR` removes bold too.** `detect_color_system` returns `None` for any non-empty `NO_COLOR`; with no color system `write_segments_raw` emits no SGR at all. no-color.org (and Rich) keep bold/italic, so t08 run `no_color_tty` and the 3 `NO_COLOR`-on-terminal capability cells fail. A user who wants attributes under `NO_COLOR` has to reimplement the variable check and pick `ColorSystem::Standard` themselves.
- **`CLICOLOR=0` and `CLICOLOR_FORCE=1` are not read** (Rich does not read them either; the written expectation stands).
- **`FORCE_COLOR` makes `is_terminal()` true** (`terminal::is_terminal` checks it before the real tty check), and `Live` writes all of its frames, cursor-control sequences included, into a pipe even when stdout is not a terminal and `FORCE_COLOR` is unset (S3 with output redirected to a file: 920 932 bytes, 1 000 frames; with `FORCE_COLOR=1` 1 453 154 bytes because of the colors).
- **Highlighting is on by default** (`ReprHighlighter`): `"3.2s"` printed on a terminal comes out bold cyan unless `Console::builder().highlight(false)` is set. Same default as Rich, but t06/t08 forbid it, so every solution needs the builder.
- **`Renderable for Text` ignores the options**: `console.print_renderable(&text)` prints no trailing newline and does not wrap. Only `Console::print*` wraps, and only with a width option.
- **Width model is per code point** (`unicode-width` 0.2.2 summed per `char`, no grapheme segmentation): see RESULTS.md for the corpus numbers. Truncation and wrapping cut inside clusters (ZWJ emoji, skin tones, keycaps, flags are fine only when the code points happen to sum right).
- **Table column allocation under width pressure** collapses columns (header `Sigma` rendered as `Si` / `gm` stacked in a 2-cell column in `table-002`) where Rich keeps the header whole and wraps the body. Tables that fit are fine (t01 and t07 pass byte for byte, S2 is byte-identical to Rich).
- **Panel titles are not styled by the border style** and the subtitle is rendered unstyled; Rich colors both. Border segments are also split differently (`╰` then `─`*13 where Rich emits `╰─` + `─`*12 with its own segment merging).
- **Percentages truncate**: `ProgressBar` shows `99%` for 999/1000 (`completed * 100.0 as u32`); Rich rounds to 100%.
- **Progress glyphs**: `BarStyle::Line` uses `╺` (U+257A) as the leading pulse; Rich uses `╸` (U+2578).

## Where the docs and the API disagree

- README "Progress Bars" example `ProgressBar::new().completed(75).total(100).width(40)` does not compile on 0.2.3: `completed` is a private field, not a method (`error[E0599]: no method named completed found ... private field, not a method`). The working form is `ProgressBar::with_total(100).width(40)` followed by `update(75)`.
- README quick start shows output boxes that do not match the code (the `Users` table example prints a rounded top border in the README and a heavy-head table in practice; `Panel::from_text("Hello, World!")` is claimed to print `┌─...┐` square corners, the default is rounded).
- `Cargo.toml` of the crate declares edition 2024 and the README says "nightly required currently"; stable 1.94.0 builds it, so the README note is stale.
- The prelude does not export `Renderable` (needed for any custom renderable), `PrintOptions` (needed for any wrapping `print`), nor `rich_rust::r#box::*` and `markup`; each needs a separate `use`.
- `ConsoleBuilder::no_color()` and `force_terminal()` carry the names Rich users look for on `Console`, so a name search for `Console.no_color` finds nothing (name-parity script counts them as missing).

## What an LLM agent will probably get wrong first try

1. `Panel::from_text(long_text)` expecting wrapping (it crops).
2. `console.print_renderable(&Text::new(..))` expecting a newline.
3. `ProgressBar::new().completed(..)` from the README.
4. Forgetting `.highlight(false)` and getting colored numbers.
5. Expecting `NO_COLOR` to keep bold.
6. Reaching for `Progress`/`add_task`, which does not exist.
7. `use rich_rust::prelude::*` and then `Renderable`/`PrintOptions` not found.

## Harness defects found while building this candidate (not rich_rust)

1. **28 of the 210 correctness goldens depend on process state.** Python Rich caches ANSI codes on `Style` objects (`Style.parse` is `lru_cache`d and `Style._ansi` is set on first render), so a style string rendered at `standard` depth earlier in the process comes out in `standard` codes at `truecolor` depth later. A fresh-process render of the same case differs from the stored golden for 28 ids (`golden_pollution.py`, e.g. `style-008`: golden `\x1b[37m`, fresh `\x1b[38;2;128;128;128m`; `tree-009`, `tree-013`). `scripts/render_reference.py` needs one process per case or the caches cleared between cases; the goldens need regenerating. Until then the classification pass marks a polluted id `reference_quirk` only when the candidate equals the fresh render.
2. `scripts/api_surface.py json` and `scripts/docs_coverage.py` run `cargo rustdoc -p <crate>` without `--lib`, which fails for any crate that also has a binary target (`rich_rust` has `demo_showcase`): "extra arguments to `rustdoc` can only be passed to one target". Both were run by hand with `--lib`.
3. `scripts/docs_coverage.py` runs `cargo test --doc -p <crate>` from a project that depends on the crate; cargo refuses ("not a member of the workspace"). The doctests were run from a pristine copy of the published 0.2.3 source instead.
4. `speed.py time` takes the candidate command after `--`, so `--out` after the command is handed to the candidate; `--out` must precede `--`.
5. Machine conditions: during the timing run other jobs were active (load average above 12, `syspolicyd` at several hundred percent CPU after fresh binaries, sibling builds); see RESULTS.md for what that does to the speed numbers.
