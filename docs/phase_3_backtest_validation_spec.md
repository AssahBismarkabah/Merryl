# Phase 3 Backtest Validation

Version: 0.6
Date: 2026-05-27
Status: Validation checkpoint; PDB-1, PDB-2, PDB-3, and PDB-3.5 complete

## 1. Purpose

This document records what the first Phase 3 backtest says about Merryl before we build the dashboard.

The goal is to protect the original product intent:

```text
Show where market participation is concentrating,
map that concentration from market regime to sector to industry/theme to stock,
then produce a small list of liquid, chart-worthy names with explainable reasons.
```

The dashboard should not be built on top of unreviewed signals. The backtest must tell us what appears useful, what is weak, and what should be fixed before making the system more visual.

## 2. Sources Reviewed

Core documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/industry_specific_validation_spec.md`
- `docs/sector_score_review_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/sector_formula_decision_checkpoint_spec.md`

Backtest artifacts:

- `reports/backtests/2025-07-01_2026-05-26_backtest_report.md`
- `exports/backtests/2025-07-01_2026-05-26_backtest_summary.csv`

Database state at validation time:

```text
score dates: 227
sector scores: 2497
industry scores: 28829
stock scores: 11350
watchlist rows: 5675
backtest results: 7
```

Backtest observation counts:

```text
sector observations: 11429
sector component observations: 80003
stock observations: 51950
industry validation observations: 51950
```

## 3. Scope Boundary

This validation tests score behavior, not trade profitability.

It does not model:

- Entry and exit rules.
- Position sizing.
- Transaction costs.
- Slippage.
- Taxes.
- Portfolio constraints.
- Maximum adverse excursion.
- Maximum favorable excursion.
- Delisted-stock survivorship corrections.
- Future sector membership changes.

This is acceptable at this stage because Merryl is still a market map and watchlist engine, not an auto-trading system.

## 4. Phase 3 Acceptance Mapping

Phase 3 asked whether we can answer:

```text
Do high-ranked sectors outperform low-ranked sectors?
Do high-ranked stocks outperform their sector?
Can we identify false positives?
Can we decide whether scoring weights need revision?
```

Current answer:

| Question | Answer | Decision |
|---|---|---|
| High-ranked sectors outperform low-ranked sectors? | Mixed. The current sector score does not show stable high-decile leadership across all horizons. PDB-2 component review confirms sector score should remain map-only for now. | Do not treat sector score as proven yet. |
| High-ranked stocks outperform their sector? | Yes. Stock decile 10 beats decile 1 across all tested horizons on average relative return. | Stock ranking is useful enough to preserve and visualize later. |
| Can we identify false positives? | Yes. Sector decile 10 is weak at 60D and inconsistent at 1D/5D. Stock hit rates are positive but not strong enough to imply direct trade signals. | Keep watchlist framing, not trade-signal framing. |
| Should scoring weights be revised? | Sector and industry logic need review before dashboard. Stock scoring should not be over-optimized from one backtest window. | Improve industry/theme scoring first; avoid blind weight fitting. |

## 5. Sector Score Findings

Sector decile 10 is not consistently stronger than sector decile 1.

| Horizon | Decile 10 hit rate | Decile 10 avg relative | Decile 1 avg relative | Decile 10 minus decile 1 |
|---:|---:|---:|---:|---:|
| 1D | 49.1% | 0.04% | 0.05% | -0.01% |
| 5D | 51.4% | 0.14% | 0.30% | -0.16% |
| 10D | 50.9% | 0.27% | 0.14% | 0.13% |
| 20D | 42.8% | 0.02% | -0.42% | 0.44% |
| 60D | 42.2% | -0.17% | 1.61% | -1.79% |

Interpretation:

- The sector score has some useful short/intermediate behavior, especially around 10D and 20D relative spread.
- It is not monotonic. The strongest decile is not reliably the best sector group.
- The 60D result is especially weak because low-ranked sectors outperformed high-ranked sectors on average.
- This means the sector score is not yet strong enough to present as a confident predictive signal.

Decision:

```text
Keep sector ranking as a market-map and attention tool.
Do not present it as a proven forward-return predictor yet.
```

## 6. Stock Score Findings

Stock decile 10 shows useful forward behavior relative to sector ETF and SPY.

| Horizon | Decile 10 hit rate | Decile 10 avg vs sector | Decile 1 avg vs sector | Decile 10 minus decile 1 |
|---:|---:|---:|---:|---:|
| 1D | 51.5% | 0.32% | -0.03% | 0.35% |
| 5D | 53.8% | 1.01% | 0.11% | 0.90% |
| 10D | 53.4% | 1.27% | 0.03% | 1.25% |
| 20D | 53.7% | 2.65% | 0.31% | 2.33% |
| 60D | 59.2% | 10.98% | 2.22% | 8.76% |

Stock decile 10 also outperformed SPY on average:

| Horizon | Avg vs sector | Median vs sector | Avg vs SPY | Median vs SPY |
|---:|---:|---:|---:|---:|
| 1D | 0.32% | 0.07% | 0.42% | 0.02% |
| 5D | 1.01% | 0.43% | 1.36% | 0.56% |
| 10D | 1.27% | 0.37% | 1.89% | 0.84% |
| 20D | 2.65% | 0.82% | 3.25% | 0.87% |
| 60D | 10.98% | 3.82% | 10.66% | 5.64% |

Interpretation:

- The stock opportunity score is doing the most useful work in the current system.
- Top-ranked stocks beat low-ranked stocks across every tested horizon.
- The strongest behavior appears at 20D and 60D, which supports the "stocks worth charting" workflow more than a one-day trading signal.
- Hit rates are above 50%, but not high enough to frame output as automatic entries.

Decision:

```text
Preserve stock scoring as the current strongest validated layer.
Keep the output framed as watchlist and chart review, not trade instruction.
```

## 7. Limitation Triage Before Phase 4

The current `implementation_spec.md` limitations are not equal in priority.

| Limitation | Phase 4 blocker? | Reason |
|---|---:|---|
| Market regime coverage is ETF-proxy based | Addressed for first build | PDB-3 and PDB-3.6 state the exact coverage: SPY, QQQ, IWM, DIA, TLT, GLD, and USO are included; VIX, DXY, US10Y, macro calendar, credit, and liquidity data are not yet included. It still must not be presented as a full macro model. |
| First valid score date has no prior rank-change baseline | No | This is expected for rolling historical windows. It affects only the first scored date. |
| Catalyst/news source | Addressed for recent news | PDB-4 connects real Alpaca News headlines for current watchlist symbols. Structured earnings calendar data remains explicitly not connected. |
| Industry/theme bridge | No | Hardened and validated in PDB-1. It should stay as an attention and confirmation layer, not a trade signal. |
| Sector score is mixed | Partial | Reviewed in PDB-2. Keep it as map-only until stronger evidence or regime-aware validation exists. |
| Backtest is not trade profitability | Addressed for pre-dashboard | PDB-5 stores validation scope in metrics and adds report wording explaining what the backtest proves and does not prove. |

## 8. Decision Before Dashboard

Do not start the full Phase 4 dashboard yet.

The original scoring-quality pass focused on the industry/theme layer is complete. PDB-1 then validated the industry/theme bridge, PDB-2 reviewed the sector score, and PDB-3 clarified/hardened Market Regime V1.

Current reason to keep holding full dashboard work:

```text
The stock layer is useful.
The industry/theme layer is useful enough to preserve.
The sector layer is still map-only / not yet a proven forward-return predictor.
The current market regime coverage is ETF-proxy based and now explicit.
The catalyst/news layer is connected for recent Alpaca News. Structured earnings calendar data remains explicitly not connected.
```

If we build the dashboard immediately, we risk visualizing pending catalyst/earnings data and mixed sector behavior as if they are mature signals.

## 9. Recommended Next Implementation Work

Initial recommended work was:

```text
Industry/theme scoring hardening before dashboard.
```

Implementation status:

```text
Completed on 2026-05-27.
```

The industry/theme score now includes:

- 5D return.
- 20D return.
- 60D return.
- Relative return vs sector ETF.
- Relative return vs SPY.
- Relative volume.
- Breadth above 20D moving average.
- Breadth above 50D moving average.
- 20D-high participation rate.
- Member count.

Completed scope:

1. Kept the public CLI unchanged.
2. Improved industry scores without adding a new provider.
3. Used existing SQLite daily prices and symbol sector/industry mappings.
4. Added transparent industry score components:
   - 5D return.
   - 20D return.
   - 60D return.
   - relative return vs sector ETF.
   - relative volume where enough stock history exists.
   - breadth where enough stock history exists.
5. Stored industry `components_json`.
6. Updated the daily report so industry/theme strength is explainable.
7. Added industry-specific backtest expansion after review showed it was needed.
8. Added sector component backtest review for PDB-2.

Out of scope:

- Dashboard.
- News/catalyst provider.
- ETF flow provider.
- Options flow.
- Portfolio simulation.
- Transaction-cost model.
- ML scoring.

## 10. Acceptance For Industry Hardening

The industry hardening and follow-up validation steps are complete:

| Acceptance item | Status |
|---|---|
| Industry scores are no longer only a simple 20D return proxy. | Complete |
| Top industries have explainable component values. | Complete |
| Daily report industry sections show stronger reasons. | Complete |
| Tests prove industry scoring does not use future prices. | Complete |
| Backtest can still run after the change. | Complete |
| `docs/implementation_spec.md` limitation about industry scoring is updated. | Complete |
| PDB-1 industry-specific validation is documented. | Complete |
| PDB-2 sector score review is documented. | Complete |
| PDB-3 market regime V1 review is documented. | Complete |
| PDB-3.5 sector formula decision is documented. | Complete |

## 11. Current State

Merryl is in a real, usable engine-validation state:

```text
Daily run works.
Historical scoring works.
Stock ranking works well enough to preserve.
Backtesting works.
Market regime coverage is explicitly labeled with included and missing data sources.
Sector score needs more evidence and possibly refinement.
Sector ranking is map-only / not yet a proven forward-return predictor.
Sector formula no longer includes a neutral rank-change placeholder.
Industry/theme scoring is hardened and validated enough to preserve as a core market-map layer.
```

This preserves the original intent and avoids drifting into a polished dashboard before the market-map logic is strong enough.

## 12. Next Checkpoint

PDB-1 industry-specific validation is complete.

Result:

```text
Industry/theme strength is validated enough to remain part of the core market map.
```

The result is recorded in:

```text
docs/industry_specific_validation_spec.md
```

PDB-2 sector score review is complete.

Result:

```text
Sector ranking remains a market-map and attention layer.
It is map-only / not yet a proven forward-return predictor.
```

The result is recorded in:

```text
docs/sector_score_review_spec.md
```

PDB-3 market regime V1 review is complete.

Result:

```text
Market regime coverage explicitly includes SPY, QQQ, IWM, DIA, TLT, GLD, and USO ETF proxy context, and names the missing macro sources.
```

The result is recorded in:

```text
docs/market_regime_v1_spec.md
```

PDB-3.5 sector formula decision checkpoint is complete.

Result:

```text
The neutral rank-change contribution was removed from sector scoring.
rank_change remains stored and reported, but it is not a scoring component.
```

The result is recorded in:

```text
docs/sector_formula_decision_checkpoint_spec.md
```

Before full dashboard work, continue following the pre-dashboard stability backlog:

```text
docs/pre_dashboard_stability_backlog_spec.md
```

The pre-dashboard implementation checkpoints are now complete through:

```text
PDB-6: Data quality and reproducibility check
```

PDB-6 made required data coverage and workflow reproducibility visible through `doctor` and tests. The next step is Phase 4 planning for the first controlled dashboard/API slice.
