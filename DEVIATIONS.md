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
`truncate` cut only between clusters, by construction.

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
