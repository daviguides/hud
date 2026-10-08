## What changes

## What was measured

Numbers from `bench/` before and after, or why none move.

## Checklist

- [ ] Test that fails without the change (golden, oracle vector or corpus case for rendering and width)
- [ ] `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`
- [ ] `cargo xtask check-layers`
- [ ] `cargo deny check`
- [ ] Public API change reviewed in the API snapshot (`api/`)
- [ ] `DEVIATIONS.md` updated if output now differs from Python Rich
- [ ] No golden, threshold or pass criterion edited
