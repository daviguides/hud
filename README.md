# hud

Functional terminal UX for Rust: capability detection, cell width and, milestone by milestone, styled output, tables, panels, trees and progress, built to be read at a glance.

**Status: in development, not published.** The name `hud` is reserved on crates.io; nothing is released until the project reaches a stable version. The crates in this workspace are marked `publish = false`.

## What works today (v0.2)

- **`hud-width`**: terminal cell width and grapheme-cluster segmentation (UAX #29, Unicode 17). `fold`, `truncate` and `pad` cut only between clusters. `no_std`, no dependencies.
- **`hud`**: styled text. `Style` parses `"bold red on #223344"`, markup (`[bold]ok[/]`, nested tags, `\[` for a bracket) builds a `Text`, and a `Console` prints it wrapped to the terminal width, with tabs, justification and overflow, in the colors the terminal has (truecolor, 256, 16 or none; `NO_COLOR` removes color and keeps bold). Output is byte for byte what Python Rich writes for the same markup, checked against its goldens and thousands of random vectors.
- **`hud`**: one resolver decides what a stream can show (color depth, `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, `TERM=dumb`, pipes, terminal size), with the precedence written in one place.

```rust
hud::println!("[bold]build[/] [green]ok[/] in [yellow]3.2s[/]");
let warning = hud::Style::new().bold().color(hud::Color::YELLOW);
assert_eq!(warning, "bold yellow".parse().unwrap());
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

## Goals

- **Speed.** Fast startup and fast render, measured against existing Rust crates and Python Rich.
- **Correctness.** ANSI output verified byte for byte against golden captures of Python Rich.
- **Assertiveness.** Correct text width for Unicode (CJK, emoji, combining characters), wrapping and truncation that never split a grapheme, and capability detection that works without wiring.
- **DX.** An API a developer, or an LLM with only the docs, gets right on the first try.

## Layout

```text
crates/hud-width/   cell width, clusters, fold, truncate, pad (no_std)
crates/hud/         capabilities, style, markup, text and console today; widgets as milestones land
xtask/              cargo xtask gen-width | check-layers
bench/              corpus, goldens, harness, candidate adapters
DEVIATIONS.md       every deliberate difference from Rich
```

```bash
cargo test --workspace
cargo xtask gen-width      # regenerate the Unicode tables from the pinned UCD files
cargo xtask check-layers   # enforce the layer rules inside the hud crate
```

## License

MIT OR Apache-2.0. The Unicode tables are generated from the Unicode Character Database under the Unicode License v3; see `THIRD_PARTY_NOTICES.md`.
