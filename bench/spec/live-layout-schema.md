# Live, Layout and Columns case schema (corpus 3)

Extension of [case-schema.md](case-schema.md). `cases/live_layout.jsonl` holds one JSON object per line, 30 cases for each of `columns`, `layout` and `live`. Python Rich 15.0.0 renders each case into `golden/live_layout/<id>.ansi`; a candidate adapter renders the same description and writes `<id>.ansi`. The existing correctness corpus (`cases/correctness.jsonl`, corpus 1) and its goldens are not touched: this is a separate file with its own label, so the later structured-output corpus (2) and this one never collide.

## Case

| field | meaning |
|---|---|
| `id` | `<feature>-<nnn>` |
| `feature` | `columns`, `layout`, `live` |
| `width` | terminal width in columns (40, 60, 80, 100 or 120) |
| `height` | terminal height in rows; default 24. A layout is as tall as the terminal, and a live frame taller than it overflows |
| `color_system` | `truecolor`, `256`, `standard` or `none`, as in the base schema |
| `corpus` | `3` |
| `renderable` | the node to render (below) |

The console is configured as in the base schema (explicit width and height, no emoji replacement, no highlighting, no environment). The console of a `live` case also has `terminal`: whether the stream is a terminal (and so interactive); the default of the other features is `true`.

## Nodes

All the nodes of the base schema (`text`, `table`, `panel`, `tree`, `progress`, `error`) plus these.

### `columns`

| field | meaning |
|---|---|
| `items` | list of nodes (`text` by markup, `panel`, `table`, `tree`) |
| `padding` | `[vertical, horizontal]` or `[top, right, bottom, left]`; default `[0, 1]` |
| `width` | fixed content width of every column, or `null` to measure |
| `expand` | stretch the grid to the console width |
| `equal` | every column as wide as the widest item |
| `column_first` | fill top to bottom, then left to right |
| `right_to_left` | start from the right |
| `align` | `null`, `left`, `center` or `right`: alignment of each item in its cell |
| `title` | markup or `null`, centered over the grid |

Rich builds a borderless grid table with `collapse_padding` and no edge padding, so cells are separated by their padding and the first and last column have none outside.

### `layout`

`root` is a layout node:

| field | meaning |
|---|---|
| `name` | optional identifier (not drawn) |
| `size` | fixed size in cells along the split, or absent (flexible); `0` is flexible, as in Rich |
| `ratio` | share of the flexible space, default 1 |
| `minimum_size` | default 1 |
| `visible` | default `true`; an invisible child takes no space |
| `renderable` | a node, for a leaf |
| `splitter` | `row` (children side by side) or `column` (stacked), for a node with `children` |
| `children` | layout nodes |

A node has `renderable` or `children`. A leaf with no renderable is the Rich placeholder (not claimed). The layout fills `width` by `height`; the output is `height` lines.

### `live`

| field | meaning |
|---|---|
| `frames` | 1 to 6 nodes; the first is the renderable the display starts with and each later one is an update |
| `transient` | clear the display when it ends |
| `vertical_overflow` | `ellipsis`, `crop` or `visible` |
| `terminal` | whether the stream is a terminal; a stream that is not one only shows the final frame |

The reference runs `Live(frames[0], console=..., auto_refresh=False, transient=..., vertical_overflow=..., redirect_stdout=False, redirect_stderr=False)`, `start(refresh=True)`, one `update(frame, refresh=True)` per later frame and `stop()`, and keeps every byte written to the stream. There is no timing: the bytes depend only on the frames.

## Expected behavior of the redraw (from Rich, for the record)

On a terminal: the cursor is hidden at the start; every refresh writes `CR`, `ESC [ 2 K` and, for each earlier line but the first, `ESC [ 1 A ESC [ 2 K` (nothing on the first frame), then the lines of the frame separated by newlines with no newline after the last; the end redraws the last frame in full (vertical overflow `visible`), writes a newline, shows the cursor and, when transient, writes `CR` and `ESC [ 1 A ESC [ 2 K` once per line. A frame taller than the console with `ellipsis` keeps `height - 1` lines and ends with a line of `...` centered, in bold red; with `crop` it keeps `height` lines. On a stream that is not a terminal nothing is written until the end, and then only the last frame unless transient. The goldens are the definition; this paragraph only explains them.
