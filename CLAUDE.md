# hud

Functional terminal UX library for Rust. Pre-alpha. Goal: speed, correctness, Unicode-width assertiveness, DX. Not decoration.

## Layout

```
hud/                                  # this repo, code only
~/work/projects/daviguides/hud/       # project knowledge
├── foundation/                       # vision, positioning, GO/NO-GO criteria
├── references/repos/                 # cloned competitors (gitignored in this repo)
├── references/studies/               # per-crate studies and benchmarks
├── consult/                          # multi-model consultation rounds
└── sessions/{logs,prompts,contexts}/ # handoffs, logs, graduated decisions
```

## Rules

- Name is `hud` (decided). Not a Rich clone: reuse and compose what exists when it measures better.
- Every claim is measured. GO/NO-GO criteria are written before the benchmark runs and cover four axes: speed, correctness, width/Unicode assertiveness, DX.
- Architecture follows Shodo Rust conventions: layers with strict dependency direction, services free of terminal and CLI framework imports.
- Edition 2024, clippy `all = deny`.

## Development

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```
