# Watchlist Actionability And Extension Filter Spec

Version: 0.1
Date: 2026-05-29
Status: Implemented for the current no-new-source pass

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/watchlist_convergence_review_spec.md`
- `docs/application_state_remaining_work_spec.md`
- `docs/phase_5_readiness_gate_spec.md`
- `docs/phase_5c_event_context_validation_spec.md`
- `docs/implementation_spec.md`

External research references:

- StockCharts ChartSchool: Average True Range and Average True Range Percent: `https://chartschool.stockcharts.com/table-of-contents/technical-indicators-and-overlays/technical-indicators/average-true-range-atr-and-average-true-range-percent-atrp`
- StockCharts ChartSchool: Price Relative / Relative Strength: `https://chartschool.stockcharts.com/table-of-contents/technical-indicators-and-overlays/technical-indicators/price-relative-relative-strength`
- StockCharts ChartSchool: Technical indicators list for distance from highs, distance from moving average, and relative volume: `https://chartschool.stockcharts.com/table-of-contents/technical-indicators-and-overlays/technical-indicators`
- Investor's Business Daily: Additional buy points, tight areas, pullbacks to 10-week / 50-day moving averages, and breakout volume concepts: `https://www.investors.com/ibd-university/how-to-buy/additional-buy-points`
- Investor's Business Daily: 5 percent buy zone and extended-stock concept: `https://www.investors.com/how-to-invest/investors-corner/how-to-buy-stocks-why-the-buy-zone-is-the-sweet-spot`

## 1. Purpose

Implementation checkpoint, 2026-05-29:

- Actionability metrics are generated during stock scoring from existing daily OHLCV data.
- Metrics and labels are stored in `stock_scores.components_json`.
- Core stock `score` and `rank` remain unchanged.
- Daily Markdown reports include `Actionability Review Queue`.
- Dashboard watchlist and leadership data expose actionability fields.
- Dashboard `Actionability Review Queue` uses the stored top-50 stock scores, while the ranked watchlist remains the existing leadership list.
- Existing `merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD` writes actionability validation outputs.
- CSV watchlist shape remains unchanged.
- No new provider, paid source, public CLI command, charting workspace, or scoring-weight change was added.

The current Merryl watchlist is good at finding visible leadership, but many names can appear after they have already made an explosive move.

That means the system currently answers:

```text
Which stocks are leading now?
```

But the product needs a second question:

```text
Which leaders are still actionable enough to be worth charting now?
```

This spec designs an actionability and extension layer that sits after stock scoring and before final review.

It should help separate:

- leaders that are already extended
- leaders pulling back constructively
- early rotation candidates
- compression candidates before a possible move
- event-backed names that still need chart confirmation

This does not turn Merryl into a trading signal. It improves the final watchlist ordering and explanation so the user does not waste time charting names that are already too far gone.

## 2. Alignment With Original Intent

The original product chain remains:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> classified watchlist for chart review elsewhere
```

This new layer belongs at the final step:

```text
Stock leadership
  -> actionability / extension classification
    -> chart review elsewhere
