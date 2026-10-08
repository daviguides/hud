# hud 0.1.0 (own engine), v0.1 results

Candidate adapters: `src/bin/width_runner.rs` (width, fold, truncate) and `src/bin/cap.rs`
(capability), built from the workspace crates by path. Everything below was produced by the shared
scripts of `bench/scripts/` under the corrected harness (`bench/CHANGELOG.md` entries 1 to 9). Raw
outputs: `bench/pilot/hud/`. hud claims only width, fold, truncate and capability in v0.1; every
other file is missing, which means unsupported, never a pass.

## Gates claimed in v0.1

| Criterion (features.md, v0.1) | Result | Threshold | Verdict |
|---|---|---|---|
| Width agreement with Rich `cell_len` (`width_check.py`) | 496 / 500 = **99.2%**; contested Mc strings 13 / 13 | at least 99%, target 100% | pass; target not met, the 4 disagreements are `D-001` |
| Grapheme splits in fold, 20 000 cases | **0**; `concat_bad` 0, `too_wide` 0, `not_greedy` 0 | 0 | pass |
| Grapheme splits in truncate, 20 000 cases | **0**; `not_prefix` 0, `not_maximal` 0 | 0 | pass |
| Capability matrix (`capability.py check`, through a pty and a pipe) | **40 / 40** | 40 / 40 | pass |
| Size resolved with at most one syscall, no child process | one `ioctl` per stream per process, cached; counting probe test (`console.rs`: 1 000 reads, 1 size query); `xtask check-layers` bans `Command::new` and `std::process` | 1 syscall, 0 children | pass |
| Code point sweep reproduced | see below | claim confirmed or corrected | done |
| UCD license verified and recorded | `THIRD_PARTY_NOTICES.md`; version and license in the `tables.rs` header | recorded | pass |
| Docs (`docs_coverage.py`) | `hud-width` 16 / 16 documented, `hud` 29 / 29, both 100% with examples; 8 and 3 doctests pass, **0 ignored** | 100%, 0 ignored | pass |
| `hud-width` adds zero dependencies (`adoption.py`) | `transitive_deps` 1 (itself only) | none | pass |
| Release to crates.io | **not done**: owner decision, no publishing before stable | n/a | deviation, see `DEVIATIONS.md` |

Not claimed in v0.1 (unsupported): correctness cases (0 / 210), tables, tasks t01 to t08, speed
S1 to S4. Table alignment (`tables/`) is a v0.3 criterion.

## Adoption cost (`adoption.py`, 3 runs, versus an empty program)

| Project | Added compile time | Added stripped binary | Transitive crates |
|---|---|---|---|
| `hello_width` (`hud-width` only) | +0.15 s | +66 976 bytes | 1 (`hud-width`) |
| `hello` (`hud`: capabilities and width) | +1.19 s | +67 520 bytes | 6 (`hud`, `hud-width`, `rustix`, `bitflags`, `errno`, `libc`) |

Targets of architecture.md (3 s, 400 KB, 10 crates) are met. Compile times were taken with the
machine load average at 14 to 22 (other sessions); a loaded machine only inflates the figure,
so the pass stands, but the number must not be used to rank. Binary size and crate count do not
depend on load.

## Width sweep (`tools/width-sweep`, Rich 15.0.0 default table, `unicode-width` 0.2.2, UCD 17.0.0)

| Universe | Code points | hud-width vs Rich | `unicode-width` vs Rich |
|---|---|---|---|
| All Unicode scalar values | 1 112 064 | 13 (8 unassigned `Cn`, 5 `Lo`) | 470 (`Mc` 410, `Cf` 25, `Lo` 15, `Cn` 8, `Sk` 5, `Po` 2, `Lm` 2, `Zp` 1) |
| Assigned graphic characters | 159 629 | 5 (Kirat Rai, `D-002`) | 435 (`Mc` 410, `Lo` 15, `Sk` 5, `Po` 2, `Lm` 2, `Mn` 1) |

The claim of `rs-rich`'s `docs/DIVERGENCES.md` (348 of 127 754) is confirmed in kind and corrected
in number: `unicode-width` does disagree with Rich, dominated by spacing marks (`Mc`), but we
count 435 to 470 depending on the universe, and the 127 754 universe is not defined in their
documents (no natural subset of the UCD we tried gives that count: all scalars 1 112 064,
assigned 297 334, assigned without private use 159 866, graphic 159 629). hud's own 5 to 13
disagreements are listed in `DEVIATIONS.md` (`D-002`, `D-002b`).

## Reproduce

```bash
cd bench/candidates/hud && cargo build --release
./target/release/width_runner ../../results/hud/width
../../.venv/bin/python ../../scripts/width_check.py ../../results/hud/width
../../.venv/bin/python ../../scripts/capability.py check -- ./target/release/cap
../../.venv/bin/python ../../scripts/width_sweep_rich.py ../../results/hud/rich_cp_widths.tsv
(cd ../../tools/width-sweep && cargo run --release -- ../../results/hud/rich_cp_widths.tsv)
```
