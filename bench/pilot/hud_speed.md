| workload | hud median ms | Python Rich ms | hud vs Python (CI) | best verified existing | hud vs best (CI) | threshold | result |
|---|---|---|---|---|---|---|---|
| S1 | 1.741 | 25.01 | 0.070 [0.068, 0.071] | rs_rich 2.06 ms | 0.846 [0.813, 0.874] | <= 0.10x Python, no regression > 25% | pass |
| S2 | 21.057 | 1032.20 | 0.020 [0.020, 0.021] | rich_rust 88.57 ms | 0.238 [0.232, 0.244] | <= 0.80x best, CI <= 1.00 | pass |
| S3 | 1.468 | 678.36 | 0.002 [0.002, 0.002] | composed 7.41 ms | 0.198 [0.194, 0.203] | <= 0.80x best, CI <= 1.00 | pass |
| S4 | 1.782 | 42.51 | 0.042 [0.040, 0.044] | rs_rich 5.73 ms | 0.311 [0.297, 0.317] | <= 0.80x best, CI <= 1.00 | pass |
