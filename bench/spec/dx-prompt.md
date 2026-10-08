# DX first-try prompt (template)

One fresh agent per run: no memory of earlier runs, no repository access, no web. It may read files under `./docs/` (the docs mirror of the crate at the pinned version, `scripts/docs_mirror.py`) and nothing else; it cannot compile or run anything. Its final message is the complete `src/main.rs`, which is the one and only attempt.

The prompt differs between candidates only in `{crate}`, `{version}` and the docs path; the task block is `spec/tasks/<task-id>.md`, identical for every candidate.

```text
You are writing one Rust program using the crate `{crate}` version {version}.

The documentation of the crate is in ./docs/ (rendered rustdoc pages) and ./README.md. Read what you need
from there. You cannot browse the web, read any other file, compile or run code.

Rules:
- The program is a single file, src/main.rs, for edition 2024, with `{crate} = "={version}"` as its only dependency.
- Produce the output through the crate's API. Reproducing the target text in string literals is not allowed.
- Do not add other dependencies. Do not use unsafe.

The task:

{contents of spec/tasks/<task-id>.md}

Reply with the complete contents of src/main.rs in a single ```rust block and nothing after it.
```
