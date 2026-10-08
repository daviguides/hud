# t03-progress: Multi-task progress that finishes

## Statement

Show three progress tasks at once: "fetch index" (3 steps), "download crates" (120 steps) and "verify" (12 steps). Each line shows the description, a 30 cell wide bar, the percentage and the completed/total counts. Advance every task to completion in a loop and exit. The check reads the final screen of a 100 column terminal after the program exits; it must equal the target.

## Run `main`

- Stream: stdout is a pseudo-terminal sized 100 columns x 40 rows.
- Environment: `NO_COLOR=1 TERM=xterm-256color`
- Check: the final terminal screen equals the target cell by cell (character, foreground, background, bold, italic, underline, strikethrough, reverse). How the escape sequences are written does not matter.

Target screen text:

```text
fetch index     ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% 3/3
download crates ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% 120/120
verify          ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% 12/12
```

Cells carrying color or attributes (every other cell is default):

- no cell carries any color or attribute
