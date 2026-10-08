# t06-markup: Inline markup with nested styles

## Statement

Print three lines with inline styles. Line 1: "Deploy" in bold, "ok" in green, "3.2s" in italic yellow. Line 2: "outer inner deep outer" where the whole line is bold, "inner deep" is also italic and "deep" is also underlined. Line 3: " FAIL " in bold red on white background, then "retry" struck through, then plain " in 5s". The check compares the final screen of a 100 column truecolor terminal cell by cell (character, color, bold, italic, underline, strikethrough). No automatic highlighting of numbers or other tokens: only the styles stated here.

## Run `main`

- Stream: stdout is a pseudo-terminal sized 100 columns x 40 rows.
- Environment: `COLORTERM=truecolor TERM=xterm-256color`
- Check: the final terminal screen equals the target cell by cell (character, foreground, background, bold, italic, underline, strikethrough, reverse). How the escape sequences are written does not matter.

Target screen text:

```text
Deploy ok in 3.2s
outer inner deep outer
 FAIL  retry in 5s
```

Cells carrying color or attributes (every other cell is default):

- line 1: `Deploy` : bold
- line 1: `ok` : fg green
- line 1: `3.2s` : fg yellow, italic
- line 2: `outer ` : bold
- line 2: `inner ` : bold, italic
- line 2: `deep` : bold, italic, underline
- line 2: ` outer` : bold
- line 3: ` FAIL ` : fg red, bg white, bold
- line 3: `retry` : strikethrough
