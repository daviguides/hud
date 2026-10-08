# Stability

hud follows semantic versioning from 1.0. This file says what that means for hud and hud-width. The rules are the ones of the project record `foundation/api-stability.md`, copied here at the freeze.

## Guarantees (1.x)

**What is public API.** The items listed in `api/hud.txt` and `api/hud-width.txt`, which CI compares with the code on every change. `#[doc(hidden)]` items (the macro support) are not API.

**What a major version is needed for.** Removing or renaming an item; changing a signature, a bound or a derive; adding a required method to a trait; adding a variant to an enum or a field to a struct that is not `#[non_exhaustive]`; changing what an environment variable does; changing the bytes of the plain and JSON formats; adding or changing a node type or key of the JSON schema `hud/1`.

**What a minor version may do.** Add items; add methods to `Renderable` with a default body; add variants to a `#[non_exhaustive]` enum and fields to a `#[non_exhaustive]` struct; add a spinner animation; raise the MSRV (see below); fix output toward what Python Rich writes (listed in `CHANGELOG.md`).

**What a patch version may do.** Fix bugs. Output changes only when the old output was a deviation from Rich or a panic, and the change is listed in `CHANGELOG.md`.

**`#[non_exhaustive]`** (they will grow; match with a wildcard arm and build with the constructors and setters): `ColorSystem`, `Format`, `BoxStyle`, `Attribute`, `ProgressColumn`, `Capabilities`, `EnvSnapshot` (`EnvSnapshot::new` and one setter per variable) and `StreamInfo` (`StreamInfo::new`, `StreamInfo::terminal`, setters).

**Exhaustive on purpose** (the set is fixed by Rich or by the terminal model): `Align`, `Justify`, `Overflow`, `VerticalOverflow`, `Stream`, `Color`, and the small data types you construct for custom renderables: `Segment`, `Span`, `Measure` and `Pad`.

**Traits.** `Renderable` is open: implement it for your own types; only `render` is required. No trait is sealed. No other trait is public.

**Output.**
- The Rich format (styled text) is byte for byte what Python Rich 15 writes for the corpus in `bench/` (corpus 1 to 5, thousands of random vectors); the deviations are listed in `DEVIATIONS.md` and a deviation is removed, never added, within 1.x.
- Plain text has no escape sequence and drops the escape character and the C1 controls of the data.
- JSON is the document `hud/1`: the node set and the keys are closed in 1.x, so a consumer written against `crates/hud/schema/hud-1.json` keeps working; anything new is `hud/2`.
- The environment variables `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, `COLORTERM`, `TERM`, `COLUMNS`, `LINES` and `HUD_FORMAT` keep the precedence written in `hud::resolve`.
- The 73 spinner names never go away; a new animation is a minor change.

**Panics.** No public function panics on any input. The fuzz harness runs seeded inputs for every feature in CI and a panic is a bug fixed in a patch release.

**The two crates.** `hud` re-exports `hud-width` (`hud::width` and six functions). They are released together, `hud-width` first; a major version of `hud-width` is a major version of `hud`. `hud::Align` and `hud::width::Align` are different types with the same three variants.

**MSRV.** 1.85. It is raised only in a minor version, never in a patch, never to a stable release younger than six months, and the raise is announced in `CHANGELOG.md`.

**Features.** There are no Cargo features today. A feature added in 1.x is additive: it never changes the meaning of code that compiles without it, and the default set never shrinks.

**Deprecation.** An item is marked `#[deprecated]` for at least one minor version, with a note that names the replacement, before a major version removes it.

## Checking it

`cargo public-api -p hud -sss` and `cargo public-api -p hud-width -sss` must match `api/hud.txt` and `api/hud-width.txt` (the `api` job of CI). A change to those files is a change to the public API and needs a line in `CHANGELOG.md`.
