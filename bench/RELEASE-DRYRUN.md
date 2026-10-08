# hud 1.0.0: release dry runs

What was run on the branch `release/1.0.0` of `daviguides/hud` before the owner's go, and what it found. Nothing was uploaded: every publish below is `--dry-run`, which builds the package and stops before the upload. The crates.io names `hud` and `hud-width` still hold only their 0.0.0 placeholders.

The release branch is built from `main` at the freeze (`7c90504`) and differs from it by the 1.0.0 versions, the publishable manifests, the install section of the README, the changelog, the "The go" section of `RELEASING.md`, and the packaging fixes below. `main` keeps `0.1.0` and `publish = false` until the go.

## Results

| Check | Command (from a clean checkout of the branch head) | Result |
|---|---|---|
| `hud-width` dry run | `cargo publish --dry-run -p hud-width` | **passes**: 13 files, 176.3 KiB (16.1 KiB compressed), verified by a build of the packaged crate |
| `hud` dry run against the registry | `cargo publish --dry-run -p hud` | **fails, as expected, with this exact error**: `failed to select a version for the requirement hud-width = "^1.0.0"; candidate versions found which didn't match: 0.0.0; location searched: crates.io index; required by package hud v1.0.0`. It resolves once `hud-width` 1.0.0 is on the registry, which is why "The go" publishes `hud-width` first and waits for the index |
| `hud` dry run with the one missing piece patched | `cargo publish --dry-run -p hud --config 'patch.crates-io.hud-width.path="crates/hud-width"'` | **passes**: 73 files, 477.6 KiB (121.4 KiB compressed), verified by a build of the packaged crate |
| Package contents | `cargo package --list -p hud` and `-p hud-width` | `hud`: `src/`, `examples/`, `schema/hud-1.json`, `README.md`, the two licenses, manifests. `hud-width`: `src/` (with the generated tables), `README.md`, the two licenses, manifests. No test, fixture or `bench/` file |
| Clean-directory install from the artifacts | `cargo package -p hud-width -p hud` (with the patch), both `.crate` files extracted into an empty directory, a new `cargo init` project with `hud` as a path dependency on the extracted package and `hud-width` patched to the other | **works**: the probe program prints `hud installed: width of 你好🇧🇷 is 6` and a table; the packaged crates build and run outside the workspace |
| CI on the branch | `ci` workflow (`check`, `test` on Linux and macOS, `msrv`, `windows`, `deny`, `api`, examples, `check-copies`) | see the run list of the branch in GitHub; the head commit's result is recorded in the final report |

## Defects the dry run found (both fixed on the branch)

1. **The crate README was outside the crate.** `crates/hud/src/lib.rs` did `include_str!("../../../README.md")`, a path that leaves the crate, so the packaged `hud` did not compile (`couldn't read ../../../../README.md`). A crate can only include a file inside itself, and `cargo package` puts the README at the crate root. Fix: the README of `hud` lives in `crates/hud/README.md`, a copy of the one at the repository root, and `lib.rs` includes `../README.md`. `cargo xtask check-copies` (a CI step) fails if the copy differs from the root file, and `cargo xtask sync-copies` writes it.
2. **The package was 31.8 MiB.** The oracle fixtures (31 MB of Rich vectors) and the integration tests, which read files under `bench/`, were being published, and no license file shipped with the crates. Fix: an `include` list in both manifests (sources, examples, the JSON schema, the README and the two licenses) and the licenses copied into the crates by the same `check-copies` task. The packages are now 176 KiB and 478 KiB.

Neither defect is visible to anyone running `cargo build` in the workspace, which is why they were found by the dry run and not by the CI.

## What the owner's "go" runs

The command sequence is the section "The go" of `RELEASING.md` on the branch: merge the branch into `main`, tag `v1.0.0`, `cargo publish -p hud-width`, wait for the index, `cargo publish -p hud`, install into an empty project from the registry, create the GitHub release. The crates.io token needs the `publish-update` scope (the names exist at 0.0.0) and the account a verified email; both were checked when the 0.0.0 placeholders were published.

Not done on purpose, per the owner's rule: no `cargo publish` without `--dry-run`, no GitHub release, no tag, no change to the placeholders, no flip of `publish = false` on `main`.
