# Releasing

> This is the copy on the branch `release/1.0.0`: the crates here are version 1.0.0 and publishable. On `main` they are `0.1.0` with `publish = false` until the owner says go, and this branch is never merged before that.

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

## The go

The one sequence the coordinator runs when the owner says go. Nothing here runs before that, and the owner's go is the only thing left to wait for: the evaluation, the dry runs and the checklist above are done on the commit that `release/1.0.0` points to.

Prerequisites: a crates.io token with the `publish-update` scope for the two crates (they exist at 0.0.0, so `publish-new` is not enough) in `~/.cargo/credentials.toml`, a verified email on the crates.io account, and the GitHub account that owns the repository active (`gh auth status`).

```bash
cd ~/work/sources/hud

# 0. the commit to release is the head of the release branch; CI is green on it
git fetch origin
git rev-parse origin/release/1.0.0
gh run list --branch release/1.0.0 --limit 1          # conclusion: success

# 1. bring the release branch to main, tag it
git checkout main && git pull --ff-only
git merge --no-ff origin/release/1.0.0 -m "release: hud and hud-width 1.0.0"
git push origin main
git tag -a v1.0.0 -m "hud and hud-width 1.0.0"
git push origin v1.0.0

# 2. publish hud-width first, wait for the index, then hud
cargo publish -p hud-width
until cargo info hud-width 2>/dev/null | grep -q "^version: 1.0.0"; do sleep 5; done
cargo publish -p hud

# 3. verify what a user gets, in an empty directory, from the registry
probe=$(mktemp -d) && cd "$probe"
cargo init --name hud_probe --quiet
cargo add hud
cat > src/main.rs <<'RS'
fn main() {
    hud::println!("[bold green]hud[/] installed: width of 你好🇧🇷 is {}", hud::cell_width("你好🇧🇷"));
}
RS
cargo run --quiet                                     # prints the line, width 6

# 4. the GitHub release, with the changelog of 1.0.0 as its notes
cd ~/work/sources/hud
sed -n '/^## 1.0.0/,$p' CHANGELOG.md > /tmp/hud-1.0.0-notes.md
gh release create v1.0.0 --title "hud 1.0.0" --notes-file /tmp/hud-1.0.0-notes.md
```

If a step fails: `cargo publish` errors before it uploads (metadata, dirty tree, missing token) change nothing on the registry. A publish that succeeded cannot be undone, only yanked (`cargo yank --version 1.0.0 hud`), so step 2 is the last step that is hard to reverse and the dry runs in `bench/` record what it will do.
