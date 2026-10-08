# Deviations

Every deliberate difference between hud and its reference (Python Rich 15.0.0, pinned in
`bench/uv.lock`) or between hud and the architecture document (`foundation/architecture.md` in the project knowledge base). A difference found by a
failing golden before it is listed here is a bug. Each entry has a reason and the condition
for removing it.

## Width and segmentation (`hud-width`)

### D-001: ZWJ between characters that are not emoji

Rich skips the code point that follows any U+200D ZERO WIDTH JOINER, so `pa‍rser` is
5 cells. UAX #29 rule GB11 joins only an Extended_Pictographic after a ZWJ; a letter after a
ZWJ is a separate cluster, and a terminal that clusters graphemes (mode 2027) shows 6
cells. hud follows the standard.

- Corpus strings: `w-0348`, `w-0356`, `w-0358`, `w-0360` (hud 6, 6, 5, 5; Rich 5, 5, 4, 4).
- Evidence (v0.2): hud is right and Rich is wrong, for text that is not an emoji sequence.
  The UCD 17.0.0 `GraphemeBreakTest.txt`, which hud passes in full, has the case on line 752:
  `÷ 0646 × 200D ÷ 0020 ÷`: a ZWJ is joined to what comes before it (rule 9.0) and there is a
  boundary after it when the next character is not a pictograph (rule 999.0). A terminal that
  clusters graphemes (mode 2027) therefore draws the letter after the ZWJ in a cell of its own,
  and one that does not draws it there too, because the ZWJ is zero cells and the letter is
  one. No terminal removes the letter. Rich's `cell_len` skips the character after any ZWJ, and
  so does `wcwidth` 0.9.2 (`wcswidth("pa\u200drser")` is 5, like Rich), so the two Python
  implementations agree with each other and with the emoji case (a ZWJ family is 2 cells in all
  of them) but not with the standard for `letter ZWJ letter`.
- Effect on the gate: 496 of 500 strings agree with Rich (99.2%).
- Remove when: Rich changes its handling, or a terminal is shown to swallow the character.

### D-002: Kirat Rai vowel signs

U+16D63 and U+16D67 to U+16D6A have `Grapheme_Cluster_Break=V` in Unicode 17 and join the
cluster they follow, so hud counts them as 0 cells. Rich counts 1. Not in the corpus.

- Remove when: the Unicode data stops classifying them as `V`, or terminals draw them in their own cell.

### D-002b: reserved Hangul Jamo Extended-B code points

U+D7C7 to U+D7CA and U+D7FC to U+D7FF are unassigned but reserved for Hangul vowels and
trailing consonants; the UCD gives them `V` and `T`, so hud counts 0 cells where Rich counts 1.
No character is assigned there.

- Remove when: characters are assigned there with a different classification.

### D-003: fold and truncate never split a cluster

Rich's `chop_cells` can cut inside flags and Indic conjuncts. `hud_width::fold` and
`truncate` cut only between clusters, by construction. The text printer folds long words with
`fold`, so a flag at the fold stays whole where Rich puts one half on each line.

- Remove when: never. This is the assertiveness gate.

### D-004: Unicode version

Tables are generated from UCD 17.0.0 (`xtask/data/ucd-17.0.0`, checksums in `SHA256SUMS`).
Changing the version is a dated entry here and a regeneration with `cargo xtask gen-width`.

## Terminal capabilities (`hud`)

### D-010: `CLICOLOR` and `CLICOLOR_FORCE` are honored

Rich ignores both (34 of 40 cells of the capability matrix). hud follows the written
expectation in `bench/spec/capability.md`: 40 of 40.

### D-011: `NO_COLOR` removes color and keeps attributes

On a terminal, `NO_COLOR` removes color only; bold, italic and underline stay
(no-color.org). When styling is off for any other reason, no escape sequences are emitted.

### D-012: forcing color on `TERM=dumb`

`FORCE_COLOR` and `CLICOLOR_FORCE` do not enable styling on `TERM=dumb`: that terminal has
no color depth. The standards are silent; this follows Rich.

## Style, markup and text (`hud`)

### D-020: hyperlinks carry no id

