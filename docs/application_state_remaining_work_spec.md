# Application State And Remaining Work Spec

Version: 0.1
Date: 2026-05-29
Status: Current application-state audit after Phase 5 readiness gate

Related documents reviewed:

- `docs/backtest_scope_clarity_spec.md`
- `docs/catalyst_earnings_source_spec.md`
- `docs/data_quality_reproducibility_spec.md`
- `docs/implementation_spec.md`
- `docs/industry_specific_validation_spec.md`
- `docs/market_regime_formula_decision_checkpoint_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_3_backtest_validation_spec.md`
- `docs/phase_4_dashboard_api_spec.md`
- `docs/phase_4_dashboard_stabilization_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5_readiness_gate_spec.md`
- `docs/phase_5b_macro_regime_validation_spec.md`
- `docs/phase_5c_event_context_validation_spec.md`
- `docs/phase_5c_source_coverage_review_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/sector_formula_decision_checkpoint_spec.md`
- `docs/sector_score_review_spec.md`
- `docs/spec_completeness_gate_spec.md`
- `docs/watchlist_convergence_review_spec.md`

## 1. Purpose

This document states where Merryl stands as an application, what is working now, and what remains before the next phase should be built.

It preserves the original product chain:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> classified watchlist for chart review elsewhere
```

The application is not meant to become the chart-review surface. Merryl identifies where attention should go and why, then the user reviews charts in a separate charting tool.

## 2. Current Application Boundary

Merryl is currently a local market-rotation intelligence and watchlist application.

It should do:

- Pull real source data.
- Store source-backed observations with provenance.
- Score the historical market window.
- Classify the final watchlist.
- Explain what is driving attention.
- Validate whether score behavior has forward usefulness.
- Display the stored market map in a read-only dashboard.

It should not do at this stage:

- Execute trades.
- Become a charting platform.
- Simulate a full portfolio.
- Add alerts or position sizing.
- Tune scores from unvalidated new sources.
- Add paid sources without a source decision document.

## 3. What Is Working Now

| Application area | Current state | Notes |
|---|---|---|
| Rust core application | Working | Public surface remains small: daily run, backtest, doctor, status, dashboard, database migration |
| Local SQLite store | Working | SQLite is the canonical local system of record |
| Real daily market data | Working | Alpaca daily OHLCV is the market-data provider; dummy candles are not allowed |
| S&P 500 anchor universe | Working | Current universe remains S&P 500 before broader universe gates are written |
| Sector ETF map | Working | Sector ETFs support sector rotation scoring and reporting |
| Historical daily scoring backfill | Working | Daily run scores every valid historical date in the fetched window |
| Market Regime V1 | Working | Uses ETF price proxies for broad equity, rates, gold, and oil context |
| FRED macro ingestion | Working | Macro observations are fetched, stored, checked by doctor, and displayed as context |
| Macro context overlay | Working | Visible in report and dashboard API, but not a scoring input |
| Macro/regime validation | Working | Backtest workflow writes macro/regime validation outputs from stored SQLite data |
| Sector scores | Working | Useful as market map and attention layer; not yet accepted as standalone forward-return signal |
| Industry/theme scores | Working | Hardened with transparent components and supportive validation |
| Stock scores | Working | Current strongest validated layer for watchlist ranking |
| Watchlist generation | Working | Produces top stock candidates from the scored market window |
| Watchlist classification | Working | Connected sources now converge into explicit classification labels for the final list |
| Alpaca News events | Working | Recent news context is attached to watchlist names |
| Alpha Vantage earnings events | Working | Free-key earnings calendar context is connected |
| SEC EDGAR filing events | Working | Filing-event context is connected with configured user agent |
| Event context validation | Working | Validation output exists, but the current event sample lacks enough future bars |
| Phase 3 backtesting | Working | Reads SQLite only and validates score behavior, not trade profitability |
| Data quality checks | Working | Doctor checks credentials, required docs, key symbols, ETF coverage, FRED coverage, score coverage, and latest rows |
| Markdown reports | Working | Human market-review output for the requested/latest score date |
| CSV exports | Working | Portable sector and stock-watchlist exports |
| Dashboard API | Working | Localhost-only Rust API reads stored SQLite data |
| Dashboard frontend | Working | Read-only Vite React dashboard over the controlled market-map chain |

## 4. What Is Implemented But Not A Score Input

The following are connected, stored, reported, or validated, but they do not currently alter score formulas:

| Data or feature | Current role | Why it is not a score input yet |
|---|---|---|
| FRED macro observations | Context and validation overlay | Macro formula change requires a separate candidate, fresh comparison, and approval |
| Macro/regime validation results | Evidence layer | Validation explains behavior; it does not tune scores automatically |
| Alpaca News labels | Watchlist context | Event-context rows do not yet have enough forward-bar validation |
| Alpha Vantage earnings dates | Watchlist context | Same event-validation gate applies |
| SEC filing labels | Watchlist context | Same event-validation gate applies |
| Event-context validation results | Evidence layer | Current result has event rows but no forward observations |
| Sector rank change | Reporting and stored context | Rank change is no longer a scoring component |
| Dashboard display | Read-only review surface | Dashboard must not recalculate, fetch new data, or change scores |

## 5. Current Blocking Facts

The Phase 5 readiness gate blocks new score changes because the current evidence is not yet enough.

Current event-context validation state:

```text
Rows with event context: 25
Event-context forward observations: 0
```

Meaning:

- Event context is connected.
- The events are visible in the final watchlist surface.
- The events are not yet validated as improving forward behavior.
- Catalyst/event score changes are not approved.

Current macro state:

- FRED macro context is connected.
- Macro/regime validation exists.
- Market Regime V1 scoring remains unchanged.
- Macro score changes are not approved until a separate candidate formula is documented and compared.

Current sector state:

- Sector scores remain part of the market-map flow.
- Sector ranking is not treated as a standalone trade signal.
- Sector formula changes are not approved without a separate decision and fresh validation.

Current source-expansion state:

- Phase 5D ETF fund-flow implementation is not approved yet.
- Options, intraday, broader universe expansion, and paid data sources remain deferred.

## 6. What Is Still Needed

### 6.1 Operational Readiness

Keep the current system running so event-labeled history can accumulate.

Required recurring checks:

```text
merryl run daily --date latest
merryl doctor
merryl status
```

For development, the equivalent command surface is:

```text
cargo run -- run daily --date latest
cargo run -- doctor
cargo run -- status
```

The purpose is not to add features. The purpose is to build enough real stored history for the already-connected sources to be judged.

### 6.2 Event Context Maturation

The next evidence gap is event-context validation.

Needed:

- More score dates after event-labeled watchlist rows.
- 1D, 5D, 10D, 20D, and 60D forward bars after those event dates.
- Comparison of event-context rows against `pending_source` rows.
- Enough observations to avoid changing formulas from a tiny sample.

Do not change catalyst/event weights until this evidence exists.

### 6.3 Macro Formula Candidate

Macro context is implemented, but a macro-adjusted regime formula is not approved.

Needed before any macro score change:

- A separate macro formula candidate document.
- Clear definition of which FRED series become scoring inputs.
- As-of handling so future macro data cannot leak backward.
- Fresh backtest comparison against Market Regime V1.
- Decision on whether the candidate improves the market-map use case.

### 6.4 Phase 5D ETF Fund-Flow Source Decision

The next planned source candidate is ETF fund flows, but implementation should not start until the source is chosen.

Needed:

- Identify whether a free source exists.
- If no free source exists, confirm whether paid access is approved.
- Confirm historical coverage, ETF coverage, update frequency, and licensing.
- Define storage schema with source provenance.
- Define validation before any sector-score change.

No flow data should be added as a scoring input in the first ingestion pass.

### 6.5 Universe Expansion Gate

The original system can grow beyond the S&P 500, but the docs intentionally keep the first build anchored.

Needed before expansion:

- Universe source decision.
- Sector and industry mapping rules.
- Liquidity and data-quality filters.
- Survivorship-bias handling.
- Backtest impact review.

Until then, the S&P 500 anchor universe is the controlled scope.

### 6.6 Dashboard Application Polish

The dashboard is working as a read-only application surface.

Remaining dashboard work should stay inside the current boundary:

- Improve clarity of stored validation/readiness state.
- Keep views consistent with the leadership/watchlist design language.
- Keep selected-date behavior stable.
- Avoid adding chart-review workflows.
- Avoid dashboard-side recalculation or provider fetching.

## 7. What Should Not Be Built Next

Do not build these next:

- A charting workspace inside Merryl.
- Trade execution.
- Portfolio simulation.
- Position sizing.
- Alerts.
- Options flow.
- Intraday flow.
- Broader stock universe.
- Paid fund-flow data ingestion.
- Automatic formula tuning.

These are not rejected forever. They are blocked until the current validation and source-decision gates are complete.

## 8. Immediate Next Step

The safest next application step is:

```text
Run the current system, accumulate forward bars for event-labeled rows, and rerun validation.
```

If starting new planning work, the next document should be:

```text
Phase 5D ETF Fund-Flow Source Decision Spec
```

That document should be written before any ETF flow implementation.

## 9. Validation Commands

For code changes:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

For application readiness:

```text
cargo run -- run daily --date latest
cargo run -- run backtest --from 2025-07-01 --to <latest scored date>
cargo run -- doctor
cargo run -- status
```

For dashboard verification:

```text
cargo run -- dashboard
```

Then inspect the local dashboard against stored data only.

## 10. Application State Summary

Merryl currently has the foundation working:

- Real data ingestion.
- Local storage.
- Historical scoring.
- Daily reporting.
- Backtesting.
- Macro validation.
- Event context.
- Event validation output.
- Watchlist classification.
- Read-only dashboard.

The main remaining work is not to rebuild the foundation. The remaining work is to accumulate evidence, keep the gates strict, and only promote new sources into score formulas after validation shows that they improve the market-map and final watchlist use case.
