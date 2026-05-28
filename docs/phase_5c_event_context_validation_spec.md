# Phase 5C Event Context Validation Spec

Version: 0.2
Date: 2026-05-28
Status: Implemented; review results before any catalyst/event score change

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/phase_5c_source_coverage_review_spec.md`
- `docs/watchlist_convergence_review_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

Phase 5C connected real event context:

```text
Alpaca News
  + Alpha Vantage Earnings Calendar
  + SEC EDGAR submissions
```

The next required step is not another data source. The next step is to validate whether the event labels help the final watchlist review surface.

This checkpoint asks:

- Do watchlist names with event context behave differently from names with `pending_source`?
- Do recent-news labels, earnings labels, filing labels, and multi-event labels show different forward behavior?
- Does event context help classify or filter chart-review candidates without pretending to be a trade signal?

## 2. Alignment With Main Spec

The main system remains:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> classified and filtered watchlist for chart review elsewhere
```

Event context belongs near the end of the chain. It answers:

```text
Why might this already-ranked leader be active now?
```

It does not replace the price, volume, sector, industry, or market-regime scoring layers.

## 3. Hard Constraints

This checkpoint must not:

- Change sector, industry, stock, or regime score weights.
- Add a new public CLI command.
- Add a paid provider.
- Fetch new data during backtest.
- Backfill fake or inferred catalyst labels.
- Use future prices to decide historical event labels.
- Present event labels as bullish or bearish trade signals.
- Turn Merryl into a chart-review platform.

Execution surface:

```text
merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD
```

## 4. Data Inputs

Read from SQLite only:

```text
stock_scores
watchlists
sector_map
prices_daily
```

The validation uses the stored `stock_scores.catalyst_status` value for each watchlist symbol on each score date. That value must represent context known when that daily run stored the score.

## 5. Historical Label Preservation Requirement

Daily scoring recomputes a rolling historical score window. When it rewrites older `stock_scores` rows, it must preserve existing non-`pending_source` catalyst labels for dates before the current report date.

Reason:

```text
The latest daily run should not erase previously captured as-of event context.
```

Boundary:

```text
Do not copy today's newly fetched event labels into older historical dates.
Do not infer missing historical catalyst labels.
Preserve only source-backed labels already stored for that older score date and symbol.
```

Fresh databases may have too few historical event-labeled rows for strong conclusions. The report must show that limitation directly.

## 6. Validation Groups

Use inclusive groups:

| Group | Meaning |
|---|---|
| `all_watchlist` | Every scored watchlist row with enough forward price bars |
| `pending_source` | Stored catalyst label is `pending_source` |
| `event_context` | Stored catalyst label is not `pending_source` |
| `recent_news` | Label includes `recent_news:` |
| `earnings` | Label includes `earnings:` |
| `filing` | Label includes `filing:` |
| `event_risk` | Label includes `earnings:` or `filing:` |
| `multiple_event_types` | Label includes two or more event types |

Groups are intentionally inclusive. A stock with `recent_news:2 | earnings:YYYY-MM-DD | filing:8-K` belongs to multiple groups.

## 7. Forward Return Policy

Use the same horizons as Phase 3:

```text
1D, 5D, 10D, 20D, 60D
```

Use trading-bar offsets, not calendar-day assumptions.

For each valid watchlist row and horizon:

- Calculate forward stock return.
- Calculate forward return versus `SPY`.
- Calculate forward return versus the stock sector ETF.
- Skip only the affected horizon when enough future bars are missing.

Do not fail the whole run unless the database lacks the required stored scores, watchlist rows, sector maps, or prices.

## 8. Outputs

Write:

```text
reports/validations/YYYY-MM-DD_YYYY-MM-DD_event_context_validation.md
exports/validations/YYYY-MM-DD_YYYY-MM-DD_event_context_validation.csv
```

Store the event validation metrics inside `backtest_results.metrics_json` alongside the existing backtest metrics.

The report must explicitly say:

```text
This validates event context behavior for watchlist review. It does not validate trade profitability and does not change score weights.
```

## 9. Metrics

For each group and horizon:

- Count.
- Hit rate versus sector ETF.
- Average forward stock return.
- Median forward stock return.
- Average forward return versus `SPY`.
- Median forward return versus `SPY`.
- Average forward return versus sector ETF.
- Median forward return versus sector ETF.

Also report:

- Watchlist rows reviewed.
- Scored watchlist rows matched to `stock_scores`.
- Rows with event context.
- Rows with `pending_source`.
- Valid forward observations.
- Skipped horizons due to missing future bars.

## 10. Acceptance

This checkpoint is accepted when:

- The event validation engine reads SQLite only.
- The existing backtest workflow writes event validation Markdown and CSV outputs.
- `backtest_results.metrics_json` includes event validation results without breaking existing backtest metrics.
- Daily rolling score rewrites preserve older source-backed catalyst labels.
- Tests prove event grouping, future-bar handling, output writing, and catalyst-label preservation.
- No score formula, public CLI surface, paid source, or dashboard charting feature is added.

## 11. Implemented Boundary

Implemented on 2026-05-28:

- Added an event-context validation engine that reads stored SQLite data only.
- Reused the existing `merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD` workflow.
- Added no new public CLI command.
- Added no new provider or paid source.
- Kept sector, industry, stock, and regime score formulas unchanged.
- Grouped watchlist rows by stored same-day `catalyst_status` labels.
- Calculated forward stock return, return versus `SPY`, and return versus sector ETF using future trading bars.
- Wrote event-context validation outputs:

```text
reports/validations/YYYY-MM-DD_YYYY-MM-DD_event_context_validation.md
exports/validations/YYYY-MM-DD_YYYY-MM-DD_event_context_validation.csv
```

- Stored event validation metrics inside `backtest_results.metrics_json` under `event_context_validation`.
- Preserved older non-`pending_source` catalyst labels when the daily workflow rewrites its rolling historical score window.

## 12. Live Verification

Live verification completed on 2026-05-28:

```text
set -a; source .env; set +a; cargo run -- run daily --date latest
  -> date: 2026-05-28
  -> macro observations: 4848
  -> news events: 264
  -> earnings events: 49
  -> filing events: 40

set -a; source .env; set +a; cargo run -- run backtest --from 2025-07-01 --to 2026-05-28
  -> event context validation report: reports/validations/2025-07-01_2026-05-28_event_context_validation.md
  -> event context validation export: exports/validations/2025-07-01_2026-05-28_event_context_validation.csv
  -> event context observations: 26225
  -> backtest result id: 13
```

Live event validation coverage:

| Metric | Value |
|---|---:|
| Watchlist rows reviewed | 5725 |
| Scored watchlist rows matched | 5725 |
| Rows with event context | 25 |
| Rows with pending source | 5700 |
| Valid forward observations | 26225 |
| Event-context forward observations | 0 |
| Skipped horizons missing future bars | 2400 |

Interpretation:

```text
Event-context rows exist, but they do not yet have enough future bars for forward validation in this date range.
```

This is expected because structured Phase 5C event labels were captured on the latest score date. The checkpoint is implemented, but catalyst/event scoring changes are not approved. Let future daily runs accumulate event-labeled history before revisiting catalyst/event weights.

Verification passed:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
cargo run -- status
```
