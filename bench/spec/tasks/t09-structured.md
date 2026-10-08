# t09-structured: One print call, three formats

## Statement

Build a table with two columns, Name (left aligned) and Count (right aligned), and two rows: api, 12 and cli, 7. Print it with exactly one print call and nothing else. The format of the output (rich text, plain text or JSON) is chosen by the environment through the variable HUD_FORMAT, so the same program must produce all four outputs below; your program must not read HUD_FORMAT itself. The runs: `terminal` (a terminal, HUD_FORMAT unset): the final screen equals the target; `pipe_default` (a pipe, HUD_FORMAT unset): the bytes equal the target; `pipe_plain` (a pipe with FORCE_COLOR=1, HUD_FORMAT=plain): the bytes equal the target, with no escape sequence; `pipe_json` (a pipe, HUD_FORMAT=json): the bytes equal the JSON document below, exactly.

## Run `terminal`

- Stream: stdout is a pseudo-terminal sized 100 columns x 40 rows.
- Environment: `COLORTERM=truecolor TERM=xterm-256color`
- Check: the final terminal screen equals the target cell by cell (character, foreground, background, bold, italic, underline, strikethrough, reverse). How the escape sequences are written does not matter.

Target screen text:

```text
┏━━━━━━┳━━━━━━━┓
┃ Name ┃ Count ┃
┡━━━━━━╇━━━━━━━┩
│ api  │    12 │
│ cli  │     7 │
└──────┴───────┘
```

Cells carrying color or attributes (every other cell is default):

- line 2: ` Name ` : bold
- line 2: ` Count ` : bold

## Run `pipe_default`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLORTERM=truecolor COLUMNS=100 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
┏━━━━━━┳━━━━━━━┓
┃ Name ┃ Count ┃
┡━━━━━━╇━━━━━━━┩
│ api  │    12 │
│ cli  │     7 │
└──────┴───────┘
```

## Run `pipe_plain`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLORTERM=truecolor COLUMNS=100 FORCE_COLOR=1 HUD_FORMAT=plain TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
┏━━━━━━┳━━━━━━━┓
┃ Name ┃ Count ┃
┡━━━━━━╇━━━━━━━┩
│ api  │    12 │
│ cli  │     7 │
└──────┴───────┘
```

## Run `pipe_json`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLORTERM=truecolor COLUMNS=100 HUD_FORMAT=json TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
{
  "schema": "hud/1",
  "content": {
    "type": "table",
    "title": null,
    "caption": null,
    "columns": [
      {
        "header": "Name",
        "justify": "left"
      },
      {
        "header": "Count",
        "justify": "right"
      }
    ],
    "rows": [
      [
        "api",
        "12"
      ],
      [
        "cli",
        "7"
      ]
    ]
  }
}
```
