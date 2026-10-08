### correctness

| candidate | style | markup | table | panel | tree | progress | error | overall | style+markup 100% | gate 98% |
|---|---|---|---|---|---|---|---|---|---|---|
| rich_rust 0.2.3 | 30/30 | 30/30 | 3/30 | 5/30 | 26/30 | 0/30 | 7/30 | 101/210 = 48.1% | pass | FAIL |
| rs-rich 0.0.9 | 30/30 | 30/30 | 30/30 | 30/30 | 30/30 | 30/30 | 30/30 | 210/210 = 100.0% | pass | pass |
| richrs 0.2.1 | 13/30 | 16/30 | 0/30 | 2/30 | 2/30 | 0/30 | 0/30 | 33/210 = 15.7% | FAIL | FAIL |
| rich-rs 1.3.0 | 16/30 | 12/30 | 5/27 | 12/30 | 7/30 | 7/30 | 7/30 | 66/207 = 31.9% | FAIL | FAIL |
| composed (owo-colors + comfy-table + indicatif) | 17/30 | 25/30 | 1/30 | 21/30 | 30/30 | 3/30 | 30/30 | 127/210 = 60.5% | FAIL | FAIL |

### assertiveness

| candidate | width agreement (gate 99%) | grapheme splits fold / truncate (gate 0) | misaligned table rows (gate 0) | capability (gate 40/40) |
|---|---|---|---|---|
| rich_rust 0.2.3 | 318/500 = 63.6% (FAIL) | 797 / 623 (FAIL) | 22/98 rows (FAIL) | 31/40 (FAIL) |
| rs-rich 0.0.9 | 500/500 = 100.0% (pass) | 176 / 119 (FAIL) | 0/97 rows (pass) | 31/40 (FAIL) |
| richrs 0.2.1 | 439/500 = 87.8% (FAIL) | unsupported / 0 (FAIL) | 11/82 rows, 1 of 12 tables unsupported (FAIL) | 5/40 (FAIL) |
| rich-rs 1.3.0 | 439/500 = 87.8% (FAIL) | 797 / 217 (FAIL) | 6/97 rows (FAIL) | 22/40 (FAIL) |
| composed (owo-colors + comfy-table + indicatif) | 439/500 = 87.8% (FAIL) | 0 / 0 (pass) | 6/97 rows (FAIL) | 22/40 (FAIL) |

### speed

| workload | candidate | verified | median ms | vs Python Rich (CI) | vs best verified Rust candidate (CI) |
|---|---|---|---|---|---|
| S1 | Python Rich 15.0.0 | reference | 24.52 | 1 | n/a |
| S1 | rich_rust 0.2.3 | yes | 23.90 | 0.975 [0.914, 0.994] | 11.80 [11.32, 12.02] |
| S1 | rs-rich 0.0.9 | yes | 2.03 | 0.083 [0.078, 0.084] | (best) |
| S1 | richrs 0.2.1 | NO: output differs from the golden, workload discarded; informational timing of different work | 4.09 | 0.167 [0.158, 0.172] (not comparable) | not ranked |
| S1 | rich-rs 1.3.0 | NO: output differs from the golden, workload discarded; informational timing of different work | 5.26 | 0.215 [0.192, 0.233] (not comparable) | not ranked |
| S1 | composed (owo-colors + comfy-table + indicatif) | NO: output differs from the golden, workload discarded; informational timing of different work | 1.88 | 0.077 [0.072, 0.079] (not comparable) | not ranked |
| S2 | Python Rich 15.0.0 | reference | 1036.71 | 1 | n/a |
| S2 | rich_rust 0.2.3 | yes | 88.68 | 0.086 [0.085, 0.086] | (best) |
| S2 | rs-rich 0.0.9 | yes | 132.03 | 0.127 [0.126, 0.128] | 1.49 [1.47, 1.50] |
| S2 | richrs 0.2.1 | NO: output differs from the golden, workload discarded; informational timing of different work | 29.82 | 0.029 [0.028, 0.029] (not comparable) | not ranked |
| S2 | rich-rs 1.3.0 | NO: output differs from the golden, workload discarded; informational timing of different work | 600.51 | 0.579 [0.575, 0.585] (not comparable) | not ranked |
| S2 | composed (owo-colors + comfy-table + indicatif) | NO: output differs from the golden, workload discarded; informational timing of different work | 21.41 | 0.021 [0.021, 0.021] (not comparable) | not ranked |
| S3 | Python Rich 15.0.0 | reference | 682.02 | 1 | n/a |
| S3 | rich_rust 0.2.3 | yes | 268.65 | 0.394 [0.387, 0.396] | 34.25 [33.89, 34.75] |
| S3 | rs-rich 0.0.9 | yes | 88.53 | 0.130 [0.127, 0.130] | 11.29 [11.15, 11.44] |
| S3 | richrs 0.2.1 | yes | 7.98 | 0.012 [0.011, 0.012] | 1.02 [1.01, 1.04] |
| S3 | rich-rs 1.3.0 | yes | 2868.96 | 4.207 [4.077, 4.263] | 365.75 [355.37, 372.38] |
| S3 | composed (owo-colors + comfy-table + indicatif) | yes | 7.84 | 0.012 [0.011, 0.012] | (best) |
| S4 | Python Rich 15.0.0 | reference | 41.13 | 1 | n/a |
| S4 | rich_rust 0.2.3 | yes | 12.86 | 0.313 [0.303, 0.326] | 2.26 [2.21, 2.31] |
| S4 | rs-rich 0.0.9 | yes | 5.70 | 0.139 [0.133, 0.145] | (best) |
| S4 | richrs 0.2.1 | NO: output differs from the golden, workload discarded; informational timing of different work | 3.40 | 0.083 [0.080, 0.086] (not comparable) | not ranked |
| S4 | rich-rs 1.3.0 | NO: output differs from the golden, workload discarded; informational timing of different work | 17.14 | 0.417 [0.401, 0.435] (not comparable) | not ranked |
| S4 | composed (owo-colors + comfy-table + indicatif) | NO: output differs from the golden, workload discarded; informational timing of different work | 1.85 | 0.045 [0.043, 0.047] (not comparable) | not ranked |