```

It does not replace:

- market regime
- sector rotation
- industry/theme strength
- stock opportunity score
- catalyst/event context
- backtest validation

It makes the current watchlist more useful by distinguishing "strong" from "still worth reviewing now."

## 3. Problem Found

Observation from current dashboard/report review:

```text
Many high-ranked watchlist names have already made large explosive moves.
```

This is expected from the current score design because stock scoring rewards:

- strong 20D relative return
- strong sector-relative strength
- elevated relative volume
- trend above moving averages
- liquidity

Those are good leadership signals, but they are also late-confirmation signals.

Without an actionability layer, Merryl can surface:

```text
real leadership, but too late for a clean chart review
```

The goal is not to remove strong movers. Strong stocks can keep trending. The goal is to label the state honestly:

```text
extended leader
actionable leader
pullback leader
early rotation candidate
base compression candidate
event watch
```

## 4. Research Findings

The external research supports using the following concepts as objective daily-data proxies:

| Concept | Why it matters for Merryl | Source alignment |
|---|---|---|
| Distance from moving averages | Helps identify whether price is close to trend support or stretched far above it | StockCharts lists distance from moving average as a technical indicator |
| Distance from recent highs | Helps distinguish breakout proximity, pullback depth, and already-moved names | StockCharts lists distance from highs and distance to highs |
| ATR / ATRP | Normalizes price extension by volatility so high-volatility and low-volatility names are not judged only by raw percentage move | StockCharts ATR/ATRP reference |
| Relative strength versus benchmark or sector | Preserves the core Merryl idea that leadership is relative, not just raw return | StockCharts Price Relative reference |
| Relative volume | Confirms whether price movement has participation | StockCharts technical indicators list includes RVOL |
| Tight ranges and pullbacks to moving averages | Helps classify leaders that are consolidating or offering secondary review opportunities after leadership appears | IBD additional buy points reference |
| Extended beyond an entry zone | Confirms that a strong stock can become too late for fresh review after moving too far | IBD buy-zone reference |

Research conclusion:

```text
The next improvement should not be another source.
The next improvement should measure where each leader is in its move.
```

## 5. Product Decision

Implement a final watchlist actionability layer using only existing daily OHLCV data and stored scores.

Decision:

```text
Add actionability labels and an actionability review queue.
Do not change core score formulas in the first pass.
Do not add a new public CLI command.
Do not add a new provider.
Do not add charting inside Merryl.
```

The first implementation should classify and display. Scoring changes should wait until validation proves the labels are useful.

## 6. Inputs

Use existing data only:

- `prices_daily`
- `stock_scores`
- `sector_scores`
- `industry_scores`
- `watchlists`
- `events`
- stored macro context where already available

No new data provider is required.

No paid source is required.

No intraday data is required for the first pass.

## 7. New Metrics To Calculate

Add actionability metrics during historical stock scoring for each scored stock/date.

Store them inside `stock_scores.components_json` so no additive schema migration is required for the first pass.

Required metrics:

| Metric | Meaning | Example use |
|---|---|---|
| `ma_20d` | 20 trading-bar moving average | trend and pullback reference |
| `ma_50d` | 50 trading-bar moving average | intermediate trend reference |
| `distance_from_20d_ma_pct` | close versus 20D moving average | extension / pullback |
| `distance_from_50d_ma_pct` | close versus 50D moving average | extension / trend shelf |
| `atr_14d` | 14-bar average true range | volatility baseline |
| `atr_14d_pct` | ATR divided by close | cross-stock volatility comparison |
| `atr_extension_from_20d_ma` | close minus 20D MA, divided by ATR | volatility-normalized extension |
| `atr_extension_from_50d_ma` | close minus 50D MA, divided by ATR | volatility-normalized extension |
| `high_20d` | highest adjusted close over 20 bars | breakout proximity / pullback depth |
| `high_60d` | highest adjusted close over 60 bars | sustained trend context |
| `distance_from_20d_high_pct` | close versus 20D high | near high or pulled back |
| `distance_from_60d_high_pct` | close versus 60D high | trend location |
| `range_10d_pct` | 10-bar high/low range as percent of close | compression or wide disorder |
| `gap_pct` | current open versus prior close | fresh gap / already repriced |
| `true_range_pct` | current true range as percent of close | one-day volatility shock |
| `actionability_labels` | generated labels for that scored date | historical validation |
| `primary_actionability` | one primary bucket | report/dashboard grouping |

Use adjusted close for moving-average, return, and high-distance calculations to stay consistent with existing scoring.

Use raw open/high/low/close only for gap and true-range calculations because those metrics describe the daily candle structure.

## 8. First Label Set

### 8.1 `extended_leader`

Meaning:

```text
Leadership is real, but the stock has already moved too far from normal trend structure.
```

Candidate conditions:

- 5D return is unusually large.
- 1D return or gap is unusually large.
- Close is far above the 20D or 50D moving average.
- ATR extension from the 20D or 50D moving average is high.
- Close is near a 20D/60D high after a large short-term move.

Use:

```text
Still a leader, but not the cleanest fresh chart-review candidate.
```

### 8.2 `actionable_leader`

Meaning:

```text
Leadership is confirmed and the stock is not too extended.
```

Candidate conditions:

- Stock score remains strong.
- Sector or industry context is strong.
- Relative strength versus sector or SPY is positive.
- Close is above trend structure.
- Distance from 20D/50D moving average is not extreme.
- ATR extension is not extreme.

Use:

```text
High-priority chart review candidate.
```

### 8.3 `early_rotation_candidate`

Meaning:

```text
The sector/industry context is improving and the stock is starting to participate before a full explosive move.
```

Candidate conditions:

- Sector rank or industry rank is strong or improving.
- Stock relative return versus sector/SPY is improving.
- 5D return is positive but not already stretched.
- Relative volume is improving but not necessarily extreme.
- Close is near or reclaiming the 20D/50D moving average.

Use:

```text
Candidate to chart before it becomes an obvious leaderboard name.
```

### 8.4 `pullback_leader`

Meaning:

```text
The stock remains a relative leader but has pulled back from recent highs toward trend support.
```

Candidate conditions:

- Relative strength versus sector or SPY remains positive.
- Close is below recent 20D/60D high by a controlled amount.
- Close is near the 20D or 50D moving average.
- Pullback volume is not extreme selling pressure.
- The stock is not below trend structure.

Use:

```text
Candidate for manual chart review around support or recovery.
```

### 8.5 `base_compression_candidate`

Meaning:

```text
The stock is holding near highs or trend support with a tight recent range.
```

Candidate conditions:

- 10D range is tight relative to price or ATR.
- Close is not far below the 20D/60D high.
- Relative strength remains positive or stable.
- Relative volume is not showing disorderly selling.

Use:

```text
Potential pre-move setup that still needs chart confirmation.
```

### 8.6 `event_watch_unconfirmed`

Meaning:

```text
There is event context, but price action has not yet confirmed a clean actionable setup.
```

Candidate conditions:

- Catalyst/event label exists.
- Stock is not an actionable leader by price structure.
- Stock is not fully extended, or it is extended but event risk needs manual review.

Use:

```text
Keep on radar, but do not imply direction.
```

## 9. Primary Bucket Priority

A stock may have multiple labels. The report and dashboard should also expose one primary bucket using this priority:

```text
extended_leader
pullback_leader
base_compression_candidate
early_rotation_candidate
actionable_leader
event_watch_unconfirmed
unclassified_leader
```

Reason:

- `extended_leader` should be assigned first as a caution, even if the stock also has leadership labels.
- `pullback_leader`, `base_compression_candidate`, and `early_rotation_candidate` are the useful "find it before the next move" groups.
- `actionable_leader` is confirmed but not stretched.
- `event_watch_unconfirmed` is context only.

The UI can group by primary bucket while preserving the original stock rank.

## 10. Threshold Governance

Initial thresholds must live in `src/config/mod.rs`, not scattered through files.

Suggested first thresholds to validate:

| Threshold | Initial value | Why |
|---|---:|---|
| Extended 5D return | `0.08` | Flags fast short-term repricing |
| Extended 1D return | `0.05` | Flags one-day explosive move |
| Extended gap | `0.04` | Flags news/gap repricing |
| Extended distance from 20D MA | `0.12` | Flags distance from short trend |
| Extended distance from 50D MA | `0.20` | Flags distance from intermediate trend |
| Extended ATR multiple from 20D MA | `2.5` | Volatility-normalized stretch |
| Pullback from 20D high min | `-0.12` | Controlled pullback, not collapse |
| Pullback from 20D high max | `-0.03` | Not just sitting at highs |
| Near 20D MA max distance | `0.04` | Pullback near short trend |
| Near 50D MA max distance | `0.06` | Pullback near intermediate trend |
| Compression 10D range max | `0.08` | Tight enough to suggest digestion |
| Relative volume confirmation | `1.2` | Matches current classification threshold |

These are not permanent truths. They are first validation thresholds.

Do not tune them from a few examples. Validate by historical forward behavior first.

## 11. Implementation Shape

### 11.1 Scoring Indicators

Add indicator helpers in:

```text
src/scoring/indicators.rs
```

Required helpers:

- `highest_close(history, idx, lookback)`
- `lowest_close(history, idx, lookback)`
- `true_range(history, idx)`
- `average_true_range(history, idx, lookback)`
- `distance_pct(value, reference)`
- `range_pct(high, low, close)`
- `gap_pct(history, idx)`

Keep these generic and reusable. Do not put actionability labels inside indicator helpers.

### 11.2 Actionability Classifier

Add a dedicated module:

```text
src/actionability.rs
```

Responsibilities:

- Convert stock score plus actionability metrics into labels.
- Choose `primary_actionability`.
- Keep threshold logic in one place.
- Return deterministic output for historical validation.

Do not mix this into `src/classification.rs`. Existing classification answers why a name is on the list. Actionability answers whether the name is still useful to chart now.

### 11.3 Stock Scoring Storage

Update:

```text
src/scoring/stocks.rs
```

Responsibilities:

- Calculate actionability metrics for each scored stock/date.
- Add metrics and labels into `components_json`.
- Keep `score` unchanged in the first pass.
- Keep rank order unchanged in the first pass.

No new SQLite column is required for the first implementation.

### 11.4 Report Output

Update:

```text
src/output/markdown.rs
src/config/mod.rs
```

Add a new section before `Top Stocks Worth Charting`:

```text
## Actionability Review Queue
```

The section should group top-50 scored stocks by primary bucket:

| Bucket | Report behavior |
|---|---|
| `early_rotation_candidate` | Show first |
| `base_compression_candidate` | Show next |
| `pullback_leader` | Show next |
| `actionable_leader` | Show next |
| `extended_leader` | Show as caution, not first |
| `event_watch_unconfirmed` | Show only if not already in another useful group |

The existing ranked watchlist should remain, but it should include:

- `Primary Actionability`
- compact actionability labels

The report must still say:

```text
This is a watchlist for chart review, not a trade signal.
```

### 11.5 Dashboard API

Update:

```text
src/dashboard/models.rs
src/dashboard/repository.rs
dashboard/src/types.ts
dashboard/src/tableColumns.ts
dashboard/src/components/DashboardPage.tsx
```

Expose:

- `primary_actionability`
- `actionability_labels`
- `distance_from_20d_ma_pct`
- `distance_from_50d_ma_pct`
- `atr_extension_from_20d_ma`
- `distance_from_20d_high_pct`

Dashboard behavior:

- Watchlist view should support grouping or filtering by actionability bucket.
- Leadership rank should remain visible.
- Extended names should be visible but visually separated as "already moved / caution."
- Do not add chart rendering.
- Do not fetch new market data from the dashboard.

### 11.6 CSV Export

Keep existing CSV shape unchanged in the first implementation unless a separate export decision is made.

Reason:

```text
The docs already treat CSV as a stable portable export.
```

The Markdown report and dashboard are the first places to expose actionability.

## 12. Validation Plan

Add actionability validation to the existing backtest workflow:

```text
merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD
```

Do not add a new public command.

Read from SQLite only:

- `stock_scores`
- `watchlists`
- `prices_daily`
- `sector_map`

Use stored `components_json.actionability_labels` and `components_json.primary_actionability` from each score date.

Write:

```text
reports/validations/YYYY-MM-DD_YYYY-MM-DD_actionability_validation.md
exports/validations/YYYY-MM-DD_YYYY-MM-DD_actionability_validation.csv
```

Also store metrics under `backtest_results.metrics_json.actionability_validation`.

Validation groups:

- `all_watchlist`
- `extended_leader`
- `actionable_leader`
- `early_rotation_candidate`
- `pullback_leader`
- `base_compression_candidate`
- `event_watch_unconfirmed`
- `unclassified_leader`

Metrics by group and horizon:

- count
- hit rate versus sector ETF
- average forward stock return
- median forward stock return
- average forward return versus SPY
- median forward return versus SPY
- average forward return versus sector ETF
- median forward return versus sector ETF

Additional useful metrics:

- average 5D forward drawdown using daily lows, if available
- percentage of rows that became extended after 5D or 10D
- percentage of early candidates that later entered the top ranked watchlist

If the additional metrics require too much code in the first implementation, defer them and keep the main forward-return validation first.

## 13. Tests Needed

Add tests under `/tests`.

Required tests:

- Indicator tests for ATR, true range, moving-average distance, high-distance, gap percent, and 10D range.
- Actionability classifier tests for each label.
- Stock scoring test proving actionability metrics are written into `components_json`.
- Markdown report test proving `Actionability Review Queue` exists.
- Dashboard API test proving primary actionability and labels are exposed.
- Backtest validation test proving actionability groups use only stored score-date labels.
- Future-bar test proving validation does not use future prices to assign actionability labels.
- Regression test proving core stock `score` and `rank` are unchanged by actionability labels.

Required commands after implementation:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
set -a; source .env; set +a; cargo run -- run daily --date latest
cargo run -- run backtest --from 2025-07-01 --to <latest scored date>
cargo run -- status
```

