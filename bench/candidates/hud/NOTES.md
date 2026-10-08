# hud 0.1.0: adapter notes

- **Capability adapter and Style.** v0.1 has no `Style` or `Console` (v0.2), so `cap.rs` reads
  `hud::capabilities(Stream::Stdout)` with no overrides and writes the SGR sequences itself
  (bold `1`, `38;2;255;136;0`, `38;5;208` or `33`). The 40 cells measure the resolver
  decisions and the depth-to-class mapping; the byte spelling is a v0.2 criterion (correctness
  cases). When `Style` exists, `cap.rs` becomes the one-line `Console` print the spec describes.
- **Width runner.** Calls `hud_width::cell_width`, `fold` and `truncate` directly. `fold` is the
  reference semantics of `spec/width.md` (greedy, a cluster wider than `w` alone on its line,
  zero-width clusters stay, empty input is one empty line): `not_greedy` is 0, so even the
  informational check passes.
- **Tables (v0.3).** `width_check.py` reads `<dir>/tables/<id>.ansi`; for hud that is the output of
  `cases_runner cases/table_unicode.jsonl`, copied into `results/hud/width/tables/` (see RESULTS.md,
  Reproduce). No script changed.
- **The four width disagreements** are strings with a ZWJ between letters (`D-001`). hud
  follows UAX #29 (a letter after a ZWJ is its own cluster); Rich skips the character after any
  ZWJ.
- **Harness changes made for hud** (`bench/CHANGELOG.md`, entries 7 to 9): `docs_coverage.py`
  runs doctests in place for workspace members (the pristine copy cannot inherit workspace
  settings); the sweep script and tool are additions.
- **Scripts not run** because hud claims nothing for them yet: `compare.py`, `tasks.py`,
  `loc.py`, `speed.py`, `api_surface.py` (no widgets to measure). Fuzz mode is a v0.2 harness part.
- **v0.3 API choices** (what an agent writing against the docs meets): `Table::new()` with chained
  `.column(...)` and `.row([...])` and, for loops, `add_column` and `add_row` (Rich's names); cells,
  headers, title and caption are markup strings; `Column::new(header)` takes `.justify(...)`,
  `.style(Style)`, `.no_wrap(bool)`, `.overflow(...)`, `.width/.min_width/.max_width(usize)`. The box
  is `BoxStyle` through `.box_style(...)` because `box` is a Rust keyword, so Rich's `box` and
  `box.ROUNDED` have no counterpart by name. A column style is a typed `Style`, not a string, so
  the Rich call `style="green"` is `.style(Style::new().color(Color::GREEN))`. Both choices cost name
  parity and are weighed at v0.6, not here.
- **Known gaps against Rich's `Table`**: DEVIATIONS D-031 (expand, ratio, table width, padding,
  header or footer toggles, row styles, nested renderables).

