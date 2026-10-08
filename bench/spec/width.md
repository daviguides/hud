# Assertiveness corpus: width, fold, truncate, tables

## Width corpus

`cases/width_corpus.jsonl`: 500 strings, `{id, category, text}`. Categories: `ascii` 30, `cjk` 70, `emoji` 61, `emoji_zwj` 50, `emoji_tone` 40, `flags` 30, `keycaps` 15, `combining` 50, `zero_width` 30, `ambiguous` 40, `control` 30, `mixed` 54. Reference cell width per string: `golden/width/width_ref.jsonl` (`ref`).

`ref` is Rich 15.0.0 `cell_len`, cross-checked against `wcwidth` 0.9.2 (control characters removed before calling it). They agree on 487 strings. The 13 disagreements, all spacing combining marks (category Mc), are hand-resolved in `width_resolutions.json`, which also records the Indic-conjunct ambiguity that was **not** resolved against either reference. Contested strings carry `status: "resolved"`; the checker reports them separately so a candidate that follows the legacy per-code-point model is visible as such.

## Fold and truncate (grapheme safety)

For every string and every width `w` in 1..40:

- `fold(text, w)`: greedy hard fold at grapheme-cluster boundaries (UAX #29 extended clusters). Each line is at most `w` cells wide. A cluster wider than `w` goes alone on its line. Zero-width clusters stay on the current line. Concatenating the lines gives back the input exactly.
- `truncate(text, w)`: the first line of `fold`.

The checker counts, per output: `grapheme_splits` (a cut inside a cluster; gate: zero), `concat_mismatch`, `line_too_wide`, and `valid_but_not_greedy` (valid but different from the reference fold, reported, not a split).

## Table alignment

`cases/table_unicode.jsonl`: 12 tables whose cells hold CJK, emoji, ZWJ sequences, combining marks and mixed text. After stripping ANSI, every line of a table must have the same cell width (by the reference width). Gate: zero misaligned rows.

## Candidate output layout

```
width.jsonl      {"id", "width"}                   500 lines
fold.jsonl       {"id", "w", "lines": [...]}       500 x 40 lines
truncate.jsonl   {"id", "w", "text"}               500 x 40 lines
tables/<id>.ansi                                    output for cases/table_unicode.jsonl
```

A missing file means unsupported, never a pass. Run `scripts/width_check.py <dir>`.
