# Industry-Specific Validation

Version: 0.1  
Date: 2026-05-27  
Status: PDB-1 complete; result is supportive with scope limits

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_3_backtest_validation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`

## 1. Purpose

This document records the PDB-1 validation result.

Question:

```text
Do stocks inside stronger industries/themes show better forward behavior
than stocks inside weaker industries/themes, especially relative to sector ETF?
```

This protects the original top-down flow:

```text
Market regime -> sector rotation -> industry/theme strength -> stock leadership -> watchlist
```

The validation uses stored SQLite data only. It does not fetch new prices, change scoring formulas, or add a new provider.

## 2. Implementation Summary

The existing backtest workflow now adds `stock_by_industry` rows.

Meaning:

```text
stock_by_industry decile 10 = stocks whose same-day industry/theme score is strongest.
stock_by_industry decile 1 = stocks whose same-day industry/theme score is weakest.
```

Forward returns are still stock forward returns.

Primary relative return remains:

```text
stock forward return - sector ETF forward return
```

This answers whether industry/theme strength improves the stock watchlist context without turning Merryl into a trade-entry system.

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
stock observations: 51950
industry validation observations: 51950
```

## 4. Result Summary

Average stock forward return versus sector ETF:

| Horizon | Industry Decile 10 | Industry Decile 1 | Decile 10 Minus Decile 1 |
|---:|---:|---:|---:|
| 1D | 0.11% | -0.56% | 0.67% |
| 5D | 0.56% | -1.12% | 1.68% |
| 10D | 1.05% | -3.01% | 4.06% |
| 20D | 1.74% | -5.97% | 7.71% |
| 60D | 7.87% | -7.14% | 15.01% |

Hit rate versus sector ETF:

| Horizon | Industry Decile 10 | Industry Decile 1 |
|---:|---:|---:|
| 1D | 50.83% | 36.84% |
| 5D | 52.06% | 42.11% |
| 10D | 53.11% | 21.05% |
| 20D | 52.60% | 5.26% |
| 60D | 58.54% | 26.67% |

Median stock forward return versus sector ETF:

| Horizon | Industry Decile 10 | Industry Decile 1 |
|---:|---:|---:|
| 1D | 0.03% | -0.75% |
| 5D | 0.19% | -2.31% |
| 10D | 0.34% | -3.67% |
| 20D | 0.59% | -3.91% |
| 60D | 3.14% | -13.81% |

## 5. Interpretation

The industry/theme bridge is useful enough to preserve.

Stocks in stronger industry/theme deciles showed better forward behavior than stocks in weaker industry/theme deciles across every tested horizon in this run.

The strongest result appears over 20D and 60D, which fits Merryl's intended use as a watchlist and chart-review engine rather than a one-day entry system.

Important scope limit:

```text
This validation uses stored top-50 daily stock score rows.
It is not a full-universe industry validation yet.
```

The weakest industry decile has a much smaller sample because the daily watchlist naturally contains fewer names from weak industries. That is useful information, but it means we should not overfit scoring weights from this run alone.

## 6. Decision

Decision:

```text
Industry/theme scoring is validated enough to remain part of the core market map.
```

Do not change stock scoring weights yet.

Reason:

- The industry/theme layer clearly adds useful context.
- The current stock score is already behaving well.
- Weight changes should not be made from one validation window.
- PDB-2 sector score review labels sector ranking as map-only for now, so stock weights should not be changed based on sector score confidence.

Current product stance:

```text
Use industry/theme strength as an attention and confirmation layer.
Do not present it as an automatic trade signal.
```

## 7. Next Work

PDB-1 is complete.

PDB-2 sector score review is also complete.

PDB-3 market regime V1 review is also complete.

PDB-3.5 sector formula decision checkpoint is also complete.

PDB-3.6 spec completeness gate is also complete.

PDB-4 catalyst/news source decision is also complete.

PDB-5 backtest scope clarity is also complete.

PDB-6 data quality and reproducibility is also complete.

That next step is now implemented:

```text
Phase 4 first slice: read-only local API plus initial dashboard shell
```

The current GICS industry/theme coverage was audited against the main spec and accepted for the first build. Recent news catalysts are connected through Alpaca News; structured earnings calendar data remains not connected. Backtest scope is now explicit as score-behavior validation, not trade profitability. `doctor` now checks core stored data before dashboard work. Phase 4 dashboard/API planning is recorded in `docs/phase_4_dashboard_api_spec.md`.
