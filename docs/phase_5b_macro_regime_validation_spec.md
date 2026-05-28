# Phase 5B Macro Regime Validation Spec

Version: 0.2
Date: 2026-05-28
Status: Implemented; review results before any Market Regime V1 scoring change

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5c_source_coverage_review_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

Merryl currently stores FRED macro observations, but Market Regime V1 still scores regime from ETF price proxies:

```text
SPY, QQQ, IWM, DIA, TLT, GLD, USO
```

This checkpoint validates whether stored macro context helps explain or improve regime interpretation before any scoring formula changes.

The goal is not to replace the regime score immediately. The goal is to test the current regime label against macro context over historical scored dates.

## 2. Alignment With Main Spec

The main market model starts at the top:

```text
Macro regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
```

This checkpoint targets the top of that chain.

It preserves the original intent by asking:

- Is risk appetite improving or weakening?
- Are rates, inflation, credit, dollar, or liquidity conditions supporting the move?
- Does macro context explain why sectors are rotating?
- Does macro context improve review quality without pretending to be a trade signal?

## 3. Hard Constraints

This checkpoint must not:

- Change Market Regime V1 scoring weights.
- Change sector, industry, or stock scoring weights.
- Add a new public CLI command.
- Fetch a new paid data source.
- Add charting, trade execution, alerts, or portfolio simulation.
- Use future macro observations when evaluating a historical score date.
- Treat backtest output as trade profitability.

If implementation is needed, prefer extending the existing validation/backtest path instead of creating a new CLI surface.

## 4. Current Data Available

Macro observations are already stored in SQLite through the Phase 5A/B implementation.

Current configured FRED series:

| Need | Series |
|---|---|
| Equity volatility/risk stress | `VIXCLS` |
| 10Y yield | `DGS10` |
| 2Y yield | `DGS2` |
| Yield curve | `T10Y2Y` |
| Fed funds | `DFF` |
| Inflation | `CPIAUCSL` |
| Employment | `UNRATE` |
| Payrolls | `PAYEMS` |
| Credit spread | `BAMLC0A0CM` |
| Dollar pressure | `DTWEXBGS` |
| Liquidity | `WALCL` |

Existing scored tables:

```text
market_regime_scores
sector_scores
industry_scores
stock_scores
prices_daily
macro_series
```

## 5. Validation Questions

The first validation should answer:

- Does every historical score date have enough macro context available on or before that date?
- Which macro series are daily, weekly, or monthly, and how often are they stale relative to a score date?
- How often does ETF-proxy regime say risk-on while macro context is stressed?
- How often does ETF-proxy regime say defensive/mixed while macro context is improving?
- Do sector leaders change in sensible ways during rate-pressure, credit-stress, dollar-pressure, or inflation-pressure regimes?
- Does macro context help explain weak sector-forward behavior already found in Phase 3 validation?
- Does macro context add review value without making the report feel like a trading signal?

## 6. Date Alignment Rule

For every score date:

```text
Use only macro observations with observation_date <= score_date.
```

Do not use future macro observations to explain a past score.

Frequency handling:

- Daily series use the latest available observation on or before the score date.
- Weekly series use the latest available observation on or before the score date.
- Monthly series use the latest available observation on or before the score date.
- If a series has no prior observation, mark it missing for that date.

Revision limitation:

```text
FRED rows currently represent the latest available vintage, not true point-in-time historical vintage.
```

This limitation must be stated in reports before any macro-aware scoring change.

## 7. Proposed Macro Context Flags

The validation can derive context flags without changing score weights.

Initial flags:

| Flag | Candidate rule |
|---|---|
| Volatility stress | `VIXCLS` elevated versus its own historical window |
| Rate pressure | `DGS10` rising over recent observations |
| Yield-curve inversion | `T10Y2Y` below zero |
| Credit stress | `BAMLC0A0CM` widening over recent observations |
| Dollar pressure | `DTWEXBGS` rising over recent observations |
| Liquidity tightening | `WALCL` falling over recent observations |
| Inflation pressure | CPI year-over-year change at or above 3% |
| Labor cooling | `UNRATE` rising or `PAYEMS` weakening |

The exact thresholds must be simple, transparent, and stored in config if implemented. Do not tune thresholds to fit one backtest.

## 8. Proposed Validation Outputs

The first implementation should produce a validation report, not a new dashboard screen.

