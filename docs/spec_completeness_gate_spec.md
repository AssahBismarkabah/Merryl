# Spec Completeness Gate

Version: 0.2
Date: 2026-05-27
Status: Complete; PDB-4 has been implemented after this gate

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/sector_score_review_spec.md`
- `docs/sector_formula_decision_checkpoint_spec.md`

## 1. Purpose

This gate exists because the project has accumulated several "V1", "lightweight", "pending", and "deferred" labels.

The original system goal is not to build a generic stock scanner or a half-built proxy dashboard. The goal remains:

```text
Show where market participation is concentrating,
map that concentration from market regime to sector to industry/theme to stock,
then produce a small list of liquid, chart-worthy names with explainable reasons.
```

Before PDB-4 continues, every reduced implementation must be classified:

```text
Required now
Acceptable first-version scope
Deferred by original spec
Incorrect drift to fix
```

## 2. Principle

Do not use "lightweight" as a way to avoid implementing core spec behavior.

The spec allows a first version, but it still requires a real market rotation system:

- Real daily OHLCV data.
- Broad market ETF context.
- Sector ETF data.
- Sector and industry mapping.
- Sector ranking.
- Industry ranking.
- Stock ranking.
- Watchlist generation.
- Historical score storage.
- Backtesting and validation.
- Explainable outputs.

Anything needed for those items should be implemented or explicitly blocked by a concrete dependency, not hand-waved as lightweight.

## 3. Current Reduced Or Placeholder Areas

| Area | Current State | Spec Risk | Gate Decision |
|---|---|---|---|
| Market regime | Uses daily ETF price proxies: SPY, QQQ, IWM, DIA, TLT, GLD, USO. Missing VIX, DXY, US10Y, macro calendar/surprises. | Can underrepresent macro/rates/liquidity context. | Accept for first build because the main spec's MVP must-have list requires SPY/QQQ/IWM/DIA and does not require full macro feeds. Replace vague wording with exact source/coverage language. Add VIX/DXY/US10Y/macro calendar to post-MVP macro expansion. |
| Catalyst/earnings | `pending_source` only. | Weakens the "why is this moving?" layer. | Required now. PDB-4 must choose and implement a real source or remove catalyst claims from user-facing ranking. This is the next implementation task. |
| Sector ranking | Map-only; not proven forward-return signal. | Dashboard could overstate sector score as predictive. | Accept as market-map layer. Keep honest wording and do not present as a trade signal. Formula semantics are fixed by PDB-3.5. |
| Industry/theme | GICS industry mapping only. No custom theme engine, no industry ETF/fund-flow confirmation. | May miss narrative/theme rotations. | Accept for first build because GICS industry mapping is in Phase 0 as the standard first mapping. Add theme baskets later after core dashboard/report workflow is controlled. |
| Universe | S&P 500 anchor universe only. | Misses non-S&P leaders. | Accept for first build because Phase 0 explicitly chose S&P 500 anchor universe first. Expansion to Russell 1000 / liquid US stocks is a later universe expansion, not a blocker before PDB-4. |
| Timeframe | Daily data only. | No intraday/live flow. | Accept for first build because daily data is the selected first timeframe. Intraday/live belongs to later execution/timing work, not PDB-4. |
| Provider/feed | Alpaca daily data, default IEX feed. | Coverage/quality may be below production expectation. | Accept for first build because the provider adapter works with real data. Upgrade provider/feed later if validation or coverage fails. |
| Backtest | Score behavior only; no MAE/MFE, volatility, turnover, transaction costs, slippage, survivorship correction. | Could be mistaken for trade profitability validation. | Partially required before dashboard. PDB-5 must add scope clarity and decide which metrics can be computed without portfolio assumptions. MAE/MFE, volatility, and turnover are candidates before dashboard; transaction costs/slippage require trade-entry assumptions. |
| Output | Markdown/CSV reports. | Not the final product surface. | Accept for pre-dashboard validation because the main spec allows a basic dashboard or report. Do not treat Markdown/CSV as final UX. |
| Dashboard | Not built yet. | Visual product is still absent. | Accept until pre-dashboard blockers close. The main spec allows a report first, but Phase 4 should still produce the basic dashboard. |
| Advanced participation | No ETF flows, options, gamma, dark pools, COT, filings. | Less direct participation evidence. | Defer by original spec. These are Phase 5 / advanced layers unless the user changes scope. |

## 4. What The Main Spec Clearly Allows To Defer

The main spec says these can be left out initially:

- Dark pool data.
- Full Level 2/order book.
- 0DTE gamma modeling.
- Dealer hedging models.
- Complex options flow interpretation.
- AI prediction.
- Automated trade execution.
- Broker integration.
- Intraday scalping tools.
- Insider filings.
- 13F filings.
- Full NLP news analysis.
- Portfolio optimization.

These should stay deferred unless the user explicitly changes the product phase.

## 5. What The Main Spec Makes Core

These should not be treated as optional:

- Daily OHLCV data.
- Sector ETF data.
- S&P 500 or broad stock universe.
- Sector and industry mapping.
- Sector ranking.
- Stock ranking.
- Watchlist generation.
- Historical backtest.
- Basic dashboard or report.
- Relative strength.
- Relative volume.
- Breadth.
- Liquidity filters.
- Explainability.

If any of these are incomplete, they should be fixed before dashboard work continues.

## 6. Immediate Gate Questions

Gate answers:

1. Current ETF-proxy regime coverage is sufficient for the first build if wording is precise. VIX/DXY/US10Y/macro calendar are not required before PDB-4.
2. S&P 500 is enough for the first build. Universe expansion is not required before PDB-4.
3. Daily data is enough for the first build. Intraday/live data is not required before PDB-4.
4. GICS industries are enough for the first build. Theme baskets are not required before PDB-4.
5. Backtest scope needs a dedicated PDB-5 decision before dashboard. It does not block PDB-4.
6. Yes. User-facing "lightweight V1" wording should be replaced with precise source/coverage wording.
7. Markdown/CSV is enough for pre-dashboard validation. Basic dashboard remains Phase 4 after pre-dashboard blockers close.

## 7. Proposed Standard Going Forward

Replace vague wording:

```text
lightweight V1
```

with precise wording:

```text
Uses these sources: ...
Does not yet include these required/optional sources: ...
Decision: accepted for first build / blocked / must implement now.
```

This prevents "lightweight" from hiding missing work.

## 8. Required Next Tasks

Immediate task before PDB-4:

```text
Replace vague user-facing regime wording with precise source/coverage wording.
```

Next implementation after this gate was:

```text
PDB-4: Catalyst/earnings decision
```

Reason:

```text
Catalyst/earnings was the remaining core "why is this moving?" placeholder.
The gate classified it as required before dashboard work.
```

Later required before dashboard:

- PDB-5: Backtest scope clarity, including MAE/MFE, volatility, turnover, and what requires trade-entry assumptions. Complete.
- PDB-6: Data quality and reproducibility checks. Complete.
- Phase 4: Basic dashboard/reporting surface after blockers close.

## 9. Done When

This gate is done when:

- Every reduced/placeholder area is classified. Complete.
- Required-now gaps are converted into implementation tasks. Complete.
- Deferred items are explicitly tied to the main spec's "left out initially" or "advanced later" sections. Complete.
- PDB-4 is either confirmed as next or replaced by a higher-priority implementation gap. Complete: PDB-4 was confirmed and then implemented.
- Docs and user-facing regime output no longer use vague "lightweight" wording where exact source/coverage language is needed. Complete for current implementation docs, regime explanations, `components_json` source notes, and daily report wording.
