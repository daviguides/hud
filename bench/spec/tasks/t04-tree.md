# t04-tree: Tree of a directory-like structure

## Statement

Print the directory-like tree shown in the target: a root "hud/" with the children src/ (lib.rs, console.rs, width.rs), tests/ (golden.rs), Cargo.toml and README.md.

## Run `main`

- Stream: stdout is a pipe (not a terminal). Terminal width is NOT detectable: it comes only from the COLUMNS variable.
- Environment: `COLUMNS=100 NO_COLOR=1 TERM=xterm-256color`
- Check: stdout bytes are identical to the target.

Target:

```text
hud/
├── src/
│   ├── lib.rs
│   ├── console.rs
│   └── width.rs
├── tests/
│   └── golden.rs
├── Cargo.toml
└── README.md
```