Preferred output:

```text
reports/validations/YYYY-MM-DD_YYYY-MM-DD_macro_regime_validation.md
```

Optional CSV if needed:

```text
exports/validations/YYYY-MM-DD_YYYY-MM-DD_macro_regime_validation.csv
```

The report should include:

- Score-date coverage.
- Macro series freshness by series.
- ETF-proxy regime labels by period.
- Macro context flags by period.
- Regime/macro disagreement examples.
- Sector leadership behavior under macro flags.
- Stock-score behavior under macro flags only if it remains readable.
- Explicit limitations and non-trade-signal language.

## 9. Implementation Approach

Recommended implementation path:

1. Read historical score dates and stored macro observations from SQLite.
2. Build an as-of macro snapshot for every score date.
3. Derive transparent macro context flags for each score date.
4. Compare flags with stored ETF-proxy regime labels and regime scores.
5. Summarize sector and stock validation behavior by macro flag if enough observations exist.
6. Write a Markdown validation report.
7. Update docs with the actual result before any scoring decision.

No new provider is needed.

No public CLI command is required. If this needs an execution surface, reuse the existing backtest/validation flow rather than exposing internal macro commands.

## 10. Acceptance Before Scoring Changes

Macro context can be considered for scoring only after:

- Macro coverage is available across the historical scored window.
- Date alignment avoids future macro data.
- Frequency and staleness behavior are visible.
- The latest-vintage limitation is documented.
- The validation report shows macro context adds useful interpretation.
- Any proposed score change has a separate formula decision document.
- A fresh backtest compares ETF-proxy regime against macro-aware regime behavior.

## 11. Out Of Scope

Out of scope for this checkpoint:

- Real-time macro release calendar.
- Point-in-time vintage reconstruction.
- Machine-learning regime classification.
- Paid macro providers.
- Options, fund-flow, dark-pool, or intraday confirmation.
- Dashboard charting work.
- Any direct buy/sell recommendation.

## 12. Implemented Boundary

Implemented on 2026-05-28:

- Added a macro/regime validation engine that reads stored SQLite data only.
- Added as-of macro snapshots for every historical score date in the selected range.
- Added date alignment so macro observations after a score date are not used for that score date.
- Added macro series freshness reporting by required FRED series.
- Added simple transparent macro stress flags:
  - volatility stress
  - rate pressure
  - yield-curve inversion
  - credit stress
  - dollar pressure
  - liquidity tightening
  - inflation pressure
  - labor cooling
- Added ETF-proxy regime versus macro-flag disagreement examples.
- Added sector leadership summaries under active macro flags.
- Added Markdown and CSV validation outputs:

```text
reports/validations/YYYY-MM-DD_YYYY-MM-DD_macro_regime_validation.md
exports/validations/YYYY-MM-DD_YYYY-MM-DD_macro_regime_validation.csv
```

Execution surface:

```text
merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD
```

This reuses the existing validation/backtest workflow and does not add a new public CLI command.

What remained unchanged:

- No provider change.
- No paid data source.
- No new dashboard screen.
- No sector, industry, stock, or regime scoring-weight change.
- No point-in-time vintage reconstruction.
- No trade-entry or portfolio model.

## 13. Live Verification

Live validation run completed on 2026-05-28:

```text
cargo run -- run backtest --from 2025-07-01 --to 2026-05-28
  -> report: reports/backtests/2025-07-01_2026-05-28_backtest_report.md
  -> summary export: exports/backtests/2025-07-01_2026-05-28_backtest_summary.csv
  -> macro regime validation report: reports/validations/2025-07-01_2026-05-28_macro_regime_validation.md
  -> macro regime validation export: exports/validations/2025-07-01_2026-05-28_macro_regime_validation.csv
  -> macro regime snapshots: 229
```

The generated macro validation report showed:

| Metric | Value |
|---|---:|
| Score dates | 229 |
| Complete macro snapshots | 229 |
| Missing macro snapshots | 0 |
| Snapshots with at least one stale macro series | 19 |
| Risk-on dates with active macro stress flags | 103 |
| Defensive/mixed dates with no active macro stress flags | 3 |

Current interpretation:

```text
FRED macro context is now validated as available and useful for review,
but it is still not approved as a Market Regime V1 score input.
```

The next decision is whether the validation result justifies a separate Market Regime formula decision document and a fresh backtest comparison.
