# t08-env: NO_COLOR and FORCE_COLOR honored

## Statement

Print the single line "error ok plain" where "error" is bold red and "ok" is green. The same unchanged program is run three ways and each result must equal its target: (a) on a terminal with NO_COLOR=1 (no colors, bold stays: NO_COLOR removes color only, not text attributes), (b) piped with FORCE_COLOR=1 and COLORTERM=truecolor (colors present), (c) piped with neither variable (plain text, no escape bytes). No automatic highlighting of numbers or other tokens: only the styles stated here.

## Run `no_color_tty`

- Stream: stdout is a pseudo-terminal sized 100 columns x 40 rows.
- Environment: `COLORTERM=truecolor NO_COLOR=1 TERM=xterm-256color`
- Check: the final terminal screen equals the target cell by cell (character, foreground, background, bold, italic, underline, strikethrough, reverse). How the escape sequences are written does not matter.

Target screen text:

```text
error ok plain
```

Cells carrying color or attributes (every other cell is default):

- line 1: `error` : bold

## Run `force_color_pipe`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLORTERM=truecolor COLUMNS=100 FORCE_COLOR=1 TERM=xterm-256color`
- Check: the final terminal screen equals the target cell by cell (character, foreground, background, bold, italic, underline, strikethrough, reverse). How the escape sequences are written does not matter.

Target screen text:

```text
error ok plain
```

Cells carrying color or attributes (every other cell is default):

- line 1: `error` : fg red, bold
- line 1: `ok` : fg green

## Run `plain_pipe`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLORTERM=truecolor COLUMNS=100 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
error ok plain
```
