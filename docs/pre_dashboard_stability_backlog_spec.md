# Pre-Dashboard Stability Backlog

Version: 1.2
Date: 2026-05-27
Status: PDB-1 through PDB-6 complete; first Phase 4 dashboard/API slice is implemented
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
- `docs/spec_completeness_gate_spec.md`
- `docs/catalyst_earnings_source_spec.md`
- `docs/backtest_scope_clarity_spec.md`
- `docs/data_quality_reproducibility_spec.md`
- `docs/phase_4_dashboard_api_spec.md`
- `docs/phase_4_dashboard_stabilization_spec.md`

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
market regime scores: 228
score dates: 228
sector scores: 2508
industry scores: 28956
stock scores: 11400
watchlist rows: 5700
backtest results: 8
```

Latest validation state:

```text
PDB-1 industry-specific validation: complete
PDB-2 sector score review: complete
PDB-3 market regime V1 review: complete
PDB-3.5 sector formula decision checkpoint: complete
Spec completeness gate: complete
PDB-4 catalyst/news source: complete
PDB-5 backtest scope clarity: complete
PDB-6 data quality/reproducibility: complete
latest backtest result id: 8
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
| PDB-3 | Market regime coverage labeling or modest hardening | Complete | The spec expects regime context, but current implementation uses ETF proxies for the score. | Replaced vague regime wording with precise score coverage: SPY, QQQ, IWM, DIA, TLT, GLD, and USO are included. Phase 5A/B later connected FRED macro context, but it is not part of regime scoring yet. | `docs/market_regime_v1_spec.md` records the current score coverage and Phase 5 macro-context boundary. |
| PDB-3.5 | Sector formula decision checkpoint | Complete | PDB-2 found that `SECTOR_RANK_CHANGE_WEIGHT` existed, but scoring used a neutral placeholder. PDB-3 was complete, so this could not be pushed forward silently. | Tested Option B2: remove neutral rank-change contribution, renormalize remaining sector weights, keep `rank_change` stored/reported only. | `docs/sector_formula_decision_checkpoint_spec.md` records the accepted decision and validation result. |
| PDB-3.6 | Spec completeness gate | Complete | The project had accumulated reduced-scope, placeholder, and deferred labels that could drift from the main spec if not audited. | Classified every reduced/placeholder area as required now, acceptable first-version scope, deferred by original spec, or incorrect drift. | `docs/spec_completeness_gate_spec.md` records the classification and confirms PDB-4 as the next required implementation task. |
| PDB-4 | Catalyst/earnings decision | Complete | The original spec preserves the question "why is this moving?" Current values were all `pending_source`. | Connected real recent news through Alpaca News using the existing Alpaca key. Kept structured earnings calendar explicitly not connected. Avoided fake catalyst inference. | `docs/catalyst_earnings_source_spec.md` records the source decision; reports show real recent-news headlines where available and do not imply earnings calendar data exists. |
| PDB-5 | Backtest scope clarity | Complete | Current backtest validates score behavior, not trade profitability. | Added report and stored metrics scope explaining what the backtest proves and does not prove. Classified which metrics can be added before dashboard without trade-entry assumptions. | `docs/backtest_scope_clarity_spec.md` records the decision; reports include validation scope and hit-rate meaning. |
| PDB-6 | Data quality and reproducibility check | Complete | Dashboard will depend on confidence in stored data and regenerated reports. | Added `doctor` checks for required symbols, sector maps, price coverage, score-date coverage, and latest score row coverage. Added tests for missing-data detection, complete fixture pass, and idempotent replacement writes. | `docs/data_quality_reproducibility_spec.md` records the implemented gate; `doctor` and tests can reveal missing core data before report/dashboard use. |

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
| Full macro engine | Later regime expansion | Current regime coverage uses explicit ETF proxies; full macro sources are not required before PDB-4. |

## 6. What Is Not A Blocker

These limitations are understood and acceptable for now:

- The first valid score date has no prior rank-change baseline.
- Markdown and CSV are not the system of record.
- S&P 500 is the first universe.
- Daily data is the first timeframe.
- Backtest does not yet model trade profitability.

They should stay documented, but they do not need to stop the next controlled implementation step.

## 7. Completed Pre-Dashboard Order

Completed pre-dashboard order:

1. PDB-6: Data quality and reproducibility check.

Reason this was last:

- Catalyst clarity prevents the dashboard from overstating unfinished context.
- Data quality checks make the eventual dashboard more reliable without adding product complexity.

Status:

```text
PDB-6 is complete.
```

The first controlled Phase 4 dashboard/API slice from `docs/phase_4_dashboard_api_spec.md` is now implemented.

## 8. Acceptance Before Phase 4

Phase 4 dashboard can start when:

- Industry-specific validation has a written result.
- Sector score review has a written decision.
- Market regime coverage is clearly labeled with included and missing data sources.
- Catalyst/earnings status is explicit and not misleading.
- Backtest outputs clearly say score behavior is not trade profitability.
- Status/doctor/tests can catch missing core data.

All listed pre-dashboard blockers are now complete.

The dashboard began as a minimal local dashboard/API slice, not a broad product expansion.

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

Implemented first dashboard slice:

- `merryl dashboard` starts a localhost-only Rust axum server.
- The API reads from SQLite and does not fetch market data or recalculate scores.
- The React dashboard shows market regime, sector rotation, industry/theme strength, stock leadership, watchlist, latest backtest review, data health, and visible limitations.
- Frontend concerns are separated into loading, page composition, reusable components, table columns, and formatting helpers.
- The visual direction follows a dense financial analytics workstation style, with compact tables, thin borders, restrained accents, and no marketing-style landing page.

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
Do not change sector weights further in this pass.
Do not force rank_change back into the sector score yet.
Revisit sector scoring after catalyst/earnings is no longer ambiguous and backtest scope is clearer.
Daily reports label sector ranking as a map-only attention layer.
```

PDB-3 is complete.

Result:

```text
Market regime coverage is explicitly labeled with included and missing sources.
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

PDB-3.6 is complete.

Reason:

```text
The project had accumulated reduced-scope labels such as pending_source, map-only, and deferred.
Those labels have now been audited against the main spec so we do not build a smaller product than intended.
```

Evidence is recorded in:

```text
docs/spec_completeness_gate_spec.md
```

## 10. Current Next Work

PDB-4 is complete.

Result:

```text
Recent news catalysts are connected through Alpaca News.
Structured earnings calendar data is not connected yet.
Stock rows with recent news show recent_news:N.
```

Evidence is recorded in:

```text
docs/catalyst_earnings_source_spec.md
```

PDB-5 is complete.

Result:

```text
Backtest reports validate score behavior, not trade profitability.
metrics_json stores validation_scope.
Hit rate is defined as positive relative forward return, not trade win rate.
```

Evidence is recorded in:

```text
docs/backtest_scope_clarity_spec.md
```

PDB-6 is complete.

Result:

```text
doctor checks required market symbols, sector maps, ETF price coverage, historical score coverage,
latest score-date alignment, and latest score row coverage.
Tests verify missing-data detection, complete fixture acceptance, and idempotent replacement writes.
```

Evidence is recorded in:

```text
docs/data_quality_reproducibility_spec.md
```

That next implementation task is now complete:

```text
Phase 4 first slice: read-only local API plus initial dashboard shell
```