## 14. Acceptance Criteria

Implementation is accepted when:

- Complete: Actionability labels are generated for scored stocks using existing daily data.
- Complete: Actionability metrics and labels are stored in `stock_scores.components_json`.
- Complete: Core stock scores and ranks are unchanged.
- Complete: Daily report includes an `Actionability Review Queue`.
- Complete: Existing ranked watchlist remains visible.
- Complete: Dashboard exposes and displays actionability consistently.
- Complete: Backtest validation can compare forward behavior by actionability bucket.
- Complete: No new provider is added.
- Complete: No public CLI command is added.
- Complete: No paid source is added.
- Complete: CSV exports remain unchanged.
- Complete: The report and dashboard keep Merryl positioned as a market-map and watchlist tool, not a trading signal.

## 15. What This Does Not Solve

This will not perfectly find stocks before every explosive move.

It also does not replace chart review because the chart still decides:

- support and resistance
- exact breakout level
- invalidation
- entry quality
- risk
- earnings/event timing

The goal is narrower and practical:

```text
Reduce late, already-extended names in the first review queue
and surface leaders that are earlier, tighter, or pulling back constructively.
```

## 16. Recommended Implementation Order

1. Add indicator helpers and unit tests.
2. Add `src/actionability.rs` and deterministic label tests.
3. Store metrics and labels in `stock_scores.components_json`.
4. Add report section and report tests.
5. Add dashboard API exposure and dashboard tests.
6. Add actionability validation into the existing backtest workflow.
7. Run real daily and backtest verification.
8. Review whether the new buckets actually improve the final watchlist.

