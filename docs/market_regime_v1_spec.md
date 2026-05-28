# Market Regime V1 Review

Version: 0.2
Date: 2026-05-28
Status: PDB-3 complete; ETF-proxy score remains, Phase 5 FRED macro context is stored separately

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
Market regime score uses daily ETF price proxies: SPY, QQQ, IWM, DIA, TLT, GLD, and USO.
FRED macro series are stored separately as context/provenance after Phase 5A/B.
FRED macro series are not yet scoring inputs.
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
Market regime score: daily ETF price proxies SPY, QQQ, IWM, DIA, TLT, GLD, and USO. FRED macro context is stored separately and is not part of scoring yet.
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

The TLT, GLD, and USO values add ETF-proxy context and report transparency. FRED macro observations add source-backed context/provenance after Phase 5A/B. Neither layer has changed the core score weights.

## 5. Current Context Labels

The market regime label may add a context suffix when simple ETF proxy thresholds are met:

```text
Inflation-sensitive: GLD and USO are both up at least 3% over 20 trading bars.
Rate-sensitive: TLT is down at least 3% over 20 trading bars.
Defensive bid: SPY is down over 20 trading bars while TLT is up at least 3%.
```

These labels are intentionally simple. They are not a substitute for a validated macro regime model.

## 6. Still Deferred

These remain outside current regime scoring:

- FRED volatility, rates, yield curve, dollar proxy, credit, inflation, employment, and liquidity series.
- Macro calendar and economic surprises.
- Full macro regime classification.

The real FRED source is now connected for context. It still requires a separate validation checkpoint before any score change.

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

Phase 5A/B verification run:

```text
cargo run -- run daily --date latest
date: 2026-05-28
macro observations: 4843

cargo run -- doctor
ok: FRED macro coverage present (11/11)
```

Latest stored regime components include:

```text
tlt_return_20d
gld_return_20d
uso_return_20d
context_label
```

Pre-dashboard stability is complete through:

```text
PDB-6: Data quality and reproducibility check
```

PDB-3.6 decided that current ETF-proxy regime coverage is acceptable for the first build when the wording is precise. PDB-4 connected recent news catalysts through Alpaca News. PDB-5 clarified backtest scope. PDB-6 added data quality and reproducibility checks. Phase 5A/B connected FRED macro context without changing regime score weights. The next regime step is validation, not formula tuning.
