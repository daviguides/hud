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
