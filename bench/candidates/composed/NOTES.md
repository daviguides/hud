# Composed baseline: notes

What the composition lacks, the glue it needs, API friction, and what an agent is likely to get wrong. Numbers are in `RESULTS.md`; this file is the reading of them.

## What the three crates do not provide

| Needed | In the composition | Glue written | File |
|---|---|---|---|
| Inline markup (`[bold red]x[/]`, escapes, unclosed tags) | nothing | tag parser, style stack | `text.rs` |
| Rich style strings (`bold red on #223344`, `color(17)`, `rgb()`) | `owo_colors::Style` takes typed calls only | parser | `style.rs` |
| Colour downgrade truecolor to 256 to 16 | none: `owo-colors` and `console` write what they are given | Rich's HLS grey rule, 6-level cube, redmean palette match | `style.rs` |
| Word wrap that respects styled spans | `comfy-table` wraps inside a cell only | Rich's `divide_line` port over styled cells | `text.rs` |
| Panel with title in the border, nesting, padding | none | `render_panel` | `render.rs` |
| Tree with guides | none | `render_tree`, including heavy guides for a bold guide style | `render.rs` |
| Error layout | none | panel plus lines (`render_error`) | `render.rs` |
| Table title and caption | `comfy-table` has neither | centred painted lines around `table.lines()` | `render.rs` |
| Terminal width when stdout is a pipe | `comfy-table` reads the tty only; `console` too | `COLUMNS` read by hand in every task | each `tNN.rs` |
| One frame of progress as a string | `indicatif` draws to a terminal | a `TermLike` that captures, plus Rich's column-width collapse | `progress.rs` |
| Cell width of controls, soft hyphen, ZWJ runs | `unicode-width` counts controls as 1 | not written; costs 30 of 61 width misses | none |
| Capability resolution | three detectors, see below | `cap-full`, 29 lines, only for the probe | `bin/cap-full.rs` |

Glue library total: 1 025 rustfmt LOC (`style.rs` 262, `text.rs` 209, `render.rs` 421, `progress.rs` 133) to reach 67.0% byte-identical and 79.1% screen-equal on the corpus. Inside the 8 DX tasks the glue is inline and counted in each file (median 37 LOC against 12 for Rich).

## Three crates, three capability policies

| Crate | Reads | Ignores | Behaviour |
|---|---|---|---|
| `comfy-table` (`tty` feature) | whether stdout is a terminal | `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR*`, `COLORTERM`, `TERM` | cell styling only on a terminal unless `enforce_styling()`; no width detection off a terminal |
| `owo-colors` `if_supports_color` via `supports-color` | `FORCE_COLOR`, `CLICOLOR_FORCE`, `NO_COLOR`, `TERM=dumb`, `COLORTERM`, `TERM`, tty | `CLICOLOR=0` | `FORCE_COLOR=1` means level 1 (16 colours) regardless of `COLORTERM`; `NO_COLOR` also drops bold; closure cannot see the level |
| `indicatif` via `console` | `CLICOLOR`, `CLICOLOR_FORCE`, `NO_COLOR`, tty | `FORCE_COLOR`, `COLORTERM` for depth | no 16-colour mode; bright colours written as `38;5;n+8`; truecolor always written as `38;2` |

A program using all three has no single answer to "is colour on". `t07` passes only because `comfy-table`'s tty rule happens to match the task; `t08` needs a hand-written precedence block because `if_supports_color` cannot keep bold under `NO_COLOR`.

## API friction found

- `owo-colors`: chaining inside the `if_supports_color` closure does not compile. `|t| t.bold().truecolor(255, 136, 0)` fails with E0515 ("returns a value referencing data owned by the current function"). The working form is `|t| t.style(Style::new().bold().truecolor(255, 136, 0))`. The crate's own docs show only a single call in the closure.
- `owo-colors`: `Style` writes colour parameters before effects, so `bold red` is `31;1` where Rich writes `1;31`. No option; every multi-attribute style differs in bytes (22 of 60 correctness failures).
- `comfy-table`: no heavy-header preset, so Rich's default box needs a custom `TableStyle` (six builder calls) in every program that wants it. `set_cell_alignment(Center)` puts the odd cell on the left. Cell content is trimmed. `ContentArrangement::Dynamic` needs `set_width`, and off a terminal nothing supplies it.
- `comfy-table` with `custom_styling` is required to put ANSI text in a cell and measure it, which pulls `ansi-str` and `console`; without it, styled cells from markup mis-measure.
- `indicatif`: `ProgressBar::new` draws to stderr and hides itself when stderr is not a terminal; a task that captures stdout shows nothing. `ProgressDrawTarget::stdout()` is the fix, and it hides off a terminal too, so a pipe needs a custom `TermLike`. `finish()` sets the position to the length: an incomplete bar needs `abandon()`. `{percent}` truncates where Rich rounds (999/1000 shows 99%, Rich 100%). The template string is a runtime-parsed mini-language with `.unwrap()`.
- `unicode-width` `str::width` and the Rich/wcwidth convention disagree on controls (1 vs 0), soft hyphen and ZWJ among ASCII.
- All three: no name overlaps with Rich beyond `Table`, `Style`, `Color`, `Column`, so an agent that knows Rich cannot guess the API (5 of 40 names).

## What an LLM agent is likely to get wrong

1. Writing `bar.finish()` for a bar that must stay partial, which fills it.
2. Using the default `ProgressBar::new(len)` and expecting output on stdout.
3. Chaining effects inside `if_supports_color` (compile error above), or using it and expecting `NO_COLOR` to keep bold.
4. Expecting `comfy-table` to read `COLUMNS` or to style under `FORCE_COLOR`.
5. Expecting a title, caption or panel to exist (none do), and so writing a literal-string frame.
6. Centering: assuming Rich's rounding.
7. Passing `Cell::new("regex ")` and expecting the trailing space to count in right alignment.

These are hypotheses from reading the crates and from the compile and runtime failures met here; the first-try protocol (not run) is what would turn them into rates.

## Judgements made while writing the adapters

- Style bytes come from `owo-colors`, not from a hand-written emitter. A 6-line emitter would match Rich's parameter order and lift style and markup to 100%, but the candidate would then not use `owo-colors` for styling.
- Table cells with markup are rendered to ANSI text first and handed to `comfy-table` (`custom_styling`); native `Cell::fg` cannot express nested spans.
- The named-colour table holds the 16 ANSI names plus `grey50` (17 of Rich's roughly 250 names). The corpus uses only those; a real implementation needs the full table.
- `cap` (colour-level aware) was written after the one-liner `cap-naive` scored 20/40; both are reported because the one-liner is the "default console" the protocol asks for.
- `reference_quirk` was applied only to the 28 goldens whose fresh-process Rich render differs from the golden (`scripts/reference_quirks.py`); nothing else was reclassified, so every other failure counts against the candidate.
