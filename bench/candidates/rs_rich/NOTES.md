# rs-rich: adapter notes, API friction, bugs, workarounds

Written while building the adapters against `rs-rich =0.0.9` (lib name `rich`). Evidence and numbers are in `RESULTS.md`.

## Adapter decisions

- **Environment injection (README adapter rule).** The core crate reads `NO_COLOR`, `COLORTERM`, `TERM`, `COLUMNS` and `LINES`, not `FORCE_COLOR` or `CLICOLOR*`. Where the schema needs `FORCE_COLOR` (`s1`, `bench` for S1 to S4, the `t08` solution, the `cap_env` variant) the adapter does `if std::env::var_os("FORCE_COLOR").is_some() { builder = builder.force_terminal(true); }`. `cap` (the capability one-liner, "no explicit overrides") does not, and scores 31/40; `cap_env` scores 34/40.
- **Correctness console** is configured only through the public builder, never from the environment: `width`, `height(24)`, `force_terminal(true)`, `color_system(Some(..)/None)`, `legacy_windows(false)`, `emoji(false)`, `highlight(false)`, `no_color(false)`. Output is `Console::render_export`, documented as "exactly as `print` would write it".
- **Error layout** (no primitive): `Panel` titled `Error`, `border_style("red")`, body a single `Text` assembled with `append(.., Some("bold"/"dim"/"cyan".into()))`.
- **Progress** uses `ProgressColumn::TextFormat(TextColumn::new("{task.description}"))`, `BarWith(BarColumn::new().bar_width(Some(n)))`, `Percentage` (or `TaskProgress { show_speed: false }`) and `MofN`. A live display is `Progress::start(console, std::io::stdout(), rate)` then `live.stop()`; a static frame is `console.print(&progress)` (`Progress` implements `Renderable`).
- **Width runner**: fold is `rich::cells::chop_cells`; truncate is the first chunk of the fold (spec definition); `cells::truncate` is recorded as a variant (`truncate_cells_fn.jsonl`).
- **Speed S3** uses `refresh_per_second = 1.0` and an explicit `live.refresh()` every 100th update, the closest to Python's `auto_refresh=False`; the crate has no way to switch the tick thread off through `Progress::start`.
- **Adoption / docs / API metrics** use the separate `hello/` and `hello_nodefault/` crates (minimal hello table), never the adapter package (which also depends on `serde_json`).

## Findings the harness owner needs

1. **Golden order dependence (harness defect, all candidates).** 28 of 222 goldens differ from an isolated Rich render because Rich caches a `Style`'s ANSI codes on first use regardless of color system. `scripts/golden_isolation_audit.py` lists them (`results/golden_isolation_audit.json`); `scripts/isolate_check.py` classifies a candidate's failures against isolated Rich. The goldens need regenerating with a fresh process or a style-cache reset per case. Until then every correct candidate fails these 28 and the `progress` feature keeps only 12 of 30 cases in the denominator.
2. **`api_surface.py json`** prints `target/doc/<crate with - as _>.json`; the rs-rich package is `rs-rich` but its lib is `rich`, so the file is `rich.json` and the printed path does not exist. Read the file name from `target/doc/*.json` (or `cargo metadata`).
3. **`docs_coverage.py`/`api_surface.py`** run from a project that depends on the crate (`hello/`); both worked there with `cargo +nightly`.
4. **Capability expectation**: Rich 15.0.0 honors `FORCE_COLOR`; rs-rich 0.0.9 does not, so a "faithful port" is not faithful on that variable.

## What an LLM agent writing against the docs is likely to get wrong

- **`use rs_rich::...`**: the package is `rs-rich` but the library is `rich` (`use rich::{Console, Table}`); `rs_rich` fails to resolve. The README example shows `use rich::...` but `cargo add rs-rich` is the natural command.
- **Highlighting is on by default** (`Console::builder().highlight` defaults to true, like upstream). Numbers such as `3.2s` get colored; the markup and table tasks need `.highlight(false)`.
- **Strings are markup.** `Table::add_row(&[&str])`, `Tree::new(&str)`, `Panel::title(&str)`, `Console::print_str` all parse `[...]`. Data with brackets must go through `Text::new(..)` (docs say so; easy to miss).
- **Base style leaks**: `Text::styled(msg, "bold")` then `append("Caused by:", Some("dim"))` renders `1;2`. Python's `Text("x", style="bold") + Text(...)` habit, or a `Group`, is not the Rust shape. This cost one failing pass of 20 cases before it was found.
- **No `Group`**: the equivalent is `containers::Renderables`, which takes `Vec<Arc<dyn Renderable + Send + Sync>>`; `Panel::new` takes `Box<dyn Renderable>`.
- **Progress columns are an enum**, not classes: `TaskProgressColumn` is `ProgressColumn::TaskProgress { show_speed }` (or `Percentage`), `MofNCompleteColumn` is `ProgressColumn::MofN`, a plain text column is `TextFormat(TextColumn::new(..))`, a sized bar is `BarWith(BarColumn::new().bar_width(Some(30)))`.
- **Live progress needs an explicit writer and an explicit stop**: `progress.start(Console::new(), std::io::stdout(), 10.0)` returns `LiveProgress<W>`; `live.stop()` commits the last frame and ends the refresh thread.
- **`Table::add_row` takes `&[&str]`**, so computed cells need a temporary: `table.add_row(&[a, b, &format!("[green]{s}[/]")])`.
- **Builder methods consume `self`, column and row methods take `&mut self`**: `Table::new().title(..).header_style(..)` then `let mut table`, `table.add_column(..)`; mixing the two orders fails to compile.
- **Width comes from `COLUMNS` or the terminal, never from a pipe**: piped runs without `COLUMNS` fall back to 80 (`DEFAULT_WIDTH`).

## Quality signals seen while reading the source

- Strengths: byte-identical to isolated Rich on 222/222 cases; width agreement 500/500 including the 13 contested Mc strings; documented divergences; `render_export` and `capture` make output testable; `Console::builder` makes every output reproducible.
- 984 public items, 84.3% documented, only 4 with a doc example; the README shows one snippet.
- 1336 public functions; several take 4 to 7 positional arguments with `Option`s (`LogRender::render`, `Status::update`), mirroring Python's keyword arguments without defaults.
- Binary cost is mostly the crate itself, not its dependencies: +2.26 MB without `syntect`/`pulldown-cmark` (21 crates) and +2.41 MB with them (50 crates).
- `chop_cells` splits flags and Indic conjuncts and returns an empty leading chunk for a grapheme wider than the width (Rich's behavior, confirmed in Python); not safe for a grapheme-exact fold or truncate.
- Not examined: fuzz behavior, panics (79 `unsafe` mentions workspace-wide per `references/studies/rs-rich.md`; none used by the code paths of the adapters).