### dx

| candidate | tasks passing (runs) | LOC median (passing) / ratio vs Python 12 | name parity (existence by name) | None-padding public fns | first-try |
|---|---|---|---|---|---|
| rich_rust 0.2.3 | 7/8 (9/10 runs) | 20 / 1.67x (FAIL) | 27/40 = 67.5% (FAIL) | 4 (FAIL) | not measured (agent runner not built) |
| rs-rich 0.0.9 | 8/8 (10/10 runs) | 15.0 / 1.25x (pass) | 35/40 = 87.5% (pass) | 39 (FAIL) | not measured (agent runner not built) |
| richrs 0.2.1 | 3/8 (5/10 runs) | 14 / 1.17x (pass) | 27/40 = 67.5% (FAIL) | 2 (FAIL) | not measured (agent runner not built) |
| rich-rs 1.3.0 | 7/8 (9/10 runs) | 16 / 1.33x (pass) | 38/40 = 95.0% (pass) | 91 (FAIL) | not measured (agent runner not built) |
| composed (owo-colors + comfy-table + indicatif) | 8/8 (10/10 runs) | 37.0 / 3.08x (FAIL) | 5/40 = 12.5% (FAIL) | 0 (pass) | not measured (agent runner not built) |

### adoption

| candidate | project | added compile s (gate 15) | added stripped bytes (gate 1 500 000) | transitive deps (gate 60) | pass |
|---|---|---|---|---|---|
| rich_rust 0.2.3 | hello | 4.07 | 2,037,072 | 60 | FAIL |
| rs-rich 0.0.9 | hello | 6.00 | 2,414,976 | 50 | FAIL |
| rs-rich 0.0.9 | hello_nodefault | 4.07 | 2,264,704 | 21 | FAIL |
| richrs 0.2.1 | hello | 3.85 | 155,264 | 44 | pass |
| rich-rs 1.3.0 | hello | 8.14 | 392,288 | 82 | FAIL |
| composed (owo-colors + comfy-table + indicatif) | hello | 1.58 | 311,296 | 25 | pass |

### docs

| candidate | crate | documented items | with an example | doctests (passed / failed / ignored) |
|---|---|---|---|---|
| rich_rust 0.2.3 | rich_rust | 1016 items = 88.1% | 24 = 2.9% | 30 / 0 / 37 |
| rs-rich 0.0.9 | rs-rich | 984 items = 84.3% | 4 = 0.5% | 4 / 0 / 0 |
| richrs 0.2.1 | richrs | 616 items = 100.0% | 0 = 0.0% | 0 / 0 / 23 |
| rich-rs 1.3.0 | rich-rs | 996 items = 82.2% | 80 = 10.0% | 107 / 0 / 35 |
| composed (owo-colors + comfy-table + indicatif) | comfy-table | 88 items = 80.0% | 4 = 100.0% | 33 / 0 / 0 |
| composed (owo-colors + comfy-table + indicatif) | indicatif | 53 items = 88.3% | 5 = 21.7% | 14 / 0 / 7 |
| composed (owo-colors + comfy-table + indicatif) | owo-colors | 138 items = 100.0% | 20 = 5.5% | 62 / 0 / 0 |
