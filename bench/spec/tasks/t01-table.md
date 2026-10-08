# t01-table: Styled table with header, alignment and a title

## Statement

Print a table titled "Build report" with four columns: Crate (left aligned), Version (centered), Downloads (right aligned) and Status (left aligned), and five rows of data. Every column header is shown. Use the crate's table facility; the output must be exactly the target below.

## Run `main`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLUMNS=100 NO_COLOR=1 TERM=xterm-256color`
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
