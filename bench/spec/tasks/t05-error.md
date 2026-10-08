# t05-error: Formatted error with a cause chain

## Statement

Print an error report: a panel titled "Error" spanning the full width, containing the message "could not read config", a blank line, "Caused by:" followed by the numbered causes ("No such file or directory (os error 2)" and "path: /etc/hud/config.toml") indented four spaces, a blank line, and the hint "run again with --verbose for details". Build it from the crate's facilities; the output must equal the target.

## Run `main`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLUMNS=100 NO_COLOR=1 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
╭───────────────────────────────────────────── Error ──────────────────────────────────────────────╮
│ could not read config                                                                            │
│                                                                                                  │
│ Caused by:                                                                                       │
│     0: No such file or directory (os error 2)                                                    │
│     1: path: /etc/hud/config.toml                                                                │
│                                                                                                  │
│ hint: run again with --verbose for details                                                       │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
```
