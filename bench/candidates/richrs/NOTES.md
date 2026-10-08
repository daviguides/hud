# richrs 0.2.1: adapter notes

Written while building `src/lib.rs`, `src/bin/*` and the eight DX solutions. Everything here was checked by running the crate or reading the 0.2.1 source (registry copy); line references are to `src/` of that release.

## Layout

| path | what |
|---|---|
| `src/lib.rs` | case interpreter: JSON case to richrs objects, output captured with `Console::begin_capture` / `end_capture` |
| `src/bin/cases_runner.rs`, `width_runner.rs`, `cap.rs`, `s1.rs`, `bench.rs` | adapters of `bench/README.md` |
| `src/bin/t01.rs` to `t08.rs` | the DX solutions (no `serde_json`; only `richrs`) |
| `hello/` | minimal hello-table crate for adoption, docs and API metrics |
| `analysis/classify.py` | failure attribution (screen equality plus root-cause tags) |
| `analysis/readme_examples/` | the README snippets, verbatim |
| `evidence/` | numbers behind `RESULTS.md` |

Adapter rules followed: public API only, no patch of the crate, no copy of Rich output. Rich's default styles (bold table header, italic title, pink progress bar) are not injected: where richrs has a different default the case fails, as the schema says the golden defines the default theme. Two adapter decisions that matter for reading the numbers: a `Err` from richrs (unknown color, unclosed markup) and a renderable richrs cannot express (nested panel or table body, `heavy_head` box) both become `<id>.unsupported` with the reason inside; the width adapter writes no `fold.jsonl` because richrs has no fold primitive.

## Capability and environment: what a program must do itself

- `Console::new()` reads only `COLORTERM` and `TERM` (`console.rs:808`). It does not read `NO_COLOR`, `FORCE_COLOR` or `CLICOLOR`/`CLICOLOR_FORCE`, and color is not conditioned on the stream: a pipe gets escapes. With `TERM` unset it defaults to truecolor.
- Width comes from `crossterm::terminal::size()` with an 80x24 fallback. On a pipe crossterm falls back to `tput cols`, which reads `COLUMNS` (verified with and without a controlling terminal: `COLUMNS=77` gives 77, unset gives 80). That works but spawns a `tput` process on every `size()` call and is undocumented in richrs; the DX solutions rely on it, the speed adapter sets the width explicitly.
- Colors are not downgraded: `Style::to_ansi` writes the color as given (`38;2;r;g;b`), `Console::write_segment` only distinguishes `ColorSystem::None` from everything else (`console.rs:533`). `ColorSystem::EightBit` and `Standard` change nothing visible.
- `NO_COLOR` that keeps bold (no-color.org) needs two code paths in the program (t08): a console with `ColorSystem::None` drops bold too.
- Idiom every styled-output solution carries: `if !console.is_terminal() || NO_COLOR { console.set_color_system(ColorSystem::None) }` plus the `ColorSystem` import from a module outside the prelude. It accounts for most of the LOC gap to Python Rich.
- `Named colors`: Rich's 256 named colors are missing (`grey50` fails to parse).

## Rendering behavior that differs from Rich

| Area | What richrs does | Evidence |
|---|---|---|
| SGR | one escape per attribute and a full reset after every segment (`\x1b[31m\x1b[1mtext\x1b[0m`); Rich merges (`\x1b[1;31m`) | `style.rs:522`, `segment.rs:207`; 23 cases differ only here |
| Table | `Column::justify`, `no_wrap`, `Table::show_lines`, `expand`, `leading`, footer, title and caption styles, vertical padding are stored and never read by `render`; cells are always left-aligned | `table.rs:560-947` (no use of `col.justify`) |
| Table box | one `BoxChars` set for the whole table, so the header separator reuses the body chars; no `heavy_head`, default is `SQUARE` (Rich default is `HEAVY_HEAD`) | `table.rs:283`, `box_chars.rs` |
| Table width | columns are scaled down proportionally when too wide but cell text is never wrapped or truncated, so rows overflow and borders shift | `table.rs:543-553`; 11 misaligned rows |
| Panel | content split on `\n` only, no word wrap; title and subtitle left-aligned (one `─` then the title); body must be `Into<Text>` | `panel.rs:246-253`, `panel.rs:433-475` |
| Tree | prefix after a non-last parent is `│` plus one space (2 columns), after a last parent three spaces; Rich uses 4 columns each | `tree.rs:333-337`; t04 |
| Progress | `advance` and `update` only mutate state; `auto_refresh` and `refresh_per_second` are never used; `render` returns segments, nothing is drawn; descriptions are not padded to a common width; percentage is `" {:3}%"`; default bar colors are green and bright black even under `NO_COLOR` | `progress.rs:271-505` |
| Live | prints with `eprint!` to **stderr**, takes only `Text` (plain text plus spans, so a styled `Segments` cannot be passed) | `live.rs:133`, `live.rs:221` |
| Text | `Text::from(&str)` does not parse markup; only `Console::print(&str)` and `Markup::parse(..)?.to_text()` do. Table cells and panel bodies from strings are therefore plain unless converted | `text.rs:595`, `console.rs:582` |
| Width | `unicode_width::UnicodeWidthStr::width` on the whole string: control characters and several combining and zero-width cases disagree with Rich (control 0/30, combining 31/50, zero_width 25/30) | `measure.rs:109` |
| Truncate | `Text::truncate(w, None)` cuts at grapheme boundaries but returns an empty string when the first cluster is wider than `w` | `text.rs:488` |
| `print` | takes `&str` only; there is no `console.print(&table)`. A table is `console.write_segments(&table.render(console.width()))?` | `console.rs:573` |

