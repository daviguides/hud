# Security policy

## Supported versions

hud is not released yet. The `main` branch is the only supported line until a stable version is published.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private vulnerability reporting for this repository (Security tab, "Report a vulnerability"). Include the version or commit, a minimal program that shows the problem and what you expect to happen.

The maintainer reads every report. A confirmed problem is fixed on `main` first and disclosed with the fix.

## What counts

hud writes bytes to a terminal, so the interesting classes are:

- **Escape sequence injection**: text from an untrusted source (a file name, a log line, a user name) that makes the terminal do something the program did not ask for. Markup and escape handling are meant to prevent this; `hud::escape` is the tool for untrusted values.
- **Panics and unbounded work** on hostile input: a string, a table or a markup document that crashes the program or takes quadratic time. The fuzz harness in `bench/` tests for panics; reports of inputs that get past it are welcome.
- **Memory safety**: the workspace forbids `unsafe`, so a memory-safety bug would come from a dependency. `cargo deny` checks advisories in CI.

Behavior that differs from Python Rich is not a vulnerability; see `DEVIATIONS.md`.
