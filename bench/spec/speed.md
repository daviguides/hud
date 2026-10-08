# Speed workloads

Environment for every speed run (`SPEED_ENV`): `FORCE_COLOR=1 COLORTERM=truecolor TERM=xterm-256color COLUMNS=100`, so every workload reaches the styling and rendering path through a pipe. Output goes to a pipe the harness drains.

| id | Workload | Input | Timed region |
|---|---|---|---|
| S1 | Cold start of a small styled table: process start to first rendered byte | none (the program holds 5 rows of data) | whole process, spawn to first byte read, and spawn to exit |
| S2 | Render a 10 000-row, 5-column table | `cases/speed/s2_table.tsv` (id, name, status, value, note); status cells colored `ok` green, `warn` yellow, `fail` red, `skip` dim; header bold cyan; title `Results`; columns Id and Value right aligned | build + render + write + flush; input already loaded |
| S3 | 100 000 progress updates across 8 tasks | none (12 500 steps per task, update `k` advances task `k mod 8`) | all updates plus one frame render and flush after every 100th update (1 000 frames), 100 columns, columns description / bar (30 wide) / percentage / completed-total |
| S4 | 1 000 lines of mixed markup | `cases/speed/s4_lines.txt` | print each line (`[bold]`, `[red]`, nested, escaped brackets), 100 columns |

## Output verification (before any timing)

A workload counts for a candidate only if its output equals the golden; otherwise it measures different work and is discarded for that candidate (evaluation.md, pilot validity 2). S1, S2, S4: byte-identical to `golden/speed/` (S2, S4 gzipped). S3: the final screen of a 100 column terminal (cursor movement and erase interpreted), text only, equals `golden/speed/s3.screen.txt`, AND the stream shows at least 1 000 distinct values of the first task's completed counter (`task 0 ... N/12500`, ANSI stripped; Python Rich shows 1 001, `golden/speed/s3.frames.json`), so a candidate that never animates and only prints the final frame does not pass; how a library moves the cursor between frames is its own. Differences on S1, S2 or S4 are listed as `documented_deviation` and the workload is then compared against candidates with the same deviation only.

## Adapters

S1: an executable that prints the table and exits. S2-S4: `bench --workload S2|S3|S4 --input FILE [--emit | --warmup W --iterations N]`. `--emit` renders once to stdout. Timed mode writes each iteration's output to stdout and prints one `{"iter": i, "ns": elapsed}` line per measured iteration to stderr; the timer starts after the input is loaded and stops after the flush. Python Rich implements the same protocol in `reference/python/speed/`.

## Method

S1: at least 30 measured runs after 5 discarded, spawned by the harness (`time.perf_counter_ns`), time to first byte and time to exit both recorded. S2-S4: at least 30 measured iterations after 5 warm-ups, inside one process. Median with a 95% bootstrap CI (5 000 resamples); candidate ratios use independent bootstrap of both sample sets. Fewer samples run as a pipeline check and issue no verdict.
