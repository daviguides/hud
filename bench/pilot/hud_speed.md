| workload | hud median ms | Python Rich ms | hud vs Python (CI) | best verified existing | hud vs best (CI) | threshold | result |
|---|---|---|---|---|---|---|---|
| S1 | 2.989 | 37.86 | 0.079 [0.077, 0.084] | rs_rich 2.96 ms | 1.010 [0.983, 1.078] | <= 0.10x Python, no regression > 25% | pass |
| S2 | 31.643 | 1608.81 | 0.020 [0.019, 0.020] | rich_rust 133.73 ms | 0.237 [0.231, 0.242] | <= 0.80x best, CI <= 1.00 | pass |
| S3 | 2.175 | 1051.00 | 0.002 [0.002, 0.002] | composed 11.12 ms | 0.196 [0.191, 0.199] | <= 0.80x best, CI <= 1.00 | pass |
| S4 | 2.529 | 59.98 | 0.042 [0.041, 0.043] | rs_rich 8.72 ms | 0.290 [0.283, 0.300] | <= 0.80x best, CI <= 1.00 | pass |
