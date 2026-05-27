# Market Regime V1 Review

Version: 0.2
Date: 2026-05-27  
Status: PDB-3 complete; ETF-proxy regime coverage is explicitly documented

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/phase_3_backtest_validation_spec.md`
- `docs/sector_score_review_spec.md`

## 1. Purpose

This document records the PDB-3 market regime review.

Question:

```text
Can the current market regime layer be shown without users mistaking it for a full macro model?
```

The answer is yes, if the report states the exact regime coverage and missing macro sources instead of using vague reduced-scope wording.

## 2. Decision

Keep Market Regime V1.

Decision:

```text
Market regime coverage uses daily ETF price proxies: SPY, QQQ, IWM, DIA, TLT, GLD, and USO.
It does not yet include VIX, DXY, US10Y, macro calendar, credit, or liquidity data.
It is not a trading signal.
It should help frame the top-down map before sector, industry/theme, and stock review.
```

## 3. What Changed

The daily data universe now includes existing Alpaca daily ETF proxies:

```text
TLT: long-duration Treasury bond proxy
GLD: gold proxy
USO: oil proxy
```

The regime scorer now records these values in `components_json`:

```text
tlt_return_20d
gld_return_20d
uso_return_20d
context_label
```

The daily report now says:

```text
Market regime coverage: daily ETF price proxies SPY, QQQ, IWM, DIA, TLT, GLD, and USO. Not yet included: VIX, DXY, US10Y, macro calendar, credit, or liquidity data.
```

## 4. What Did Not Change

The core regime score weights remain unchanged.

Current weighted score inputs remain:

```text
SPY trend
QQQ relative to SPY
IWM relative to SPY
DIA relative to SPY
```

Reason:

```text
Changing regime weights before validation would be blind formula tuning.
```

The new TLT, GLD, and USO values add context and report transparency first. They do not yet overhaul the score.

## 5. Current Context Labels

The market regime label may add a context suffix when simple ETF proxy thresholds are met:

```text
Inflation-sensitive: GLD and USO are both up at least 3% over 20 trading bars.
Rate-sensitive: TLT is down at least 3% over 20 trading bars.
Defensive bid: SPY is down over 20 trading bars while TLT is up at least 3%.
```

These labels are intentionally simple. They are not a substitute for macro data, rate series, central-bank policy analysis, or inflation data.

## 6. Still Deferred

These remain outside current regime coverage:

- VIX trend.
- DXY trend.
- US10Y / yield curve data.
- Macro calendar and economic surprises.
- Liquidity indicators.
- Credit spreads.
- Full macro regime classification.

They can be added later when a real source is chosen.

## 7. Product Stance

Use regime as context:

```text
Regime tells us what kind of market background we may be operating in.
Sector, industry/theme, and stock behavior still decide where attention goes.
Charts and risk planning still decide trade timing.
```

## 8. Next Work

PDB-3 and PDB-3.6 are complete.

Verification run:

```text
cargo run -- run daily --date latest
date: 2026-05-27
historical score dates: 228
```

Latest stored regime components include:

```text
tlt_return_20d
gld_return_20d
uso_return_20d
context_label
```

The next pre-dashboard item is:

```text
PDB-5: Backtest scope clarity
```

PDB-3.6 decided that current ETF-proxy regime coverage is acceptable for the first build when the wording is precise. PDB-4 connected recent news catalysts through Alpaca News. Additional macro sources are post-MVP macro expansion.