Rich writes `ESC ] 8 ; id=<random> ; url ESC \`, so the bytes change from run to run. hud writes
`ESC ] 8 ; ; url ESC \` (no id), which is valid and deterministic. A link wrapped over two
lines is two links; the id only tied them together in the terminal.

- Remove when: a multi-line link needs to stay one hover target; a stable id derived from the URL and the offset would then be added.

### D-021: markup that does not parse prints as written

Rich raises `MarkupError` when a closing tag has nothing to close. `hud::println!` and
`Console::print` never fail and never panic: the string is printed literally. `Text::from_markup`
returns the error for callers that want it.

### D-022: emoji codes are not replaced

Rich replaces `:smile:` with the emoji in markup. hud prints the text as written; the corpus
configures Rich with the replacement off for the same reason.

- Remove when: a user asks for it; it would be an opt-in on `Text::from_markup`.

### D-023: theme style names are not known

Rich resolves names of its default theme (`[repr.number]`, `[strong]`, `[reset]`, ...) as well as
style strings. hud knows only style strings. An unknown name is a style that changes nothing and
its text is kept, which is what Rich does for names missing from its theme.

### D-024: widths count cells, not characters

Where Rich compares the number of characters with a width (`rstrip_end`, which cuts the
whitespace a line has beyond the width), hud compares cells. The two agree on text where every
character is one cell. With a combining mark (`e` followed by U+0301 is two characters and one
cell) Rich cuts a cell too many and the line ends one cell short of the width, which also changes
how `full` justification spreads spaces. With a wide character (a CJK, fullwidth or emoji
character is one character and two cells) Rich cuts too little: a word that exactly fits a
table column and is followed by a space keeps the space, overflows by one cell and gets an
ellipsis (`한국 …` where hud prints `한국어`). Measured over 900 random Unicode markup vectors:
10 differ, every one with a combining mark or a flag in the input; over 600 random Unicode table
vectors: 49 differ, 43 with a flag or a combining mark and 6 with a wide character, and in each
of those 6 the line Rich prints has `…` where hud's does not (`tests/oracle.rs`).

- Remove when: Rich counts cells there.

### D-025: `escape` also protects a backslash before a bracket that starts no tag

Rich's `escape` leaves `\[x` alone and prints `[x`, because its markup parser eats any backslash
before a bracket. hud's `escape` adds one, so `Text::from_markup(&escape(s)).plain() == s` for
any `s` that does not end in a backslash (checked by the fuzz mode).

### D-026: a `Text` keeps its own justify, overflow, no_wrap, tab size and end when printed

Rich's `Console.print` joins plain `Text` arguments into a new one, which drops those settings,
and wraps `justify=` in an `Align` of the whole block. hud prints a `Text` as the renderable it
is: its settings apply, and `Console::print` has no justify argument. The differential vectors
render through a `Group` in Rich for the same reason.

### D-027: a span that starts or ends inside a grapheme cluster covers the whole cluster

Rich splits the cluster between two runs, so the line is measured and written in pieces that
disagree with the whole (a base letter and its combining mark can end up in different escape
sequences). hud moves the span edges to the cluster edges, so a run never holds half a cluster
and the widths of the runs add up to the width of the line. Only text with a tag in the middle
of a cluster is affected.

- Remove when: never; it follows from the assertiveness gate.

### D-030: color names and palettes are data from Rich

The 235 color names and the standard and 256-color palettes in `model/palette.rs` are extracted
from Rich 15.0.0, with its license notice in `THIRD_PARTY_NOTICES.md`, because they are the
interface that makes a name written for Rich work here and the downgrade identical. No code is
copied.

- Remove when: never needed to remove; regenerate with `bench/scripts/gen_palette.py` if the pinned Rich changes.

### D-031: what `Table` does not do yet

Not claimed in v0.3, because no case, task or workload needs it: `expand` and column `ratio`,
a table `width` or `min_width`, `padding`, `pad_edge`, `collapse_padding`, `show_header`,
`show_footer`, `show_edge`, `leading`, row styles and sections, a vertical alignment other than
bottom for the header and top for the rows, a cell that is not a markup string (nested tables,
panels), and the substitution of an ASCII box when the terminal is not UTF-8 (hud has no encoding
detection). The ones Rich has and the roadmap puts later are listed in `features.md`; a table
built in hud ignores none of them silently because there is no API for them.

- Remove when: each option lands with its own goldens.

### D-032: a cell with markup that does not parse prints as written

Rich raises `MarkupError` from `Table.add_row` or at print time. hud prints the string as it is,
the same rule as D-021 for text, so a table never fails to print because of one cell.

- Remove when: never; same decision as D-021.

### D-033: what `Panel` and `Tree` do not do yet

Not claimed in v0.4, because no case, task or workload needs it. `Panel`: a panel `style`, a
`width` or `height`, and the ASCII box substitution when the terminal is not UTF-8 (hud has no
encoding detection). `Tree`: `hide_root`, `expanded` (collapsed branches), a per-node `style`,
labels that are not markup strings (panels, tables or any renderable as a label), the highlight of
labels, and the ASCII guides for a non-UTF-8 terminal. A panel or tree built in hud ignores none
of them silently because there is no API for them. The tree guide style is set on the root and on
any node (`Tree::guide_style`), inherited downward as Rich does.

- Remove when: each option lands with its own goldens.

### D-034: no room, no lines

When the width left for a panel body or a tree label is zero (a console of 0 or 1 cell, a panel
whose padding takes the whole width, a tree node nested deeper than the width allows), Rich renders
nothing into it, so the panel prints only its edges and the tree node prints no line. hud does the
same. The vectors that found it were 134 of 1 200 ASCII trees and most ASCII panels at narrow
widths.

- Remove when: never. It is Rich's behavior and the printed rows stay inside the terminal.

### D-035: a panel with markup that does not parse prints as written

A body, a title or a subtitle with a markup error: Rich raises `MarkupError` at construction or
print time, hud prints the string as it is (the rule of D-021 and D-032).

- Remove when: hud returns the error from a fallible API.

### D-036: what `Progress` does not do yet

Not claimed, listed so an unclaimed feature is never a pass: tasks without a total (the pulsing
bar), `visible`, `start=False` and `stop_task`, task `fields`, the speed, file size and download
columns, `Progress.update(description=..., total=..., advance=...)` as one call (hud has
`Task::set_description`, `Task::set_total`, `Task::advance`), and printing other output while a
display is live (Rich's render hook: the display and a `println!` interleave). The template of a
`TextColumn` knows `{task.description}`, `{task.completed}` and `{task.total}` only; Rich formats any
task attribute with a format spec. `track` takes an exact-size iterator and does not take Rich's
`total`, `update_period` or `transient` arguments (`ProgressBuilder::transient` does the last).

- Remove when: a case in the corpus needs one of them.

### D-037: the bytes of a live display (resolved on the live-layout branch)

Until this branch a display drew each frame with a newline after every line and put the next one over it
with `CR`, `ESC [ n A` and `ESC [ J`; its bytes were not Rich's, only its screens were. `Live` and `Progress`
now share one redraw protocol (`services/live.rs`) that writes what Rich writes: the cursor hidden at the
start, each frame preceded by `CR`, `ESC [ 2 K` and `ESC [ 1 A ESC [ 2 K` per earlier line, its lines
separated by newlines with none after the last, a newline and `ESC [ ? 25 h` at the end, and the erase of a
transient display after that. A frame taller than the terminal keeps one line less and ends with `...`
centered in bold red (Rich's `live.ellipsis`). Verified on corpus 3: `live` 30 of 30 and `progress_live` 12 of
12 byte-identical, and 1 100 ASCII random vectors of each (`oracle_live_layout.rs`). What still differs from
Rich is listed in D-044.

- Remove when: kept as the record of why the screens of the earlier milestones were compared and not the bytes.

### D-038: steps are whole numbers, estimates never go below zero

`completed` and `total` are `u64`; Rich accepts floats, so a fractional step does not exist and
`completed` cannot be negative. A task whose `completed` is above its `total` has a negative
`remaining` in Rich and the time remaining prints a negative duration; hud prints `0:00:00`. The
oracle vectors never exceed the total for this reason. `u64::MAX` steps are drawn without
overflow (fuzz).

- Remove when: a user needs fractional steps (an `f64` total would change the public types).

### D-039: the width of the bar is computed from an exact product

Rich computes the filled half cells as `int(width * 2 * completed / total)`, an integer true
division that Python rounds once. hud forms the exact product in `u128` and divides in `f64`, which
rounds twice. The two differ only when `width * 2 * completed` is above 2^53, that is, for a total
above about 10^14 steps; no corpus case or oracle vector reaches it.

- Remove when: never needed.

### D-040: `ErrorReport` is a fixed layout, not a Rich widget

Rich has no error widget; the reference layout is a composition of Rich primitives fixed in
`bench/spec/case-schema.md` (a red rounded panel titled `Error`, the message in bold, `Caused by:`
in dim over causes numbered from 0 and indented four spaces, `hint:` in cyan). hud implements that
layout and nothing else. The message, the causes and the hint are plain text, never markup, as in
the reference (`Text(...)`), so a bracket in an error prints as written. Not claimed: options for
the title, border or colors, a custom `hint:` prefix, source context lines and backtraces.

- Remove when: the API audit of v0.7 decides which options a report needs.

### D-041: `from_error` keeps a cause that repeats its parent

`ErrorReport::from_error` walks `source()` and prints every error in the chain. An error whose
`Display` already includes its source (`"{self}: {source}"`) shows the same text in the message and
again as a cause. Dropping text on a guess is worse than repeating it, so nothing is deduplicated.

- Remove when: never.

### D-042: `hud::report` returns an `ExitCode`

A `main` that returns `Result<(), E>` makes the standard library print `Error: {E:?}` in front of
the Debug text, which would put seven characters before the top border of the panel. `hud::report`
renders the report to standard error and returns the exit code instead. `ExitCode` lives in
`std::process`, so the `xtask check-layers` rule against child processes now matches
`process::Command` and `Command::new` instead of the whole `std::process` path; the rule still
forbids every way of starting a child process.

- Remove when: `Termination` can be implemented for a custom error type with a clean output.

### D-043: `Group` has no `fit` option

Rich's `Group(*renderables, fit=True)` takes a `fit` flag. hud's `Group` is built with `push`, renders
its items in order and measures as the widest minimum and the widest maximum of its items; it has
no `fit` option. The error report uses it with the layout above and matches Rich on every vector.

- Remove when: a user needs `fit=False`.

### D-044: what `Live`, `Layout` and `Columns` do not do yet

`Live` has no alternate screen (`screen=True`), no redirection of standard output and error (`redirect_stdout`,
`redirect_stderr`), no nested displays (a second `Live` on the same console draws over the first) and no
`get_renderable` callback; replace what it shows with `update`. A `Progress` starts drawing at its first
task, where Rich's `start()` already hides the cursor: a `Progress` that ends with no task writes nothing, and
Rich writes the cursor sequences and a line break (hud draws the empty frame Rich draws, one blank line, in front of the
first task so the bytes of every other case are Rich's). `Layout` has no placeholder (a leaf with no
renderable is blank where Rich draws a titled panel with a `repr` of the layout), no `tree`, no
`refresh_screen`, no custom splitters, and a layout whose children are all invisible is blank (Rich recurses
forever). A `Layout` placed inside another layout as the renderable of a leaf takes the height of its region,
as in Rich; a layout printed on its own, or inside a `Panel` or a `Group`, is as tall as the standard output
terminal, not as the console it is printed on (the renderable has no console height to ask for). `Columns`
holds markup, `Text`, tables, trees, panels and any renderable; a `Text` with a base style of its own keeps its
padding unstyled in a cell. When `width` is so small that no column fits, hud uses one column and Rich raises
a `ValueError`. `Group` items do not receive the height of a region.

- Remove when: a case in the corpus needs one of them.

## Known gaps against the architecture document (`foundation/architecture.md` in the project knowledge base)

- **Windows.** v0.1 resolves the size from `COLUMNS` and `LINES` and falls back to 80x24; it
  does not enable virtual terminal processing. Both need platform calls that are not
  implemented yet and cannot be verified on the development machine. No Windows build target
  has been exercised.
- **Release.** The v0.1 milestone text asks for crates.io releases of `hud-width` and `hud`.
  By decision of the project owner nothing is published before a stable version; both crates
  carry `publish = false`.
- **`cargo deny`.** The license allowlist and the `atty` ban of the `check` job are not wired
  yet; the dependency tree of `hud` is `rustix`, `bitflags`, `errno` and `libc`.
