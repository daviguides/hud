# rich-rs 1.3.0: adapter notes

What an adapter author or an agent hits when writing against rich-rs. Every claim names the evidence; commands run from `bench/`. Numbers are in `RESULTS.md`.

## Layout of this directory

| path | content |
|---|---|
| `tasks/` | crate with only `rich-rs = "=1.3.0"`: `t01`..`t08` (the 8 DX task solutions), `s1` (speed S1), `cap` (capability one-liner), plus two kept-on-purpose failing variants `t02_naive`, `t05_group` |
| `adapters/` | crate with `rich-rs`, `serde_json`: `cases-runner`, `width-runner`, `bench` (S2-S4) |
| `hello/` | minimal hello-table crate for adoption cost, docs coverage and API metrics |
| `analysis/` | `classify.py` (failure attribution), `screen_equiv.py`, `sgr_equiv.py` (supplementary, never a gate), `run_speed.py` (quiet-gated speed and adoption run), `check_tasks.py` (per-run diff for the 8 task solutions, absolute paths) |

Each crate has its own empty `[workspace]` table and pins `rich-rs =1.3.0`.

## What the adapters inject (adapter rule: say so)

| Item | Why | Where |
|---|---|---|
| Console width taken from `COLUMNS` and passed to `Console::set_size` | rich-rs does not read `COLUMNS`; its width comes from `crossterm::terminal::size()`, which asks `/dev/tty` first | `tasks/src/bin/t02.rs`, `t05.rs`, `adapters/src/bin/bench.rs` |
| `with_title_style(italic)` and `with_caption_style(italic dim)` on every table | the schema says the Rich default theme applies where it is silent; rich-rs leaves the title unstyled by default. `HUD_BENCH_CRATE_DEFAULTS=1` switches this off | `adapters/src/lib.rs` |
| `end = ""` for Table, Panel, Tree, Progress; `"\n"` for Text | `Console::print` appends `end` unconditionally; those renderables already end in a newline | `adapters/src/lib.rs` (`render_case`) |
| Error layout as one `Text` built with `Text::append` | blank `Text::plain("")` lines vanish inside a `Group` (below) | `adapters/src/lib.rs` (`error_body`) |
| Progress case rendered by printing a `Progress` as a `Renderable` | the crate offers no other single-frame path | `adapters/src/lib.rs` |
| Fold = `chop_cells`; truncate = `Text::truncate(w, Crop, false)` | the crate's own hard-fold and truncate facilities; `--truncate-via chop` gives the first line of the fold instead | `adapters/src/bin/width-runner.rs` |

No copy of Rich output, no unsafe, no patching of the crate.

## API friction

