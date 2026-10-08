# Output format selection (normative, pre-registered for v0.8)

Written before the implementation; gate S6 of `foundation/structured-output-criteria.md`.

## The three formats

| Format | What `Console::print` writes |
|---|---|
| `rich` | What the console wrote before v0.8: styled text with the escape sequences its capabilities allow. On a pipe without a forced terminal this has no escape sequence (the capability resolver already decides that), on a terminal it is styled. |
| `plain` | The same lines with no escape sequence at all, whatever the capabilities say (`FORCE_COLOR`, `COLORTERM`, a terminal). The text equals the `rich` output at the same width with its escape sequences removed. |
| `json` | The document of `structured-json.md`, independent of width and capabilities. |

## Selection

1. An explicit choice on the console (`ConsoleBuilder::format`) wins over everything.
2. Otherwise the environment variable `HUD_FORMAT`, read once per process: the values `rich`, `plain` and `json`, matched without regard to ASCII case. Any other value, and an empty value, is ignored as if unset, silently.
3. Otherwise `rich`. The default never changes with the stream: a pipe already gets text without escapes from the capability resolver, and JSON is only ever produced when asked.

`NO_COLOR` and `FORCE_COLOR` keep their meaning for `rich`; `plain` and `json` ignore both.

## The matrix

Cells are the product of: `HUD_FORMAT` in {unset, `rich`, `plain`, `json`, `JSON`, `bogus`} (6) x stream in {terminal, pipe} (2) x `FORCE_COLOR` in {unset, `1`} (2) = 24 cells, with `TERM=xterm-256color` and `COLORTERM=truecolor` always set, plus 6 override cells: builder format in {`rich`, `plain`, `json`} x `HUD_FORMAT` in {`json`, `plain`} where the builder wins. Expected result of printing the table of task `t09-structured`:

| `HUD_FORMAT` | terminal | pipe | pipe with `FORCE_COLOR=1` |
|---|---|---|---|
| unset, `rich`, `bogus` | styled (contains ESC) | no ESC, equal to `plain` | styled (contains ESC) |
| `plain` | plain (no ESC) | plain | plain (no ESC) |
| `json`, `JSON` | JSON document | JSON document | JSON document |

A cell passes when the output has the property of its row (styled means at least one `ESC` byte and the plain text equal to `plain`; plain means no `ESC` and the exact lines of the table; JSON means the exact bytes of the `t09-structured` target).
