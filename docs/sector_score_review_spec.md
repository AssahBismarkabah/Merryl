# Sector Score Review

Version: 0.2
Date: 2026-05-27
Status: PDB-2 and PDB-3.5 complete; sector score remains map-only until stronger evidence exists

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_3_backtest_validation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/industry_specific_validation_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/sector_formula_decision_checkpoint_spec.md`

## 1. Purpose

This document records the PDB-2 sector score review.

Question:

```text
Is the current sector score strong enough to treat as a forward-return signal,
or should it remain a market-map and attention layer?
```

This protects the original top-down flow:

```text
Market regime -> sector rotation -> industry/theme strength -> stock leadership -> watchlist
```

The review uses stored SQLite data only. It does not fetch new prices, change scoring formulas, or add a new provider.

## 2. Implementation Summary

The existing backtest workflow now adds `sector_component_*` rows.

Meaning:

```text
sector_component_return_5d decile 10 = sectors with the strongest same-day 5D return.
sector_component_return_20d decile 10 = sectors with the strongest same-day 20D return.
sector_component_return_60d decile 10 = sectors with the strongest same-day 60D return.
sector_component_relative_return_vs_spy decile 10 = sectors strongest vs SPY.
sector_component_relative_volume decile 10 = sectors with highest relative volume.
sector_component_breadth decile 10 = sectors with strongest breadth.
sector_component_rank_change decile 10 = sectors with strongest stored rank improvement.
```

Forward returns are still sector ETF forward returns.

Primary relative return remains:

```text
sector ETF forward return - SPY forward return
```

This lets us review the current sector score components without changing the formula blindly.

## 3. Backtest Run

Command:

```text
cargo run -- run backtest --from 2025-07-01 --to 2026-05-26
```

Artifacts:

```text
reports/backtests/2025-07-01_2026-05-26_backtest_report.md
exports/backtests/2025-07-01_2026-05-26_backtest_summary.csv
```

Observation counts:

```text
sector observations: 11429
sector component observations: 80003
stock observations: 51950
industry validation observations: 51950
```

## 4. Total Sector Score Result

Average sector ETF forward return versus SPY:

| Horizon | Sector Decile 10 | Sector Decile 1 | Decile 10 Minus Decile 1 |
|---:|---:|---:|---:|
| 1D | 0.04% | 0.05% | -0.01% |
| 5D | 0.14% | 0.30% | -0.16% |
| 10D | 0.27% | 0.14% | 0.13% |
| 20D | 0.02% | -0.42% | 0.44% |
| 60D | -0.17% | 1.61% | -1.79% |

Interpretation:

- The total sector score is useful for market mapping and attention.
- It is not consistently predictive across horizons.
- The 20D result is constructive.
- The 60D result is weak and suggests the current total sector score can over-rank extended sectors.

## 5. Component Review

Decile 10 minus decile 1 average relative return:

| Component | 1D | 5D | 10D | 20D | 60D |
|---|---:|---:|---:|---:|---:|
| 5D return | 0.05% | -0.08% | 0.36% | 0.18% | 0.58% |
| 20D return | 0.03% | -0.14% | -0.18% | 0.44% | -0.63% |
| 60D return | -0.04% | 0.30% | 0.54% | 0.27% | -4.85% |
| Relative return vs SPY | 0.03% | -0.14% | -0.18% | 0.44% | -0.63% |
| Relative volume | 0.03% | -0.01% | 0.43% | 0.59% | 1.06% |
| Breadth | 0.03% | -0.09% | 0.05% | 0.21% | -0.27% |
| Rank change | -0.02% | -0.17% | 0.06% | 0.10% | -0.30% |

Findings:

- Relative volume has the cleanest positive behavior from 10D through 60D.
- 5D return adds useful short/intermediate information, especially from 10D through 60D.
- 60D return is useful over 5D-20D but weak over 60D, which may reflect overextension or mean reversion.
- 20D return and relative return vs SPY behave the same in daily cross-section because SPY's same-day 20D return is constant across sectors.
- Breadth is not strong enough in the current form to carry sector ranking.
- Rank change is weak and should not be treated as a predictive component yet.

## 6. Formula Issue Found And Closed

PDB-2 found that the sector formula had a rank-change weight in configuration, but the score used a neutral rank-change placeholder.

Reason:

```text
rank_change is known only after sector ranks are calculated.
```

Old behavior:

```text
rank_change is stored and reported,
but it is not truly contributing to the sector score.
```

Initial PDB-2 decision:

```text
Do not force rank_change into the score now.
```

Adding it directly would require a second-pass ranking design and a fresh validation cycle.

PDB-3.5 decision:

```text
Remove the neutral rank-change contribution from sector scoring.
Renormalize the remaining active sector weights.
Keep rank_change stored and reported, but do not score it yet.
```

Reason:

```text
The old neutral placeholder made the formula semantics misleading.
Option B2 fixed the semantics without materially disrupting sector ranking, stock validation, or the latest watchlist.
```

## 7. Decision

Decision:

```text
Keep sector ranking as a market-map and attention layer.
Label it as map-only / not yet a proven forward-return predictor.
Do not change sector scoring weights in this pass.
```

Reason:

- The total sector score is mixed.
- Relative volume appears useful enough to preserve.
- Momentum windows show different behavior by horizon.
- Breadth and rank change need more evidence or redesign.
- Changing weights now would be blind optimization from one window.

Current product stance:

```text
Use sector ranking to decide where to look first.
Use industry/theme and stock validation to decide what is more actionable.
Do not present sector decile rank as a trade signal.
```

Daily report handling:

```text
The daily report labels sector ranking as a market-map and attention layer.
This keeps the sector tables useful without presenting the current sector score as a proven forward-return signal.
```

## 8. Next Work

PDB-2 is complete.

PDB-3 market regime V1 review is complete.

PDB-3.5 sector formula decision checkpoint is complete.

The next pre-dashboard item is:

```text
PDB-4: Catalyst/earnings decision
```

The sector formula decision is closed for this pass. Sector ranking remains map-only until stronger evidence exists.
