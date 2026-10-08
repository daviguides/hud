| workload | hud median ms | Python Rich ms | hud vs Python (CI) | best verified existing | hud vs best (CI) | threshold | result |
|---|---|---|---|---|---|---|---|
| S1 | 1.713 | 24.52 | 0.070 [0.065, 0.072] | rs_rich 2.03 ms | 0.846 [0.812, 0.874] | <= 0.10x Python, no regression > 25% | pass |
| S2 | 20.139 | 1036.71 | 0.019 [0.019, 0.020] | rich_rust 88.68 ms | 0.227 [0.222, 0.235] | <= 0.80x best, CI <= 1.00 | pass |
| S3 | 1.372 | 682.02 | 0.002 [0.002, 0.002] | composed 7.84 ms | 0.175 [0.173, 0.181] | <= 0.80x best, CI <= 1.00 | pass |
| S4 | 1.681 | 41.13 | 0.041 [0.039, 0.043] | rs_rich 5.70 ms | 0.295 [0.287, 0.304] | <= 0.80x best, CI <= 1.00 | pass |
