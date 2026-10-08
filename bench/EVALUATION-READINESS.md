# hud: first full evaluation, readiness

State after v0.6 part A. The thresholds and decision rules are the pre-registered ones in `~/work/projects/daviguides/hud/foundation/evaluation.md`; nothing below changes one. This file lists what is still to be done before hud can receive a verdict, what only the user can do, and the commands in order.

## What is done and what is open

| Axis | State | Evidence |
|---|---|---|
| 1 Correctness (gate) | **measured, passes**: 210 / 210 = 100%, style and markup 100%, fuzz 0 panics on all nine features (80 000 inputs each) | `candidates/hud/RESULTS.md` (v0.6), `pilot/hud/correctness.json`, `pilot/hud/fuzz_v06.json` |
| 2 Assertiveness (gate) | **measured, passes**: width 99.2%, 0 grapheme splits, 0 misaligned rows, capability 40 / 40 | `pilot/hud/width.json`, `pilot/hud/capability.json` |
| 3 Speed | **open**. Never measured on an idle machine in this strict sense; the records of v0.3 to v0.5 were taken at load 1.7 to 3.3 and read as same-session ratios. Part A took no timing (load 3.3 to 12) | `pilot/hud/speed_conditions_v05.json` |
| 4 DX | **partly measured**: 8 / 8 tasks pass, LOC median 11 (Python Rich 12, rs-rich 15), name parity 28 / 40 = 70% by name, 0 padded functions, adoption +2.0 s / +219 KB / 6 crates, docs 100%. **Open: first-try success**, which needs real models | `pilot/hud/tasks.json`, `pilot/hud/api.json`, `pilot/hud/dx_dry_run_v06.txt` |
| Verdict | not issued: speed and first-try are open, so the engine and API entries stay INCONCLUSIVE (evaluation.md, pilot validity 7) | |

## What the user must do

1. **Quiet the machine for the speed pass.** Evaluation rule 1: one machine, no concurrent builds or other benchmark arms. In practice: close the other Claude Code sessions, editors and browsers, stop anything compiling, and start only when `uptime` shows a 1-minute load average under 1.0 for several minutes. The driver (`scripts/pilot.py`, `wait_idle`) waits for no `cargo`/`rustc` process and a load average of at most 4.0, and after 30 minutes it proceeds anyway and writes `PROCEEDED UNDER LOAD` in the log; 4.0 is looser than rule 1, and that line marks a row taken under load, which has to be re-measured alone, so keep the machine quiet until the log is clean. Do not touch the machine while it runs (the duration was not measured: each workload is 30 iterations for six candidates and Python Rich, plus the adoption builds).
2. **Pin the two models and confirm the token budget for the DX test.** `bench/spec/dx-models.json` has `primary.id` and `stress.id` set to `null` and the runner refuses a real run until both are set. Per the decided protocol: your primary daily model (5 repeats per task, the 80% gate is read on it) and one smaller model (3 repeats, reported only). Also fill `estimate.prices_usd_per_mtok` if you want a dollar figure: without prices the estimate is tokens only.

   Estimate printed by `dx_runner.py estimate` (assumptions in `dx-models.json`: 12 turns, 10 doc pages of about 4 300 tokens and a 1 067-token README read per run, 6 000 tokens of system context, 3 000 output tokens per run):

   | Model | Runs | Input tokens | Output tokens | Cost |
   |---|---|---|---|---|
   | primary | 40 (8 tasks x 5) | 15 243 720 | 120 000 | not computed (no prices) |
   | stress | 24 (8 tasks x 3) | 9 146 232 | 72 000 | not computed (no prices) |

   Input tokens are counted without prompt caching, so they are an upper bound for the cached part. The runner prints this estimate again before the first call and asks for confirmation (`--yes` skips the question).

## Steps, in order

Everything runs from `bench/` with `.venv/bin/python`; the candidates' release binaries are built in place.

**0. Preflight (the machine need not be quiet).**

```bash
cargo test --workspace && cargo xtask check-layers && cargo clippy --workspace --all-targets -- -D warnings
(cd candidates/hud && cargo build --release)
.venv/bin/python -m pytest -q                 # harness self-tests (rule 4)
.venv/bin/python scripts/pilot.py static      # correctness, width, capability, tasks, LOC, API for hud and the five pilot candidates, under the fixed pty reader
```

`pilot.py` has a `hud` entry (changelog 27). Check that the static records of the five pilot candidates are what `PILOT-RESULTS.md` says (capability 31, 31, 5, 22, 22 of 40 were confirmed under the fixed reader in v0.6).

**1. Speed pass (quiet machine).** Adoption cost and S1 to S4 for hud, the five pilot candidates and Python Rich, one at a time:

```bash
.venv/bin/python scripts/pilot.py timed        # writes pilot/<candidate>/speed/S*.json, adoption_*.json, pilot/speed_conditions.log
grep -c "PROCEEDED UNDER LOAD" pilot/speed_conditions.log   # must print 0, else rerun the affected rows alone
.venv/bin/python scripts/pilot.py unverified   # workloads whose output differs from the golden: informational, never ranked
```

Add `hud` to `NAMES` and `LABEL` in `scripts/pilot_report.py` (it reads `pilot/hud/speed/S*.json`, which `pilot.py timed` writes), then `.venv/bin/python scripts/pilot_report.py` for the cross-candidate tables.

