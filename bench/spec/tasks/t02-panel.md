# t02-panel: Titled panel containing wrapped text

## Statement

Print the paragraph given in the target inside a panel titled "Notice". The panel spans the full terminal width and the text wraps inside it. The text is: hud renders terminal output that reads at a glance. Tables, panels, trees and progress share one width model, so wide characters such as 日本語 and emoji keep every border aligned on any terminal.

## Run `main`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLUMNS=100 NO_COLOR=1 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
╭───────────────────────────────────────────── Notice ─────────────────────────────────────────────╮
│ hud renders terminal output that reads at a glance. Tables, panels, trees and progress share one │
│ width model, so wide characters such as 日本語 and emoji keep every border aligned on any        │
│ terminal.                                                                                        │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
```