## 17. Decision Before Code

Recommended decision:

```text
Proceed with actionability classification as the next local application improvement.
```

Reason:

- It directly addresses the observed problem.
- It uses the data already connected.
- It improves the final watchlist without adding a new source.
- It respects the Phase 5 readiness gate.
- It does not change score formulas before validation.

Do not proceed if the intended next step is to make Merryl a charting platform. This spec only decides what Merryl should put in front of the user for chart review elsewhere.

## 18. Implementation Verification

Commands run after implementation:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
npm run build
set -a; source .env; set +a; cargo run -- run daily --date latest
cargo run -- run backtest --from 2025-07-01 --to 2026-05-29
cargo run -- status
```

Latest live run:

```text
date: 2026-05-29
historical score dates: 228
actionability validation report: reports/validations/2025-07-01_2026-05-29_actionability_validation.md
actionability validation export: exports/validations/2025-07-01_2026-05-29_actionability_validation.csv
actionability observations: 26350
```

Current interpretation:

```text
The actionability layer is now implemented as a review and validation layer.
It is not approved as a scoring-weight change.
The next decision should come from observing the validation output over more live runs.
```

Dashboard correction, 2026-05-29:

```text
The top 25 ranked watchlist can all be extended in a strong momentum tape.
The dashboard actionability queue now reads the stored top-50 stock score rows, orders them by actionability bucket, and leaves the ranked watchlist unchanged.
This exposes non-extended candidates without expanding Merryl into a broad scanner.
```
