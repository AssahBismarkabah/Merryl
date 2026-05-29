# Phase 5 Readiness Gate Spec

Version: 0.1
Date: 2026-05-29
Status: Active control document before the next Phase 5 implementation

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5b_macro_regime_validation_spec.md`
- `docs/market_regime_formula_decision_checkpoint_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/phase_5c_event_context_validation_spec.md`
- `docs/phase_5c_source_coverage_review_spec.md`
- `docs/watchlist_convergence_review_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

This document prevents Phase 5 from drifting into new sources or formula changes before the current evidence is strong enough.

The product remains:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> classified watchlist for chart review elsewhere
```

The immediate question is:

```text
What is safe to do next after Phase 5A/B macro context and Phase 5C event context are connected?
```

## 2. Current State

Implemented:

- FRED macro ingestion.
- Macro/regime validation.
- Non-scoring macro context overlay.
- Alpaca News catalyst context.
- Alpha Vantage earnings calendar context.
- SEC EDGAR filing-event context.
- Watchlist classification labels.
- Event-context validation outputs.

Latest recorded event-context validation result:

```text
Rows with event context: 25
Event-context forward observations: 0
```

Interpretation:

```text
Event context exists, but it does not yet have enough future bars for forward validation.
```

Therefore:

- Catalyst/event score changes are not approved.
- Macro score changes are not approved.
- Phase 5D data-source implementation is not approved until source access, cost, and validation design are clear.

## 3. Guardrails

Do not:

- Change sector, industry, stock, or regime score weights from the current evidence.
- Add paid ETF fund-flow, options, or market-data sources without a source decision document.
- Add public CLI commands for individual source internals.
- Expand the stock universe before sector/industry mapping and data-health gates are defined.
- Add charting, trade execution, portfolio simulation, alerts, or position sizing.
- Treat validation output as trade profitability.
- Treat `pending_source` as bearish, neutral, or a trading signal.

Do:

- Keep source-backed context visible.
- Keep scoring changes behind separate formula decision documents.
- Keep all new source data stored with provenance before it affects scoring.
- Keep the final output focused on classification and filtering of chart-worthy names.

## 4. Current Decisions

| Area | Current decision | Why |
|---|---|---|
| Macro context | Keep as non-scoring overlay | Macro validation showed availability and disagreement, but not score improvement |
| Catalyst/event context | Keep as non-scoring watchlist context | Event labels have no forward-bar validation yet |
| Phase 5D ETF fund flows | Do not implement yet | Likely paid, source access not approved, validation plan not written |
| Options/intraday | Defer | Higher complexity and not required for current daily rotation map |
| Universe expansion | Defer | Mapping, liquidity, and survivorship-bias gates are not written |
| Dashboard charting | Do not add | Merryl should identify what to chart elsewhere, not become the chart-review tool |

## 5. Immediate Next Plan

### Step 1: Keep The Current System Running

Continue running:

```text
merryl run daily --date latest
```

Reason:

```text
The event-context validation needs event-labeled score dates with future price bars.
```

### Step 2: Rerun Validation As History Accumulates

Rerun:

```text
merryl run backtest --from 2025-07-01 --to <latest scored date>
```

Track:

- `event_context_validation.event_context_row_count`
- `event_context_validation.event_context_forward_observation_count`
- event-context group rows in `reports/validations/*_event_context_validation.md`

### Step 3: Review Before Any Formula Change

Do not change score weights until the validation report shows enough event-context forward observations to compare groups.

Minimum review sequence:

| Review point | Use |
|---|---|
| After 1 trading bar | Smoke check that event-context forward observations are no longer zero |
| After 5 trading bars | First short-horizon sanity review |
| After 20 trading bars | First useful medium-horizon review |
| After 60 trading bars | Durable validation review before any long-horizon formula discussion |

Do not tune weights from one short sample.

### Step 4: If Starting New Phase 5 Work, Start With A Source Decision Document

The next source candidate in the Phase 5 order is:

```text
Phase 5D: ETF Fund Flows
```

But implementation should not start until a separate Phase 5D source decision document answers:

- Is there a free source?
- If not free, is paid access approved?
- Which ETFs are covered?
- Does the source provide historical rows for validation?
- What are `effective_date`, `processed_at`, and `fetched_at` semantics?
- How will flow data be stored before it affects scoring?
- What validation must pass before any sector-score change?

## 6. Readiness Gates

### Gate A: Current System Stable

Required:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
merryl doctor
merryl status
```

Acceptance:

- Daily workflow runs with real providers.
- Backtest workflow writes score, macro, and event validation outputs.
- Dashboard remains read-only and uses stored SQLite data.

### Gate B: Event Context Has Forward History

Required before catalyst/event score discussion:

- Event-context rows exist.
- Event-context forward observations are greater than zero.
- Event-context groups show enough rows to compare against `pending_source`.
- Missing forward bars are reported, not hidden.

Recommended before any score-weight change:

- At least 20 trading bars of event-labeled history.
- Prefer 60 trading bars before durable formula changes.
- A separate catalyst/event formula decision document.

### Gate C: Macro Score Change Candidate

Required before macro score changes:

- Candidate macro-adjusted formula written in a separate document.
- Same historical date range compared against Market Regime V1.
- No future macro observations.
- Fresh backtest and validation outputs.
- Decision explicitly records whether the candidate improves market-map usefulness.

### Gate D: Phase 5D ETF Flow Source Readiness

Required before ETF flow implementation:

- Source access and cost approved.
- Source licensing is acceptable.
- Historical coverage supports validation.
- Storage schema preserves provenance.
- No sector score change in the first ingestion pass.

## 7. What Needs Testing

For documentation-only readiness work:

```text
rg "next target|superseded|score weights|Phase 5D" docs
git diff --check
```

For any Rust code change:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

For daily data-source health:

```text
set -a; source .env; set +a; cargo run -- run daily --date latest
set -a; source .env; set +a; cargo run -- doctor
cargo run -- status
```

For validation readiness:

```text
set -a; source .env; set +a; cargo run -- run backtest --from 2025-07-01 --to <latest scored date>
```

Review generated files:

```text
reports/validations/<from>_<to>_macro_regime_validation.md
reports/validations/<from>_<to>_event_context_validation.md
exports/validations/<from>_<to>_event_context_validation.csv
```

## 8. Current Recommendation

Do not implement Phase 5D yet.

The next concrete work should be:

1. Keep daily runs active so event-context history accumulates.
2. Rerun event-context validation after future bars exist.
3. If we want a new implementation track immediately, create a Phase 5D ETF fund-flow source decision document first, not code.

Decision:

```text
Current next step = readiness monitoring and Phase 5D source decision planning.
No score formula changes are approved.
No paid source implementation is approved.
```
