# hud: Live, Layout and Columns, results

Branch `parallel/live-layout`, measured against `foundation/live-layout-criteria.md` (pre-registered before the implementation). Rich 15.0.0 is the reference. Nothing is published.

## Verdict by criterion

| # | Criterion | Result | Verdict |
|---|---|---|---|
| 1 | Corpus 3 byte-identical | `columns` 30/30, `layout` 30/30, `live` 30/30; `progress_live` 12/12 (added by CHANGELOG L2, before the comparison) | GO |
| 2 | No regression | correctness 210/210, `table_unicode` 12/12, width 99.2% (496/500) with 0 splits in 40 000 fold and truncate cases, 0 misaligned table rows of 97, 0 of 704 rows too wide, capability 40/40, t01 to t08 all pass, S3 verify 1 000 of 1 000 frames, `check-layers` ok, clippy `all = deny` clean; after merging v0.7 the same gates pass and the workspace tests are 98 + 13 + 8 unit and oracle tests and 52 doctests | GO |
| 3 | Progress on the shared protocol | the 30 progress goldens, t03 and the S3 frame check pass; `progress_live` 12/12; 1 100 of 1 100 ASCII random vectors byte-identical to Rich's Progress | GO |
| 4 | Redraw cost | S3 median 1.408 ms against 1.485 ms for the merge base measured alternately in one session (4 rounds of 30): ratio **0.947**, 95% CI [0.936, 0.966] | GO, at load 3.0 to 3.2 (below the pilot's 4.0, not strictly idle) |
| 5 | Fuzz | `columns`, `layout`, `live`: 80 000 inputs each, 16 seeds, 0 panics, 0 violations; the nine earlier features: 80 000 each, 0 and 0; after the merge, all thirteen features 15 000 each, 0 and 0. Mutation check below | GO |
| 6 | Differential oracle | 4 400 ASCII vectors (1 100 each for `columns`, `layout`, `live`, `progress_live`): all byte-identical. 1 200 Unicode vectors: 40 differ (columns 31, layout 5, live 4, progress 0), every one a flag or combining mark (D-003) or a wide character where Rich ellipsizes a word that fits (D-024); 0 unexplained | GO |
| 7 | API | no positional `None` (0 functions with more than 3 positional parameters, 0 with `Option` parameters); `&mut self` only on `Layout::{split_row, split_column, unsplit, update, get_mut}`, as in Rich, no method returns a `Result`; name parity by existence unchanged at 28/40 = 70% | GO |
| 8 | Docs | 138/138 documented, 34/34 types with a doctest, 0 ignored doctests, README shows `Columns`, `Layout` and `Live` | GO |
| 9 | Layering and hygiene | no new dependency, no child process, `unsafe_code` forbidden, MSRV 1.85, the redraw protocol exists once (`services/live.rs`, used by `Live` and `Progress`) | GO |

## What the measurement found

- **A real cost in the first S3 run, and its cause.** The first A/B gave 1.456 (CI 1.443 to 1.468): worse than the gate. Following `no-go-investigation.md`: the instrument was sound (output verified equal to the golden, same session, same alternation), so the change was read. The per-frame bytes grew 3.5% (a per-line erase instead of one `ESC [ n A`), which cannot explain 45%; a sample of the running process showed two `write` calls per frame. A frame that does not end with a newline is cut by Rust's line-buffered standard output at its last newline, and the tail waits for the flush: two system calls. `integrations::write_frame` now flushes what the stream held and writes the frame with one call (unix, through the `rustix` already in the tree). S3 went from 1.456 to 0.947.
- **A defect found by an existing fuzz property.** The new `push_frame` did not cut a line wider than the stream (`Progress` lines come from `render_lines`, not from `split_lines`): 2 violations in 15 000 progress inputs; fixed by cropping each frame to the stream width.
- **Panel height.** Rich passes the height of a layout region to the panel it holds, and the panel fills it. That needed a new defaulted `Renderable::render_region(width, height, caps)`; `Panel` overrides it and so does `Layout`.
- **Cells lay text out.** Rich renders a table cell with the justify and overflow of its column, so text in a `Columns` grid is left-justified with an ellipsis at the edge (and `align` has no effect on wrapped text). `Body` now remembers when it holds text so the grid can do the same.
- **One non-obvious Rich rule, reproduced on purpose.** The grid measures a fixed-width column with a padding rule that differs from the one it renders with when left and right padding differ (`Table._get_padding_width`); hud follows the measuring rule as Rich does.
- **Rich's `Progress.stop`** writes a line break on a stream that is not interactive even when the display is transient and shows nothing; hud now does too (found by `progress_live-011`).

## Mutation sensitivity of the fuzz properties

The library was mutated, the fuzz run on 3 000 inputs, and the mutation reverted (nothing committed).

| Mutation | Caught in |
|---|---|
| live rewind goes up one line too far | 1 995 of 3 000 `live` inputs |
| columns padding lines one cell short | 98 of 3 000 `columns` inputs |
| layout remainder not carried to the next region | 791 of 3 000 `layout` inputs |
| layout region one line short | 1 190 of 3 000 `layout` inputs |
| layout row offset plus one | not caught: an equivalent mutant, offsets only order the regions |

## Not claimed

Listed in `DEVIATIONS.md` D-048: the `Layout` placeholder, `Layout.tree` and `refresh_screen`, custom splitters, a layout whose children are all invisible, `Live` alternate screen, output redirection, nested displays and `get_renderable`, the `Status` spinner (named in the v0.9 scope of `features.md`, not part of this track), a `Columns` with `width` so small that no column fits, and items with a base style of their own. D-037 (the bytes of a live display) is resolved.

## Merge notes for the coordinator

- `origin/main` as of v0.7 (`2d56cdc`) is already merged into the branch (`ca87b6c`); `main` has moved since, so a second small merge may be needed. Conflicts solved on the way: exports and modules (`lib.rs`, `model/mod.rs`), `console.rs`, `services/panel.rs` (`Pad` is the spacing value since v0.7, `Padding` the wrapper), `progress.rs` (`shown` replaces the task count in the fast-plan test), and the numbering of `DEVIATIONS.md` (this track's entry is D-048) and of `bench/CHANGELOG.md` (entries `L1` to `L4`, to be renumbered).
- The public API snapshot `api/hud.txt` is regenerated (146 lines added, `Body(_)` becomes `Body(_, _)`).
- New public items: `Columns`, `Layout`, `Live`, `LiveBuilder`, `VerticalOverflow`, `Renderable::render_region`, `From<Columns>` and `From<Layout>` for `Body`.
- Behavior changes to `Progress`: its bytes are Rich's (per-line erase, no newline after the last line, a blank first frame, the line break at the end of a non-interactive display); screens are unchanged (t03, S3, the progress goldens pass).
- `fuzz_live_layout` is added to the CI fuzz job.
- Not run here: the evaluation of v0.9 as a candidate with the DX agent runner, and Windows (the frame write falls back to the standard stream there).
