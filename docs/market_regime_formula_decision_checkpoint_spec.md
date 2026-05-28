# Market Regime Formula Decision Checkpoint Spec

Version: 0.2
Date: 2026-05-28
Status: Implemented; Market Regime V1 score remains unchanged

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5b_macro_regime_validation_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

This checkpoint decides what to do after Phase 5B macro/regime validation.

The question is:

```text
Should FRED macro context change Market Regime V1 scoring now?
```

Decision:

```text
Do not change Market Regime V1 scoring yet.
Implement a non-scoring macro context overlay first.
```

This preserves the main market-map chain:

```text
Macro regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> Watchlist for chart review elsewhere
```

It also avoids turning a newly validated context layer into an untested scoring formula.

## 2. Inputs Reviewed

Source specs reviewed:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5b_macro_regime_validation_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/implementation_spec.md`

Generated validation report reviewed:

```text
reports/validations/2025-07-01_2026-05-28_macro_regime_validation.md
```

Validation summary:

| Metric | Value |
|---|---:|
| Score dates | 229 |
| Macro snapshots | 229 |
| Complete macro snapshots | 229 |
| Missing macro snapshots | 0 |
| Snapshots with at least one stale macro series | 19 |
| Risk-on dates with active macro stress flags | 103 |
| Defensive/mixed dates with no active macro stress flags | 3 |

Macro flag summary:

| Flag | Active dates | Risk-on dates while active | Defensive/mixed dates while active |
|---|---:|---:|---:|
| volatility_stress | 47 | 1 | 36 |
| rate_pressure | 112 | 53 | 52 |
| yield_curve_inversion | 0 | 0 | 0 |
| credit_stress | 66 | 23 | 39 |
| dollar_pressure | 117 | 52 | 57 |
| liquidity_tightening | 99 | 52 | 47 |
| inflation_pressure | 128 | 69 | 49 |
| labor_cooling | 147 | 69 | 78 |

## 3. Interpretation

The validation proves that macro context is available and useful for review.

It shows:

- FRED macro coverage is complete across the scored window.
- Date alignment can be handled without future macro observations.
- Macro flags expose real disagreement between ETF-proxy regime and macro context.
- Sector leadership differs under macro stress flags.

It does not prove:

- A macro-adjusted score is better than the current ETF-proxy score.
- A macro flag should be bullish or bearish by itself.
- Trade profitability.
- Point-in-time macro-vintage correctness.

The important finding is:

```text
Macro context should be visible in the daily market regime review,
but it should not yet alter the numerical regime score.
```

## 4. Decision Options

### Option A: Keep Market Regime V1 Unchanged And Only Keep Validation Reports

Pros:

- Safest.
- No scoring churn.
- No risk of adding unvalidated macro penalties.

Cons:

- Daily market regime still hides useful macro disagreement unless the user opens the validation report.
- Macro context remains less visible than the main spec intended.

Decision:

```text
Reject as too passive.
```

### Option B: Add Non-Scoring Macro Context Overlay

Description:

```text
Keep the ETF-proxy Market Regime V1 score unchanged.
Add macro context flags beside the regime score in daily report/dashboard data.
```

Example:

```text
Risk-on / macro stress: rate pressure, dollar pressure, labor cooling
```

Pros:

- Preserves current validated scoring behavior.
- Adds the missing macro "why" context to the daily review.
- Makes disagreement visible without implying a buy/sell signal.
- Fits the main spec's top-down macro-regime requirement.
- Does not require a paid source.
- Does not add CLI bloat.

Cons:

- Still not a macro-aware score.
- Needs careful language so users do not treat flags as trade signals.

Decision:

```text
Accept as the next implementation target.
```

### Option C: Add Macro Penalty/Bonus To Market Regime Score

Description:

```text
Adjust Market Regime V1 score based on macro stress flags.
```

Pros:

- Produces one cleaner macro-aware score.
- Directly addresses ETF-proxy disagreement.

Cons:

- Requires choosing weights.
- Could degrade useful ETF-price information.
- Requires fresh comparison backtest before acceptance.
- Current validation did not prove scoring improvement.

Decision:

```text
Defer.
```

### Option D: Replace ETF-Proxy Regime With Macro-Only Regime

Pros:

- More macro-pure.

Cons:

- Violates the current first-build design.
- Removes price confirmation from regime.
- Overreacts to FRED data that is delayed, revised, and frequency-mixed.
- Weakens the "what is moving?" part of the main spec.

Decision:

```text
Reject.
```

## 5. Accepted Next Implementation

Implement a non-scoring macro context overlay.

Scope:

- Reuse the existing macro flag logic from Phase 5B validation.
- Build an as-of macro context for the current/latest report date.
- Add macro context flags to the daily report's Market Regime or Macro Context section.
- Add macro context fields to dashboard-ready data if needed by the existing dashboard.
- Keep Market Regime V1 numerical score and label calculation unchanged.
- Keep `merryl run daily --date latest` as the execution surface.
- Do not add a new public CLI command.

Example output:

```text
ETF-proxy regime: Risk-on, score 71.5
Macro context: rate_pressure, dollar_pressure, labor_cooling
Interpretation: ETF price action is risk-on while macro stress flags remain active.
```

## 6. Non-Goals

This next implementation must not:

- Change Market Regime V1 score weights.
- Change sector, industry, or stock score weights.
- Add paid data sources.
- Add ETF fund flows.
- Add options flow.
- Add intraday execution logic.
- Add portfolio simulation or trade recommendations.
- Add a new dashboard screen.
- Add a public `merryl run macro` command.

Phase 5C remains accepted as catalyst/event context only. Catalyst flags must not be pulled into this regime formula decision.

## 7. Acceptance Criteria

The macro context overlay is accepted when:

- Daily report shows current as-of macro flags near Market Regime or Macro Context.
- The report explicitly says macro flags are context, not score inputs.
- Dashboard data can expose the same macro context without recalculating scores in the frontend.
- Existing Market Regime V1 score and label values are unchanged for the same price data.
- Existing tests pass.
- A focused test proves future macro observations are not used for a historical report date.
- `docs/implementation_spec.md` records the overlay as implemented after completion.

Implementation result:

- Daily reports now include a `Macro Context Overlay` section after `Market Regime`.
- The overlay uses Phase 5B as-of macro flag logic from stored FRED observations.
- The report states that macro flags are context only and are not Market Regime V1 score inputs.
- Dashboard API data exposes the same stored-context overlay under `regime.macro_context`.
- Market Regime V1 score and label calculation remains ETF-proxy based and unchanged.
- No new public CLI command was added.
- Tests cover report rendering, dashboard/API exposure, and no future macro observations.

## 8. Required Follow-Up Before Any Formula Change

A scoring formula change can only happen after a separate document compares:

```text
ETF-proxy Market Regime V1
  vs
ETF-proxy + macro overlay
  vs
macro-adjusted Market Regime candidate
```

Required evidence:

- Same historical scored date range.
- Same price and macro inputs.
- No future macro observations.
- Fresh backtest comparison.
- Clear decision on whether the macro-adjusted score improves market-map usefulness.

Until then:

```text
Macro context is visible context, not a score input.
```
