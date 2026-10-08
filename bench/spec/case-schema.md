# Correctness case schema

Language-neutral description of one rendering case. `cases/correctness.jsonl` (210 cases, 30 per feature) and `cases/table_unicode.jsonl` (12 cases) hold one JSON object per line. Python Rich 15.0.0 renders each case into `golden/correctness/<id>.ansi` (`golden/table_unicode/`); a candidate adapter renders the same description and writes `<id>.ansi`. Where this schema is silent about styling, the default theme of Rich 15.0.0 applies: the golden is the definition, and byte equality is what correctness measures.

## Case

| field | meaning |
|---|---|
| `id` | `<feature>-<nnn>`, or `tw-<nnn>` for the Unicode table set |
| `feature` | `style`, `markup`, `table`, `panel`, `tree`, `progress`, `error` |
| `width` | terminal width in columns (40, 60, 80, 100 or 120) |
| `color_system` | `truecolor`, `256`, `standard` (16) or `none` |
| `corpus` | corpus version, `1` |
| `renderable` | the node to render (below) |

The console is configured explicitly, never from the process environment: width `width`, terminal forced on, color depth `color_system` (`none` = terminal on, no color, no styles), no emoji-code replacement (`:smile:` stays literal), no automatic highlighting of numbers or other tokens, no legacy Windows mode. The output is the node printed once, followed by a newline.

## Renderables

| `t` | fields |
|---|---|
| `text` | `markup` (inline markup string) or `plain` + `style` (a style string such as `bold red on #223344`) |
| `table` | `title`, `caption` (string or null), `box`, `show_lines`, `columns[]` (`header`, `justify`, `style`, `no_wrap`), `rows[][]` (markup strings) |
| `panel` | `title`, `subtitle`, `box`, `expand`, `padding` `[vertical, horizontal]`, `border_style`, `body` (a renderable) |
| `tree` | `guide_style`, `root` = `{label, children[]}` (labels are markup strings, nesting up to depth 3) |
| `progress` | `bar_width`, `tasks[]` (`description`, `total`, `completed`): one frame, no timing columns, columns in this order: description, bar, percentage, completed/total |
| `error` | `message`, `causes[]`, `hint`: layout below |

`box` values: `rounded`, `ascii`, `simple`, `heavy`, `double`, `minimal`, `square`, `heavy_head`; each maps to the box style of the same name in Rich.

## Inline markup

Tags `[style]...[/style]`, `[/]` closing the innermost, nested and adjacent tags, `\[` as a literal bracket, and an unclosed tag running to the end of the string. Style words: `bold`/`b`, `italic`/`i`, `underline`/`u`, `strike`/`s`, `dim`, `reverse`; colors by name (`red`, `bright_blue`, `grey50`, `default`), index (`color(208)`), hex (`#ff8800`) or `rgb(12,200,99)`; `on <color>` for the background.

## Error layout

Not a Rich primitive, so the layout is fixed here and the golden is a composition of Rich primitives. A rounded panel titled `Error` with a red border containing, in order: the message in bold; if `causes` is not empty, a blank line, `Caused by:` in dim and one line `    <n>: <cause>` per cause, numbered from 0; if `hint` is set, a blank line and `hint: <hint>` in cyan.

## Known classification inputs

Rich color downgrade for `256` and `standard` is the reference. A candidate that picks a different nearest color is a `candidate_bug` unless it lists the difference as a `documented_deviation` with a reason.