## API friction an agent would hit

1. README Live example does not compile: `live.update(&format!(..))?` (`Text: From<&String>` is missing and `update` returns `()`). README Progress example compiles and prints nothing. `analysis/readme_examples/`, `evidence/readme_examples.txt`.
2. README shows `richrs = "0.1"`; the latest is 0.2.1.
3. `ColorSystem`, `BoxChars`, `Row`, `Segments` and `cell_len` are not in `prelude`; they live in `richrs::console`, `richrs::box_chars`, `richrs::table`, `richrs::segment`, `richrs::measure`.
4. Padding argument orders disagree with Rich and between components: `Table::padding(horizontal, vertical)`, `Panel::padding(left, top, right, bottom)`, Rich uses `(top, right, bottom, left)`.
5. `Progress::update(id, completed, advance, total, visible)`: four `Option`s, the only `None`-padded public function by the harness count.
6. Every call returns `Result`, so each print needs `?` and `fn main() -> Result<()>`.
7. `Table::new()` returns a builder for `title`, `box_chars`, `header_style` but `add_column` and `add_row` take `&mut self`: mixing both needs `let mut table = Table::new().title(..);` and a style the docs do not show.
8. `Cargo` surface: the library package also ships a demo binary and `test_runner.rs` in `src/`, which breaks `cargo rustdoc -p richrs` (needs `--lib`).
9. 23 doctests, all `ignore`d; 0 examples counted by rustdoc coverage.

## Likely agent mistakes (for the first-try run)

Treating `Column::justify` as working; calling `console.print(&table)`; expecting `Progress` to draw; passing markup to `Text::from`; assuming `NO_COLOR` or `FORCE_COLOR` are honored; using the README Live snippet; assuming wrapping inside `Panel`; guessing `Table::add_row(["a", "b"])` (needs `add_row_cells` or `Row::new`); importing `ColorSystem` from the prelude.

## Solutions that needed a workaround (t03, t06, t08 pass)

- t03: descriptions padded by hand to equal width (`"fetch index    "`), a final `write_segments(&progress.render(..))`; the bar styles are dropped by `ColorSystem::None` under `NO_COLOR`.
- t08: `NO_COLOR` selects a markup string with `[bold]` only; `FORCE_COLOR` is read by the program.
- t01, t02, t04, t05, t07: no workaround exists inside the crate. A program could draw the heavy-head box, wrap the text or indent the tree itself, which is a different output path from "the crate's table facility" and was not written.

## Reproduce

```bash
cd bench/candidates/richrs && cargo build --release          # adapters and t01..t08
cd ../.. && R=results/richrs
candidates/richrs/target/release/cases_runner cases/correctness.jsonl $R/correctness
.venv/bin/python scripts/compare.py $R/correctness
.venv/bin/python candidates/richrs/analysis/classify.py $R/correctness --write
candidates/richrs/target/release/width_runner $R/width && .venv/bin/python scripts/width_check.py $R/width
.venv/bin/python scripts/capability.py check -- candidates/richrs/target/release/cap
.venv/bin/python scripts/speed.py verify S3 -- candidates/richrs/target/release/bench
for t in 01-table 02-panel 03-progress 04-tree 05-error 06-markup 07-pipe 08-env; do
  .venv/bin/python scripts/tasks.py check t$t -- candidates/richrs/target/release/t${t%%-*}; done
```
