# Corpus 5: Spinner and Status (case schema)

Extends `case-schema.md` and `live-layout-schema.md`. File `cases/status.jsonl`, goldens `golden/status/<id>.ansi`, label `5`. Every frame is drawn at a stated time: the clock is part of the case.

Common keys: `id`, `feature` (`spinner` or `status`), `width`, `height` (24), `color_system`, `corpus` (`"5"`), `renderable`.

## `spinner` (146 cases: each of the 73 animations twice)

`renderable`: `{"t": "spinner", "name", "text" (markup or null), "style" (a Rich style string or null), "speed", "ops": [...]}`.

Ops, in order, on one spinner and one console whose clock reads `at`:

- `{"op": "print", "at": t}`: set the clock to `t`, then print the spinner (one frame, a newline).
- `{"op": "update", "text", "style", "speed"}`: `Spinner.update`; the new speed takes effect at the next print.

The first print fixes the start of the animation (Rich takes the time of the first render).

## `status` (36 cases)

`renderable`: `{"t": "status", "text" (markup, may be empty), "spinner" (animation name), "spinner_style" (string or null for the default), "speed", "terminal" (bool: an interactive stream or not), "events": [...]}`.

Events, in order; every event carries `at`, the clock reading:

- `start`: `Status.start()` (the cursor is hidden; nothing is drawn yet).
- `refresh`: draw the display now (the refresh thread is switched off in the reference; hud's counterpart refreshes explicitly).
- `update`: `Status.update(...)` with the keys present among `text`, `spinner`, `spinner_style`, `speed`. A new `spinner` replaces the spinner and refreshes at once; the others change the existing one.
- `stop`: `Status.stop()`; the last frame is drawn and a transient status is erased.

A status is transient. On a stream that is not a terminal nothing is ever written, and that empty output is the golden.

The golden is the exact bytes written to the stream from the first event to `stop`.