1. **`Console::print(&x, None, None, None, false, "\n")`**: four positional parameters that are almost always `None`/`false`, plus a terminator. The README shows exactly this call. `end` is added after any renderable: for a `Table` the first attempt printed a trailing blank line (440 vs 441 bytes, `t01`); `""` is right for Table/Panel/Tree and `"\n"` for Text, which is not documented.
2. **Width.** `Console::new()` ignores `COLUMNS`. With a controlling terminal of 150 columns and stdout piped with `COLUMNS=100`, the panel of `t02_naive` is 150 wide (`script -q /dev/null sh -c 'stty cols 150 rows 40; COLUMNS=100 NO_COLOR=1 TERM=xterm-256color tasks/target/release/t02_naive | cat'`). It passes under `tasks.py check` only because the checker has no controlling terminal and `crossterm` falls back to `tput`, which does read `COLUMNS`. The result depends on how the harness is started.
3. **Everything is `Box<dyn Renderable + Send + Sync>`**: each cell is `Box::new(Text::plain(..))`; `add_row_strs` exists but takes plain strings (no markup). Styles: `Style::parse("bold cyan").unwrap_or_default()`; a typo in the style string silently becomes the default style.
4. **`Text::from_markup(markup, emoji: bool) -> Result`**: the `bool` is easy to get wrong (`rich_print!` passes `true`, so `:smile:` is replaced there and nowhere else documented in the README).
5. **Progress.** `Progress::new(columns, LiveOptions, disable, expand)` has two bare bools; `add_task(&str, bool, Option<f64>, f64, bool)` has three more; `update(..)` has eight parameters including `Option<Option<f64>>`; `TaskProgressColumn::new(false)` takes a bool; `start()`/`stop()` are required or nothing renders; `Progress::with_console` takes `Console<Stdout>` only, so a progress display cannot be captured to a buffer.
6. **`Group::new` is homogeneous** (`IntoIterator<Item = R>` with one `R`); mixing renderables needs `Group::from_arcs`, which is not in the README.
7. **`Text::plain("")` inside a `Group` renders no line.** `t05_group` (kept) loses both blank lines of the error panel. Workaround: one `Text` with embedded `\n` (`t05`).
8. **`NO_COLOR` drops bold.** With `NO_COLOR` set the console color system becomes `None` and `print_segments` writes unstyled text, so bold, italic and underline vanish. Capability rule 5 (and Rich) keeps them on a terminal. `t08` run `no_color_tty` fails for this reason; the statement says bold "is allowed to remain" while the check requires it (possible `task_defect` of the statement; the verdict is the parent's).
9. **`Column` docs contradict behavior.** The struct comment says `justify`, `vertical`, `overflow`, `no_wrap` "are not yet fully implemented"; `justify` works (`t01` passes). `no_wrap` and `overflow` do not behave like Rich (a no_wrap header wraps instead of ellipsizing, `table-007`).
10. **Capability gaps** (`capability.json`): `CLICOLOR` and `CLICOLOR_FORCE` ignored (as Rich); `TERM=xterm` without `COLORTERM` is treated as truecolor on a terminal and 256 when forced; `ESC[?7l`/`ESC[?7h` (DECAWM) is written to a terminal even when `TERM=dumb`.
11. **SGR spelling differs from Rich**: `ESC[22m`/`ESC[39m`/`ESC[49m` per attribute instead of `ESC[0m`, and a different attribute order. 101 of 206 byte-different cases have identical screens (`analysis/screen_equiv.py`), but the pre-registered metric is bytes.

## Bugs and defects found

| Defect | Evidence |
|---|---|
| Hard fold and cell width split grapheme clusters | `chop_cells` iterates `chars()`; `cell_len` is `unicode_width` per code point. 797 splits in 20 000 folds, 87.8% width agreement, 6 misaligned table rows |
| Table title style not applied to the padding of the title line | Rich italicizes the whole padded line; rich-rs only the text (`table-000`) |
| Tree guide keeps `bold` when `guide_style` is bold | `tree-006`: Rich prints the guide without bold, rich-rs with |
| Progress bar half-cell rounding and width fitting differ from Rich | `progress-008`: 99/100 shows a full bar; `progress-001`: bars wider than Rich at width 60 |
| Heavy dependencies, none optional | `syntect` (pulls `onig`, C code), `pulldown-cmark`, `regex`, `crossterm`, `atty` (unmaintained; `std::io::IsTerminal` exists since Rust 1.70) |

## Docs gaps

- 82.2% of public items documented, 10.0% with an example (`rustdoc --show-coverage`); 107 doctests pass, 35 are `ignore`d.
- The README says rich-rs "will automatically detect color support" and says nothing about `COLUMNS`, `NO_COLOR`, `FORCE_COLOR`, `RICH_RS_COLOR_SYSTEM` or the `end` semantics.
- `rich_print!` and `get_console()` exist (`lib.rs`) but not in the README.

## What a first-try agent is likely to get wrong

`print(.., "\n")` on a Table (double newline); blank `Text::plain("")` in a `Group`; trusting `Console::new()` for width; forgetting `progress.start()`/`stop()`; `add_task` bool and `Option` positionals in the wrong order; `Text::from_markup(.., true)` replacing `:word:` sequences; expecting bold to survive `NO_COLOR`; building a heterogeneous `Group` with `Group::new`.

## Not done here

- Fuzz mode (1 000 inputs per feature) is not built in the harness.
- The first-try DX test with a fresh agent was not run (protocol only).
- The correctness golden is contaminated for 28 of 210 cases by Rich's per-process `Style` ANSI cache (fresh-process render differs). Only 1 case could be classified `reference_quirk` for rich-rs, because the others also differ in SGR spelling; see `RESULTS.md`.
