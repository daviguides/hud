# t10-composition: Nested renderables in a layout

## Statement

Print one screen made with a layout. The area is split into two rows. The bottom row is exactly 4 lines tall and holds a progress display with two finished tasks: "build" (10 of 10) and "test" (40 of 40); each line shows the description, a 20 cell wide bar and the percentage, nothing else. The top row takes all the remaining height and is split into two columns of equal size: the left one holds a table with columns Crate and Version and the rows (hud, 0.1.0) and (hud-width, 0.1.0); the right one holds a panel titled "Status" that contains the text "All checks passed". The check compares the bytes written to a pipe; the console is 100 columns by 40 lines (the variables COLUMNS and LINES say so) and the layout fills those 40 lines. The output must be exactly the target below.

## Run `main`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLUMNS=100 LINES=40 NO_COLOR=1 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
┏━━━━━━━━━━━┳━━━━━━━━━┓                           ╭──────────────────── Status ────────────────────╮
┃ Crate     ┃ Version ┃                           │ All checks passed                              │
┡━━━━━━━━━━━╇━━━━━━━━━┩                           │                                                │
│ hud       │ 0.1.0   │                           │                                                │
│ hud-width │ 0.1.0   │                           │                                                │
└───────────┴─────────┘                           │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  │                                                │
                                                  ╰────────────────────────────────────────────────╯
build ━━━━━━━━━━━━━━━━━━━━ 100%                                                                     
test  ━━━━━━━━━━━━━━━━━━━━ 100%                                                                     
                                                                                                    
                                                                                                    
```
