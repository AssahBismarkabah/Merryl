# Pre-Dashboard Stability Backlog

Version: 0.6
Date: 2026-05-27
Status: PDB-1, PDB-2, PDB-3, and PDB-3.5 complete; PDB-4 is next before Phase 4 dashboard
Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/implementation_spec.md`
- `docs/phase_3_backtest_validation_spec.md`
- `docs/industry_specific_validation_spec.md`
- `docs/sector_score_review_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/sector_formula_decision_checkpoint_spec.md`

## 1. Purpose

This document defines what must be stabilized before Merryl moves into full dashboard work.

The goal is to avoid building a visual product on top of unstable or unclear engine behavior.

The original product intent remains:

```text
Show where market participation is concentrating,
map that concentration from market regime to sector to industry/theme to stock,
then produce a small list of liquid, chart-worthy names with explainable reasons.
```

The dashboard should visualize a controlled market map. It should not hide unresolved signal-quality questions behind a nicer interface.

## 2. Current State

Implemented:

- Daily market map workflow.
- Real Alpaca daily OHLCV ingestion.
- SQLite storage.
- Market regime V1.
- Sector scoring.
- Industry/theme scoring with transparent return, relative return, volume, breadth, and 20D-high components.
- Stock leadership scoring.
- Watchlist report and CSV exports.
- Historical scoring.
- Backtest workflow.
- Phase 3 validation checkpoint.

Current data state at the latest check:

```text
symbols: 521
daily prices: 150331
score dates: 228
sector scores: 2508
industry scores: 28956
stock scores: 11400
watchlist rows: 5700
backtest results: 7
```

Latest validation state:

```text
PDB-1 industry-specific validation: complete
PDB-2 sector score review: complete
PDB-3 market regime V1 review: complete
PDB-3.5 sector formula decision checkpoint: complete
latest backtest result id: 7
```

## 3. Stability Rule

Do not start the full Phase 4 dashboard until the pre-dashboard blockers are either:

- implemented and validated, or
- explicitly accepted as visible V1 limitations.

This does not mean every advanced feature must be built before dashboard.

It means the core market-map chain must be understandable and controlled:

```text
Market regime -> sector rotation -> industry/theme strength -> stock leadership -> watchlist
```

## 4. Pre-Dashboard Work To Tackle

These items should be addressed before full dashboard work.

| ID | Item | Status | Why It Matters | Required Action | Done When |
|---|---|---|---|---|---|
| PDB-1 | Industry-specific validation | Complete | The industry/theme bridge was hardened, but we had not tested whether stronger industry groups improve stock outcomes. | Extended backtest analysis to compare stock behavior by industry score decile. | `docs/industry_specific_validation_spec.md` records that stronger industries/themes improved stock forward behavior versus weaker industries/themes in the tested window. |
| PDB-2 | Sector score review | Complete | Phase 3 showed sector decile behavior is mixed. Sector ranking is core to the market map. | Analyzed sector score components and identified where the issue is weights, horizon mix, breadth, relative volume, and rank-change design. | `docs/sector_score_review_spec.md` records the decision to keep sector ranking as map-only / not yet a proven forward-return predictor. |
| PDB-3 | Market regime V1 labeling or modest hardening | Complete | The spec expects regime context, but current V1 used only broad ETF proxies. | Labeled regime clearly as lightweight V1 and added existing-data ETF proxies TLT, GLD, and USO for context. | `docs/market_regime_v1_spec.md` records that users should not mistake regime V1 for a full macro model. |
| PDB-3.5 | Sector formula decision checkpoint | Complete | PDB-2 found that `SECTOR_RANK_CHANGE_WEIGHT` existed, but scoring used a neutral placeholder. PDB-3 was complete, so this could not be pushed forward silently. | Tested Option B2: remove neutral rank-change contribution, renormalize remaining sector weights, keep `rank_change` stored/reported only. | `docs/sector_formula_decision_checkpoint_spec.md` records the accepted decision and validation result. |
| PDB-4 | Catalyst/earnings decision | Next | The original spec preserves the question "why is this moving?" Current values are `pending_source`. | Decide whether to connect a real source now or keep it deferred with explicit report wording. Avoid fake catalyst inference. | The report and docs make the catalyst state explicit and do not imply unavailable data exists. |
| PDB-5 | Backtest scope clarity | Pending | Current backtest validates score behavior, not trade profitability. | Keep a clear document/report note explaining what the backtest proves and does not prove. | No output suggests the scores are direct trade entries or profitability claims. |
| PDB-6 | Data quality and reproducibility check | Pending | Dashboard will depend on confidence in stored data and regenerated reports. | Add or run checks for required symbols, price coverage, score-date coverage, and idempotent workflow writes. | `doctor`, `status`, or tests can reveal missing core data before report/dashboard use. |

## 5. Work To Defer Deliberately

These items are real future needs, but they should not block the first controlled dashboard once the pre-dashboard blockers above are addressed.

| Item | Phase | Why Deferred |
|---|---|---|
| Full news/NLP catalyst engine | Phase 5 | Important, but can distract from proving the price/volume/breadth market map. |
| ETF fund flows | Phase 5 | Useful confirmation layer, but requires additional provider selection and data validation. |
| Options flow and gamma exposure | Phase 5 | High cost/complexity; not required for v1 market map. |
| Dark pool prints | Phase 5 | Not required for first useful stock-sector watchlist product. |
| Intraday/live data | Later Phase 5 or dedicated execution phase | Current system is daily-first by design. Intraday changes the workflow and data volume. |
| Portfolio simulation | After score validation | Merryl is a watchlist engine first. Portfolio simulation should come after signal behavior is understood. |
| Transaction-cost and slippage modeling | After portfolio simulation is scoped | Required for trade profitability, not required for watchlist validation. |
| Sharpe/Sortino and full risk analytics | After portfolio simulation is scoped | These metrics need portfolio assumptions. |
| Russell 1000/Russell 3000/all liquid US stocks | Later universe expansion | S&P 500 is the controlled first universe. Expansion should come after core logic is stable. |
| Custom AI theme classification | Later enrichment | GICS sector/industry is the current standard mapping. |
| Full macro engine | Later regime expansion | Market regime V1 can remain lightweight if clearly labeled. |

## 6. What Is Not A Blocker

These limitations are understood and acceptable for now:

- The first valid score date has no prior rank-change baseline.
- Markdown and CSV are not the system of record.
- S&P 500 is the first universe.
- Daily data is the first timeframe.
- Backtest does not yet model trade profitability.

They should stay documented, but they do not need to stop the next controlled implementation step.

## 7. Recommended Order

Recommended pre-dashboard order:

1. PDB-4: Catalyst/earnings decision.
2. PDB-5: Backtest scope clarity.
3. PDB-6: Data quality and reproducibility check.

Reason:

- Catalyst clarity prevents the dashboard from overstating unfinished context.
- Data quality checks make the eventual dashboard more reliable without adding product complexity.

## 8. Acceptance Before Phase 4

Phase 4 dashboard can start when:

- Industry-specific validation has a written result.
- Sector score review has a written decision.
- Market regime V1 is clearly labeled or modestly hardened.
- Catalyst/earnings status is explicit and not misleading.
- Backtest outputs clearly say score behavior is not trade profitability.
- Status/doctor/tests can catch missing core data.

The dashboard should begin as a minimal local dashboard/API slice, not a broad product expansion.

Initial dashboard scope should remain:

```text
Market regime
Sector rotation
Industry/theme strength
Stock leadership
Watchlist
Historical score/backtest review
```

No alerts, portfolio simulation, intraday execution, options flow, or advanced data layer should be added in the first dashboard slice.

## 9. Completed Work

PDB-1 is complete.

Result:

```text
Industry/theme strength is validated enough to remain part of the core market map.
```

Evidence is recorded in:

```text
docs/industry_specific_validation_spec.md
```

Decision:

```text
Use industry/theme strength as an attention and confirmation layer.
Do not change stock scoring weights yet.
```

PDB-2 is complete.

Result:

```text
Sector ranking stays as a market-map and attention layer.
It is map-only / not yet a proven forward-return predictor.
```

Evidence is recorded in:

```text
docs/sector_score_review_spec.md
```

Decision:

```text
Do not change sector weights yet.
Do not force rank_change into the sector score yet.
Revisit sector scoring after market regime V1 is clearly labeled or modestly hardened.
Daily reports label sector ranking as a map-only attention layer.
```

PDB-3 is complete.

Result:

```text
Market Regime V1 is explicitly labeled as lightweight context.
The data universe now includes TLT, GLD, and USO as existing-data ETF proxies.
Regime output records those proxy returns in components_json.
```

Evidence is recorded in:

```text
docs/market_regime_v1_spec.md
```

Decision:

```text
Keep the core regime score weights unchanged.
Use new proxy values as context, not a full macro model.
Do not add a new macro provider in this pass.
```

PDB-3.5 is complete.

Result:

```text
Option B2 accepted.
The neutral rank-change contribution was removed from sector scoring.
Remaining sector weights are normalized.
rank_change remains stored and reported, but it is not a scoring component.
```

Evidence is recorded in:

```text
docs/sector_formula_decision_checkpoint_spec.md
```

## 10. Current Next Work

The next implementation task should be:

```text
PDB-4: Catalyst/earnings decision
```

Expected output:

- A clear decision to connect a real catalyst/earnings source now or keep it deferred.
- Daily report wording that does not imply catalyst data exists when it is still `pending_source`.
- No fake catalyst inference.