Reading the speed thresholds (evaluation.md, axis 3):
- S1 at most 0.10x Python Rich, median with its 95% bootstrap CI.
- S2, S3, S4 at most 0.80x the best existing Rust candidate with CI upper bound at most 1.00, and no workload more than 25% slower than it. The evaluation text says "the best existing Rust candidate that passed both gates"; no pilot candidate passed both (`PILOT-RESULTS.md`), so the reference is the fastest pilot candidate whose output is verified equal for that workload, which is how the absolute targets of `features.md` (S2 71.6 ms, S3 5.97 ms, S4 4.70 ms) were derived. State this reading in the report; it does not change a threshold.
- A workload counts only if hud's output is EQUAL to the golden (`speed.py verify`; S1 to S4 are EQUAL today, S3 with 1 000 of 1 000 frames).

**2. DX first-try (needs the models and the budget).**

```bash
.venv/bin/python scripts/dx_runner.py estimate                       # tokens, again
.venv/bin/python scripts/dx_runner.py run --execute --primary-model <ID> --stress-model <ID>   # runs go to results/dx/runs.jsonl
.venv/bin/python scripts/dx_runner.py report results/dx/runs.jsonl   # first-try success per model, per task, CI over tasks
```

The runner gives each run only the docs mirror and the README, an agent restricted to Read, Grep and Glob, one attempt, a transcript audit (a run that reads outside the mirror is a `protocol_violation`, discarded and rerun), and classifies every failure before it counts (`candidate_failure`, `harness_error`, `unsupported`). Dry run with the mock agent: 8 of 8 tasks `success`, 0 `harness_error`, 0 `protocol_violation` (`pilot/hud/dx_dry_run_v06.txt`). The Claude CLI invocation (`ClaudeCli.command`) was built from `claude --help` and has **never run**: expect the first real run to need a smoke test with `--tasks t06-markup --repeats 1` on the stress model before the full run. The runner supports only `hud` as a candidate; first-try on the other candidates is not needed for the API verdict, because each pilot candidate already fails another DX threshold (`PILOT-RESULTS.md`: rs-rich +2.41 MB and 84.3% docs, rich_rust +2.04 MB, richrs name parity 68%, rich-rs 82 dependencies, composed 3.1x LOC and 12.5% parity), so rule 1 of the API decision cannot hold for any of them.

**3. Verdict, per `evaluation.md` "Decision rules" and "Pilot validity".**

| Entry | GO when | NO-GO when | INCONCLUSIVE when |
|---|---|---|---|
| Engine: own engine as a candidate | axis 1 and 2 gates pass (they do today) and the S1, S2, S3, S4 thresholds above hold with the CI bound named in each | a gate fails, or a speed threshold fails with the CI wholly on the wrong side, and pilot validity 1 to 6 held | the CI straddles a threshold: report how many more samples separate it; or any speed row was measured under load |
| API: hud's own API | first-try success at least 80% on the primary model over 8 tasks x 5 repeats; LOC median at most 12 (1.5x Python Rich) and at most 0.80x rs-rich's 15 (= 12); name parity at least 70% by name **and** a recognizable signature; 0 padded functions; adoption 15 s / 1.5 MB / 60 deps (targets 3 s / 400 KB / 10); docs 100% | a threshold fails with valid runs | first-try not run, or fewer than 8 x 5 runs, or the CI over tasks straddles 80% |

Name parity today is 28 / 40 = 70% **by existence of the name**; evaluation.md asks for a recognizable signature too, so a signature review of the 28 is part of this step (v0.7 re-reviews it). The stress model adds no threshold.

Write the result as `bench/HUD-RESULTS.md` in the style of `PILOT-RESULTS.md`: per-axis numbers with CIs, the `documented_deviation` and `harness_error` lists (`DEVIATIONS.md`, the runner's `excluded`), and the verdict with the rule that produced it.

## Checklist

- [ ] `cargo test --workspace`, `cargo xtask check-layers`, clippy and fmt green on the commit to be measured; the commit hash is recorded in the report
- [ ] `pytest -q` in `bench/` green (the pty reader fix is in: changelog 24)
- [ ] `pilot.py static` run for all six; the five pilot candidates unchanged against `PILOT-RESULTS.md`
- [ ] User quiet-machine confirmation; `uptime` load under 1.0; no `cargo`/`rustc`, editor builds or other sessions working
- [ ] `pilot.py timed` complete; `speed_conditions.log` has no `PROCEEDED UNDER LOAD`; hud's four outputs EQUAL
- [ ] `pilot_report.py` includes hud; S1 against Python Rich and S2 to S4 against the fastest verified pilot candidate, with CI bounds
- [ ] `dx-models.json`: both model ids pinned by the user, prices filled if a dollar figure is wanted, token budget confirmed
- [ ] Smoke test of the Claude CLI invocation on one task before the full run
- [ ] Full DX run: 8 x 5 on the primary model, 8 x 3 on the stress model; every `harness_error` and `protocol_violation` rerun
- [ ] Signature review of the 28 matched Rich names
- [ ] `HUD-RESULTS.md` written with the verdict and the rule that produced it; raw outputs kept; thresholds untouched
