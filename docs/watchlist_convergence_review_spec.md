# Watchlist Convergence Review Spec

Version: 0.3
Date: 2026-05-28
Status: Implemented; Phase 5C event-context validation is implemented

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/phase_5c_source_coverage_review_spec.md`
- `docs/market_regime_formula_decision_checkpoint_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

This checkpoint answers the concern that Merryl must not keep adding sources without improving the final output.

The final output remains:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> classified and filtered watchlist for chart review elsewhere
```

The question for this checkpoint is:

```text
Are the connected sources converging toward better classification, filtering,
and explanation of the final watchlist?
```

Decision:

```text
Yes, but the convergence should now be made explicit in the watchlist output.
Do not add another data source next.
Do not change score formulas next.
Implement a final watchlist classification layer using existing stored data first.
```

## 2. Inputs Reviewed

Source specs reviewed:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/implementation_spec.md`

Live data reviewed:

```text
data/market.db
reports/2026-05-28_market_report.md
reports/validations/2025-07-01_2026-05-28_macro_regime_validation.md
```

Latest scored date:

```text
2026-05-28
```

## 3. Current Convergence Evidence

Stored historical surface:

| Area | Value |
|---|---:|
| Market regime score dates | 229 |
| Stock score dates | 229 |
| Macro observations | 4,848 |
| Latest scored stocks | 50 |
| Latest watchlist rows | 25 |

Latest market map:

| Layer | Current leading output |
|---|---|
| Market regime | Risk-on, score 73.8 |
| Macro overlay | liquidity_tightening, inflation_pressure |
| Top sector | Technology, score 88.7 |
| Top industry with broad stock presence | Technology Hardware, Storage & Peripherals |
| Top watchlist stock | DELL |

Latest top-25 catalyst/event coverage:

| Event context | Count |
|---|---:|
| Source-backed catalyst label | 25 |
| Recent news | 21 |
| Earnings date | 24 |
| Filing event | 18 |
| Pending source | 0 |

Latest top-50 catalyst/event coverage:

| Event context | Count |
|---|---:|
| Recent news | 39 |
| Earnings date | 49 |
| Filing event | 29 |
| Pending source | 0 |

Stored event coverage:

| Event type | Rows | Distinct symbols | Earliest date | Latest date |
|---|---:|---:|---|---|
| Earnings | 49 | 49 | 2026-05-28 | 2026-08-24 |
| Filing | 40 | 29 | 2026-05-14 | 2026-05-28 |
| News | 269 | 39 | 2026-05-27 | 2026-05-28 |

## 4. Source-To-Watchlist Mapping

Every connected source must have a job in the final output.

| Source / layer | Current role | Converges to final watchlist how? | Score input now? |
|---|---|---|---|
| Alpaca daily OHLCV | Price, volume, returns, liquidity | Primary stock, sector, and industry ranking inputs | Yes |
| Sector ETF map | Sector proxy and sector ranking | Narrows the market to active sectors | Yes |
| GICS sector/industry map | Stock grouping | Narrows leaders into industry/theme context | Yes |
| FRED macro series | Macro stress/context overlay | Explains whether the top-down regime has macro disagreement | No |
| Alpaca News | Recent headline context | Explains why a watchlist name may be active now | No |
| Alpha Vantage Earnings Calendar | Upcoming earnings date | Flags event risk and review timing for watchlist names | No |
| SEC EDGAR submissions | Recent filing event | Adds official event context for watchlist names | No |
| Backtest results | Score behavior validation | Decides whether scores deserve more trust or formula changes | No |

This means the source stack is not arbitrary. It currently supports:

```text
rank -> filter -> annotate -> validate
```

But the final report should make that convergence easier to see.

## 5. Current Gap

The current report already has the right data, but the final watchlist is still mostly a ranked table.

The report shows:

- rank
- score
- sector
- industry
- relative return
- relative volume
- trend state
- catalyst labels

What it does not yet show clearly enough:

- why each name survived the final filter as a classification label
- whether a name is a sector leader, industry leader, volume leader, catalyst-backed leader, or broad market-map continuation name
- whether macro context is supportive, conflicting, or only cautionary at the report level
- which rows deserve chart review first when the top 25 is still too many

This is a product-output issue, not a provider issue.

## 6. Decision

Do not move next to:

- ETF fund flows
- options flow
- intraday data
- universe expansion
- macro score penalties or bonuses
- catalyst score bonuses
- dashboard charting

Next implementation should be:

```text
Final Watchlist Classification Layer
```

It should use only existing stored data:

- market regime score
- macro context overlay
- sector scores
- industry scores
- stock scores
- catalyst/event labels
- historical rank-change and new-leader state

It should not fetch a new source.
It should not change rank formulas.
It should not add a new public CLI command.

## 7. Proposed Classification Labels

The first classification layer should produce compact labels that explain the final filter.

Initial labels:

| Label | Meaning |
|---|---|
| `sector_leader` | Stock is in a high-ranked sector |
| `industry_leader` | Stock is in a high-ranked industry/theme |
| `relative_strength_leader` | Stock has strong relative return versus sector/SPY |
| `volume_confirmed` | Stock has elevated relative volume |
| `new_leader` | Stock newly entered the latest watchlist versus prior scored date |
| `event_context` | Stock has news, earnings, filing, or combined event context |
| `event_risk` | Stock has near-term earnings or recent filing context requiring manual review |
| `macro_conflict_context` | Report-level macro overlay conflicts with ETF-proxy risk-on/risk-off tone |

The labels should explain the watchlist; they should not become trading signals.

## 8. Acceptance Criteria For The Next Implementation

The classification layer is accepted when:

- The daily report adds a compact final classification field for each top watchlist row.
- Dashboard API exposes the same classification labels without recalculating them in the frontend.
- Labels are derived only from stored scores/events/macro context.
- The current rank order remains unchanged.
- Existing CSV shape is preserved unless a separate export decision is made.
- No new data source, CLI command, or paid dependency is added.
- Tests prove labels are deterministic for sector, industry, relative strength, volume, event, and new-leader cases.
- The report still says Merryl is a watchlist and market-map tool, not a trading signal.

Implementation result:

- Daily report watchlist rows now include a compact `Classification` column.
- Dashboard API watchlist rows now expose `classifications`.
- Labels are derived from existing stored scores, catalyst labels, previous watchlist state, and macro context.
- Rank order remains unchanged.
- CSV shape remains unchanged.
- No new source, CLI command, schema migration, paid dependency, or score formula change was added.
- Tests cover deterministic classification labels, Markdown report output, and dashboard API exposure.

## 9. Follow-Up After Classification

Now that the classification layer exists, the follow-up validation checkpoint is:

```text
docs/phase_5c_event_context_validation_spec.md
```

This validates catalyst/event labels as a watchlist review layer before any catalyst/event scoring-weight change.

Live result:

```text
Event-context rows exist, but they do not yet have enough future bars for forward validation.
```

Do not move next to:

- ETF fund-flow confirmation
- options flow
- intraday data
- universe expansion
- catalyst/event score bonuses
