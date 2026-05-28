# Phase 5C Source Coverage Review Spec

Version: 0.1
Date: 2026-05-28
Status: Complete checkpoint; use before the next Phase 5 target

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5b_macro_regime_validation_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

This checkpoint reviews whether the completed Phase 5C catalyst/event sources are good enough to keep as source-backed context before Merryl moves to another data-source expansion target.

It does not change Merryl's job:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> Watchlist for chart review elsewhere
```

The review answers:

- Are real catalyst/event sources connected?
- Are missing values visible instead of silently treated as neutral?
- Does the context preserve the "why might this be moving?" question?
- Should catalyst/event data affect scores now?
- What is the next Phase 5 target that best preserves the market-rotation flow?

## 2. Live Coverage Snapshot

Live verification source:

```text
merryl run daily --date latest
```

Latest verified report:

```text
reports/2026-05-28_market_report.md
```

Latest scored date:

```text
2026-05-28
```

Stored historical coverage:

| Stored area | Coverage |
|---|---:|
| Stock score dates | 229 |
| Sector score dates | 229 |
| Market regime score dates | 229 |
| Macro observations | 4847 |

Latest scored-stock catalyst coverage:

| Slice | Rows | Pending source | Recent news | Earnings date | Filing event |
|---|---:|---:|---:|---:|---:|
| Top 25 scored stocks | 25 | 0 | 21 | 24 | 18 |
| Top 50 scored stocks | 50 | 0 | 39 | 49 | 29 |

Stored event coverage:

| Event type | Rows | Distinct symbols | Earliest event date | Latest event date |
|---|---:|---:|---|---|
| Earnings | 49 | 49 | 2026-05-28 | 2026-08-24 |
| Filing | 40 | 29 | 2026-05-14 | 2026-05-28 |
| News | 273 | 39 | 2026-05-27 | 2026-05-28 |

Event quality status:

| Quality status | Rows |
|---|---:|
| ok | 362 |

Example current catalyst labels:

```text
recent_news:18 | earnings:2026-05-28 | filing:8-K
recent_news:8 | earnings:2026-08-04 | filing:8-K
recent_news:1 | earnings:2026-07-29
earnings:2026-06-01 | filing:8-K
```

## 3. Interpretation

Phase 5C is working as a context layer for the latest ranked stock surface.

The important result is not that every stock has every possible event type. The important result is that the report now distinguishes:

```text
recent news
upcoming earnings date
recent SEC filing
combined event context
pending source where no context exists
```

This preserves the original market question:

```text
Why might this leader be active now?
```

The current coverage is strong enough to keep Phase 5C as part of the daily report and dashboard context.

## 4. Limitations

This checkpoint does not prove that catalyst/event flags improve forward returns.

Current limitations:

- This is one live coverage snapshot, not a long historical catalyst validation.
- Alpha Vantage earnings calendar is mainly upcoming event-risk context, not a complete historical surprise/outcome model.
- SEC filing events are metadata flags. Merryl does not parse full filing text yet.
- Alpaca news counts can include company, sector, or market-wide articles. A news flag is context, not causality.
- Catalyst labels are stored on current scored stocks as context and are not score inputs.
- Event coverage can vary by provider availability, rate limits, ticker coverage, and source freshness.

## 5. Decision

Keep Phase 5C connected.

Do not change score weights from this checkpoint.

Accepted decisions:

- Keep Alpaca News as recent-news context.
- Keep Alpha Vantage Earnings Calendar as free structured earnings-date context.
- Keep SEC EDGAR submissions as official filing-event context.
- Keep `events` as the canonical catalyst/event storage path.
- Keep catalyst/event labels compact in the report.
- Keep detailed event data in SQLite, not in CSV exports.
- Keep catalyst/event data out of sector, industry, stock, and regime score formulas until separate validation exists.

Rejected for now:

- Do not add paid Finnhub, Polygon/Massive, ETF Global, Cboe DataShop, fund-flow, or options sources.
- Do not expand the universe before mapping and source-quality gates are stronger.
- Do not add a new public CLI command for catalyst ingestion.
- Do not present event flags as bullish or bearish trade signals.

## 6. Next Phase 5 Target

The next validation-backed target should be macro/regime validation, not another paid or advanced source.

Reason:

- It is at the top of the market-map chain.
- The main spec starts with market regime before sector, industry, and stock selection.
- FRED macro data is already ingested and stored.
- The current largest source limitation remains Market Regime V1 using ETF price proxies only.
- `docs/phase_5_data_source_expansion_spec.md` already requires a macro/regime validation checkpoint before any macro scoring change.

Next target document:

```text
docs/phase_5b_macro_regime_validation_spec.md
```

The next target should compare:

```text
ETF-proxy regime behavior
  vs
FRED-aware macro context over historical scored dates
```

without changing scoring formulas first.

## 7. Non-Goals For The Next Step

The next step should not:

- Add ETF fund flows.
- Add options flow.
- Add dark-pool inference.
- Add intraday execution logic.
- Add portfolio simulation.
- Expand to Russell 1000 or Russell 3000.
- Turn the dashboard into a chart-review platform.
- Change score formulas before a validation checkpoint exists.

## 8. Acceptance

This checkpoint is accepted because:

- Phase 5C uses only free or already connected sources.
- Live daily verification stored news, earnings, and filing events.
- Latest scored top-25 and top-50 stocks had zero `pending_source` catalyst labels.
- Event rows include `quality_status`.
- The report labels event context as context, not a trade signal.
- The next Phase 5 target is explicit and aligned with the source specs.
