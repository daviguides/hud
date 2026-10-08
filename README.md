# hud

Functional terminal UX for Rust: capability detection, cell width, styled output, tables, panels, trees, progress, live displays, layouts, status spinners and error reports, as rich text, plain text or JSON, built to be read at a glance.

**Status: 1.0.** The public API is frozen under semantic versioning; see [STABILITY.md](STABILITY.md). The crates are `hud` and `hud-width`, released together.

## Install

```sh
cargo add hud
```

or, in `Cargo.toml`:

```toml
[dependencies]
hud = "1"
```

The crate needs Rust 1.85 or newer (edition 2024) and has no required features. `hud-width`, the cell width and grapheme-cluster crate underneath it, builds without `std` and without dependencies.

## What it does

- **`hud-width`**: terminal cell width and grapheme-cluster segmentation (UAX #29, Unicode 17). `fold`, `truncate` and `pad` cut only between clusters. `no_std`, no dependencies.
- **`hud`**: styled text. `Style` parses `"bold red on #223344"`, markup (`[bold]ok[/]`, nested tags, `\[` for a bracket) builds a `Text`, and a `Console` prints it wrapped to the terminal width, with tabs, justification and overflow, in the colors the terminal has (truecolor, 256, 16 or none; `NO_COLOR` removes color and keeps bold). Output is byte for byte what Python Rich writes for the same markup, checked against its goldens and thousands of random vectors.
- **`hud`**: `Table`, `Panel` and `Tree` with Rich's boxes, padding and width arrangement, aligned on cells, never on characters; `Progress` with the bar, percentage, count, elapsed, remaining and spinner columns, handles you can clone and advance from any thread, and a display that redraws in place on a terminal and prints once anywhere else; `Group` stacks renderables, and `ErrorReport` prints an error with its causes and a hint, from any `std::error::Error`. Every one is byte for byte what Python Rich writes, checked against its goldens and thousands of random vectors.
- **`hud`**: `Columns` arranges any items in as many columns as fit, `Layout` divides the terminal into rows and columns by size and ratio, and `Live` redraws any renderable in place; `Live` and `Progress` write the same bytes Rich writes, frame by frame (corpus 3, and thousands of random vectors).
- **`hud`**: `Status` shows a message with a spinner next to it while something runs and erases itself when it ends, and `Spinner` is the frame on its own; 73 animations, byte for byte what Rich writes (corpus 5, and thousands of random vectors with a fixed clock).
- **`hud`**: one print call, three formats. `Console::print` follows the console's `Format`: rich (the styled text above), plain (no escape sequence at all) or JSON, a document `hud/1` that says what a value contains and never how it looks (no style, width or wrapping, the same bytes on every console, valid under `crates/hud/schema/hud-1.json`). Choose it in code with `ConsoleBuilder::format`, or from the environment with `HUD_FORMAT` set to `rich`, `plain` or `json`, so the same program serves a person and a script.
- **`hud`**: one resolver decides what a stream can show (color depth, `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, `TERM=dumb`, pipes, terminal size), with the precedence written in one place.

```rust
hud::println!("[bold]build[/] [green]ok[/] in [yellow]3.2s[/]");
let warning = hud::Style::new().bold().color(hud::Color::YELLOW);
assert_eq!(warning, "bold yellow".parse().unwrap());
```

```rust
use hud::{Column, Console, Justify, Panel, Table};

let table = Table::new()
    .title("Build report")
    .column("Crate")
    .column(Column::new("Downloads").justify(Justify::Right))
    .row(["clap", "12,400,000"])
    .row(["serde", "[green]98,300,000[/]"]);
Console::stdout().print(&Panel::new(table).title("Notice"));
```

```rust
use hud::{Console, Format, Tree};

let tree = Tree::new("hud").child(Tree::new("src")).child(Tree::new("Cargo.toml"));
let console = Console::builder().width(40).build();
let for_a_log = console.render_as(&tree, Format::Plain);
let for_a_program = console.render_as(&tree, Format::Json);
assert!(!for_a_log.contains('\x1b'));
assert!(for_a_program.contains("\"type\": \"tree\""));
```

```rust
let progress = hud::Progress::new();
let task = progress.add_task("Downloading", 100);
for _ in 0..100 {
    task.advance(1);
}
```

```rust
use hud::{Columns, Console, Layout, Panel};

let columns = Columns::new(["clap", "serde", "tokio", "hyper", "rayon", "regex"]);
let console = Console::builder().width(40).height(6).plain().build();
println!("{}", console.render_to_plain(&columns));

let layout = Layout::column([
    Layout::new(Panel::new("[bold]hud[/]")).name("header").size(3),
    Layout::row([Layout::new(columns), Layout::new("a second pane")]).name("body"),
]);
console.print(&layout);
```

```rust,no_run
use hud::{Live, Table};

let live = Live::new("starting");
live.start();
for step in 1..=3 {
    live.update(Table::new().column("step").row([step.to_string()]));
    live.refresh();
}
live.stop();
```

```rust,no_run
use hud::Status;

let status = Status::new("[bold]fetching[/] the index");
status.start();
// ... work ...
status.update().text("unpacking").spinner("line");
// ... more work ...
status.stop();
```

```rust,no_run
use std::process::ExitCode;

fn run() -> Result<(), std::io::Error> {
    std::fs::read_to_string("/etc/hud/config.toml").map(|_| ())
}

fn main() -> ExitCode {
    hud::report(run())
}
```

```rust
use hud::{ColorSystem, Stream, capabilities, cell_width, truncate};

let caps = capabilities(Stream::Stdout);
if caps.color_system == ColorSystem::TrueColor {
    // emit 24-bit color
}
assert_eq!(cell_width("你好🇧🇷"), 6);
assert_eq!(truncate("你好世界", 5), "你好");
```

## Examples

One runnable example per widget, and a gallery that shows them together:

```bash
cargo run -p hud --example gallery
cargo run -p hud --example table      # also: panel, tree, progress, error, markup
cargo run -p hud --example pipe | cat # the same table without escape sequences
NO_COLOR=1 cargo run -p hud --example env
```

## Measured

Every claim is checked by the benchmark in `bench/`, with pass criteria written before it ran. Against Python Rich 15.0.0 as the reference:

- **Correctness**: 210 of 210 cases byte for byte (style, markup, table, panel, tree, progress, error report).
- **Width**: 496 of 500 strings agree with Rich; the 4 that differ are a zero-width joiner between letters, where the Unicode standard breaks the cluster and Rich does not (`DEVIATIONS.md`, D-001).
- **Clusters**: 0 splits in 20,000 fold and 20,000 truncate cases, and the whole of Unicode 17's `GraphemeBreakTest` passes.
- **Capabilities**: 40 of 40 cells of the environment and terminal matrix.
- **Robustness**: 80,000 generated inputs per feature, 0 panics.
- **Tasks**: the 8 canonical tasks (table, panel, progress, tree, error, markup, pipe, environment) are solved in 11 lines of code at the median, against 12 for Python Rich.

The harness, goldens and the other candidates measured the same way are in `bench/`; see `bench/PILOT-RESULTS.md`.

## Goals

- **Speed.** Fast startup and fast render, measured against existing Rust crates and Python Rich.
- **Correctness.** ANSI output verified byte for byte against golden captures of Python Rich.
- **Assertiveness.** Correct text width for Unicode (CJK, emoji, combining characters), wrapping and truncation that never split a grapheme, and capability detection that works without wiring.
- **DX.** An API a developer, or an LLM with only the docs, gets right on the first try.

## Layout

```text
crates/hud-width/   cell width, clusters, fold, truncate, pad (no_std)
crates/hud/         capabilities, style, markup, text, console, table, panel, tree, progress, error report, columns, layout and live
xtask/              cargo xtask gen-width | check-layers
bench/              corpus, goldens, harness, candidate adapters
DEVIATIONS.md       every deliberate difference from Rich
```

```bash
cargo test --workspace
cargo xtask gen-width      # regenerate the Unicode tables from the pinned UCD files
cargo xtask check-layers   # enforce the layer rules inside the hud crate
```

## Contributing

See `CONTRIBUTING.md` for how to build, test and propose a change, `SECURITY.md` to report a vulnerability, and `RELEASING.md` for what a stable release involves.

## License

MIT OR Apache-2.0. The Unicode tables are generated from the Unicode Character Database under the Unicode License v3; see `THIRD_PARTY_NOTICES.md`.
