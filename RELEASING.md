# Releasing

hud has not been released. The names `hud` and `hud-width` are reserved on crates.io at version 0.0.0, and every crate in the workspace carries `publish = false`. The milestones v0.1 and later are internal git tags.

**Publishing requires the owner's explicit go.** Nothing in this document, in the CI or in a script publishes on its own, and no contributor or tool publishes a crate without that go.

## What a stable release is

A stable release is 1.0.0 of `hud-width` and of `hud`, published together, after the final evaluation says GO.

## Checklist

Everything is checked before the owner is asked.

1. **Evaluation.** The full evaluation in `bench/` has run on the commit to release, with verdicts computed by the rules written before any measurement: correctness and assertiveness gates, speed on an idle machine, the first-try test with the pinned models, and adoption cost. The result is committed as `bench/EVALUATION-RESULTS.md`. A non-GO axis has its causal decomposition (`foundation/no-go-investigation.md`) and is resolved or recorded.
2. **CI green** on the commit: check, test on Linux and macOS, MSRV, fuzz, `cargo deny`, API snapshot, examples.
3. **API.** The public API snapshot is reviewed and committed. Every public item is documented and every example in the docs compiles and runs (`cargo test --doc`, `cargo build --examples`).
4. **Metadata.** `Cargo.toml` of both crates: `description`, `license`, `repository`, `readme`, `keywords`, `categories`, `rust-version`, and exact `hud-width` version in `hud`. `publish = false` is removed in the release commit only.
5. **Notices.** `THIRD_PARTY_NOTICES.md` lists the Unicode Character Database version and license text; the generated tables carry the notice.
6. **Docs.** README claims match the committed evaluation results; the changelog for 1.0.0 is written; `DEVIATIONS.md` is current.
7. **Dry run.** `cargo publish --dry-run -p hud-width` and then `-p hud`, from a clean checkout of the release commit.

## The release

Only after the owner says go:

1. Tag the release commit `v1.0.0`.
2. Publish `hud-width`, wait for the index to show it, then publish `hud`.
3. Create the GitHub release from the tag with the changelog.
4. Confirm `cargo add hud` and `cargo add hud-width` resolve and build in an empty project.

## After

Add the published version to the README install section and remove the "not yet published" notice in the same commit as the first post-release change. Later versions follow semver; the API snapshot job fails a change that removes or alters a public item without a major bump.
