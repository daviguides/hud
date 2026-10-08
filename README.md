# hud

Functional terminal UX for Rust. One crate for styled output, tables, panels, trees and progress, built to be read at a glance.

**Status: pre-alpha.** The name is reserved; there is no API yet.

## Goals

- **Speed.** Fast startup and fast render, measured against existing Rust crates and Python Rich.
- **Correctness.** ANSI output verified byte for byte against golden captures.
- **Assertiveness.** Correct text width for Unicode (CJK, emoji, combining characters), wrapping and truncation that never split a grapheme, and capability detection (truecolor, 256, none, `NO_COLOR`, no TTY).
- **DX.** An API a developer, or an LLM with only the docs, gets right on the first try.

## Why

Python has Rich. Rust has good pieces (`owo-colors`, `indicatif`, `comfy-table`) and a few young Rich ports, but no unified, measured, ergonomic library. hud is built by comparing what exists, borrowing what works, and deciding with benchmarks.

## License

MIT OR Apache-2.0.
