# Structured output: JSON shape `hud/1` (normative, pre-registered for v0.8)

Written before the implementation. `foundation/structured-output-criteria.md` is the list of gates; this file is the expected output of the JSON mode, and `structured-json.schema.json` is its machine form. Neither changes to fit a result; a defect in either is an instrument change, logged in `CHANGELOG.md` and applied to every arm.

## Principle

JSON describes WHAT a value says, never HOW it looks: no styles, no colors, no widths, no wrapping, no box characters, no timings. The same value gives the same JSON on a 20 column and a 200 column console, in every run. Text fields are plain strings: markup is read and its tags removed (`[bold]x[/]` is `"x"`); markup that does not parse is kept as written, the way printing treats it.

## Canonical text

The bytes are exactly `json.dumps(document, indent=2, ensure_ascii=False) + "\n"` of Python 3: two space indent, `,` and `: ` separators, keys in the order listed below, non-ASCII written as is, only `"` `\` and the control characters below U+0020 escaped (`\n`, `\r`, `\t`, `\b`, `\f` as short escapes, the rest as `\u00xx` in lower case), `[]` and `{}` for empty containers, integers without a fraction or exponent, no floating point anywhere, one final newline.

## Document

```json
{ "schema": "hud/1", "content": <node> }
```

## Nodes

Every node is an object whose first key is `type`. Keys appear in the order shown.

| `type` | Keys after `type` | Notes |
|---|---|---|
| `text` | `text` (string) | A `Text`, a string printed as markup, and the plain rendering of a custom renderable at width 80. Line breaks stay as `\n`. |
| `table` | `title` (string or null), `caption` (string or null), `columns` (array of `{ "header": string, "justify": "left"\|"center"\|"right"\|"full" }`), `rows` (array of arrays of strings) | Rows keep their order; a row shorter than the columns is not padded. A column with no justify set is `left`, as in a rendered column. |
| `panel` | `title` (string or null), `subtitle` (string or null), `body` (node) | |
| `tree` | `label` (string), `children` (array of `tree` nodes) | Children are `tree` nodes in order. |
| `progress` | `tasks` (array of `{ "description": string, "completed": integer, "total": integer, "finished": boolean, "visible": boolean }`) | State at the moment of rendering; no elapsed time, speed or estimate. |
| `error` | `message` (string), `causes` (array of strings), `hint` (string or null) | The cause chain in order. |
| `columns` | `title` (string or null), `items` (array of nodes) | |
| `layout` | `name` (string or null), `ratio` (integer), `size` (integer or null), `visible` (boolean), `direction` (`"none"\|"row"\|"column"`), `content` (node or null), `children` (array of `layout` nodes) | `direction` is how the children are split; a leaf has `none` and no children. |
| `group` | `items` (array of nodes) | |
| `padding` | `top`, `right`, `bottom`, `left` (integers), `content` (node) | |

## Example

The value of task `t09-structured` (two columns, two rows):

```json
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

## Round trip

A document parses with any JSON parser; rebuilding the node tree from the parsed value and writing it again gives the same bytes (criterion S3). Rebuilding a widget from JSON is not offered.
