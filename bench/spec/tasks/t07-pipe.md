# t07-pipe: Output to a pipe (no TTY)

## Statement

Print the same table as the target, but styled: header in bold cyan, status "ok" green, "warn" yellow, "fail" red. The program runs with stdout redirected to a file (not a terminal) and TERM=xterm-256color, COLORTERM=truecolor in the environment. The captured bytes must contain no escape byte at all and equal the target.

## Run `main`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLORTERM=truecolor COLUMNS=100 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
               Build report                
┏━━━━━━━━━┳━━━━━━━━━┳━━━━━━━━━━━━┳━━━━━━━━┓
┃ Crate   ┃ Version ┃  Downloads ┃ Status ┃
┡━━━━━━━━━╇━━━━━━━━━╇━━━━━━━━━━━━╇━━━━━━━━┩
│ clap    │ 4.5.40  │ 12,400,000 │ ok     │
│ serde   │ 1.0.219 │ 98,300,000 │ ok     │
│ tokio   │ 1.46.1  │ 45,100,000 │ ok     │
│ ratatui │ 0.29.0  │  2,310,000 │ warn   │
│ syn     │ 2.0.104 │ 87,000,000 │ fail   │
└─────────┴─────────┴────────────┴────────┘
```
