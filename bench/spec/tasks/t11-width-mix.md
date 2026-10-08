# t11-width-mix: Wide characters in a table inside a panel

## Statement

Print a panel titled "Release notes" that contains a table with three columns, Locale, Greeting and Note, all left aligned, and four rows: (ja, こんにちは世界, wide characters), (ko, 안녕하세요, wide characters), (pt, Olá, mundo, one accent) and (emoji, 🚀 launch 🎉, two wide symbols). Use the crate's panel and table facilities. The characters of the first two rows and the emoji occupy two terminal cells each, so every border must still line up. The output must be exactly the target below.

## Run `main`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLUMNS=100 NO_COLOR=1 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
╭───────────────────────────────────────── Release notes ──────────────────────────────────────────╮
│ ┏━━━━━━━━┳━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━┓                                                   │
│ ┃ Locale ┃ Greeting       ┃ Note             ┃                                                   │
│ ┡━━━━━━━━╇━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━┩                                                   │
│ │ ja     │ こんにちは世界 │ wide characters  │                                                   │
│ │ ko     │ 안녕하세요     │ wide characters  │                                                   │
│ │ pt     │ Olá, mundo     │ one accent       │                                                   │
│ │ emoji  │ 🚀 launch 🎉   │ two wide symbols │                                                   │
│ └────────┴────────────────┴──────────────────┘                                                   │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
```
