# Contributing to hud

Thanks for looking. hud is a library for terminal output that reads at a glance, and it is measured: a change is accepted when the numbers in `bench/` do not get worse and the tests describe the new behavior.

## Build and test

```bash
cargo test --workspace                    # unit, property and doc tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo xtask check-layers                  # layer rules inside the hud crate
cargo xtask gen-width                     # regenerate the Unicode tables; the diff must be empty
cargo deny check                          # licenses, advisories, bans (cargo install cargo-deny)
cargo build -p hud --examples
cargo public-api -p hud > api/hud.txt          # refresh the public API snapshot (needs nightly)
cargo public-api -p hud-width > api/hud-width.txt
```

The minimum supported Rust version is 1.85. `hud-width` must keep building without `std` (`cargo build -p hud-width --target wasm32-unknown-unknown`).

The benchmark lives in `bench/` and has its own Python environment (`uv sync`, then `uv run pytest`). Run it before and after a change that touches rendering, width or capabilities; `bench/README.md` lists the scripts.

## Rules that are not negotiable

- **No dependency on another Rich port**, and no copied code from one. Python Rich and the other crates are references through the goldens, nothing more.
- **No `unsafe`.** `unsafe_code` is forbidden in the workspace.
- **Layers.** `model` is data, `services` is logic with no terminal I/O, `integrations` talks to the OS, `console` and `widgets` compose them. `cargo xtask check-layers` enforces it.
- **No child processes.** The terminal size comes from the OS once per process.
- **Goldens, thresholds and the pass criteria are read-only.** If you think a golden or a criterion is wrong, open an issue with the evidence; do not edit it in a feature change. A deliberate difference from Rich goes in `DEVIATIONS.md` with the standard or the source that justifies it.
- **Docs are an instrument.** The README and the rustdoc are what the first-try test reads. Keep examples compiling, and keep claims to what the benchmark measures.

## Proposing a change

1. Open an issue first for anything that changes the public API.
2. Write the test that fails, then the change. Rendering changes need a golden or an oracle vector; width changes need a case in the corpus.
3. Run the commands above. The CI runs them on Linux and macOS, plus the minimum Rust version and a fuzz job.
4. In the pull request, say what you measured. If the change moves a number in `bench/`, include both.

Commit messages follow `type(scope): summary`, for example `fix(width): keep a flag whole when truncating`.

## Reporting bugs

Use the bug report template. The most useful report is the smallest program that prints something wrong, the output you expected, and the output of `cargo run -p hud --example env` in the same terminal.

## License

By contributing you agree that your work is licensed under MIT OR Apache-2.0, as the rest of the repository.
