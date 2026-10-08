# Capability matrix

40 cells: depth (4) x environment variable (5) x stream (2). Defined in `scripts/capability.py`, listed in `cases/capability_matrix.json` with the expectation of every cell, written from the standards before any run.

## Test program

The one-line adapter every candidate provides: print `x` in bold and `#ff8800`, then a newline, through the candidate's default console with no explicit overrides.

## Dimensions

| depth | environment |
|---|---|
| `truecolor` | `TERM=xterm-256color COLORTERM=truecolor` |
| `256` | `TERM=xterm-256color` |
| `16` | `TERM=xterm` |
| `none` | `TERM=dumb` |

Variable (one per cell): unset, `NO_COLOR=1`, `FORCE_COLOR=1`, `CLICOLOR=0`, `CLICOLOR_FORCE=1`. Stream: stdout is a pseudo-terminal (100x24) or a pipe. Nothing else is inherited from the harness environment except `PATH`, `HOME` and `LANG`.

## Expectation rules

1. Color is on when stdout is a terminal.
2. `FORCE_COLOR` or `CLICOLOR_FORCE` turns it on for a pipe too.
3. `CLICOLOR=0` turns it off even on a terminal (bixense.com/clicolors).
4. `TERM=dumb` has no color depth, so color stays off even when forced. The standards are silent here; the cell is marked `standards_silent` and follows Rich.
5. `NO_COLOR` removes color and keeps bold (no-color.org) when output is a terminal; when color is off for any other reason there are no escape sequences at all.
6. When color is on, the depth follows `COLORTERM` then `TERM` (`truecolor`, 256-color, 16-color).

## Observation

The checker parses the SGR sequences in the captured bytes and classifies `{escapes, bold, color_class}`, `color_class` being `truecolor` (`38;2`), `256` (`38;5`), `standard` (30-37, 90-97) or `none`. It does not compare bytes: which SGR spelling a candidate uses is not the question here; exactness of color downgrade is measured by the correctness corpus.

## Rich cross-check

Rich 15.0.0 matches 34 of 40. The 6 mismatches are exactly `CLICOLOR=0` on a terminal and `CLICOLOR_FORCE=1` on a pipe (3 depths each): Rich does not read those variables. They are `reference_quirk`s: the written expectation stands, so a candidate that honors `CLICOLOR` scores them as passes. Record: `golden/capability/rich_cross_check.json`.
