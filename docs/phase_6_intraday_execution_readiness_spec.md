# Phase 6A Intraday Execution Readiness Spec

Version: 0.1
Date: 2026-06-10
Status: Implemented on `main`

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/watchlist_actionability_extension_filter_spec.md`
- `docs/application_state_remaining_work_spec.md`

## 1. Product Boundary

Phase 6A implements Issue #1 as an execution-readiness signal layer, not as automated trade execution.

The system chain is:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> Actionability queue
          -> Intraday execution-readiness signals
```

Merryl can identify high-momentum names that are structurally close to a long-side intraday setup. It must not place orders, size positions, manage stops, create alerts, or become the charting platform.

## 2. Locked Decisions

- Add one public workflow: `merryl run intraday --date latest`.
- Use existing Alpaca credentials and `ALPACA_FEED`; free delayed/basic data is acceptable for the first pass.
- Implement long-side readiness only.
- Keep current regime, sector, industry, stock, watchlist, and actionability score formulas unchanged.
- Store intraday readiness rows in SQLite and have the dashboard read those rows only.

## 3. Data And Configuration

New optional environment variables:

```text
MERRYL_ALPACA_REQUESTS_PER_MINUTE=180
MERRYL_INTRADAY_PROFILE_TIMEFRAME=30Min
MERRYL_INTRADAY_TRIGGER_TIMEFRAME=5Min
MERRYL_INTRADAY_CANDIDATE_LIMIT=50
MERRYL_INTRADAY_OPENING_RANGE_MINUTES=30
```

Storage additions:

- `prices_intraday.vwap`
- `volume_profiles`
- `intraday_setups`
- `intraday_triggers`

All additions are additive migrations. Repeated intraday runs for the same date replace that date's readiness rows and upsert raw intraday bars.

## 4. Three-Stage Pipeline

### Stage 1: High-Momentum Universe

Stage 1 scans active daily stock history, not only the current top-25 watchlist.

A symbol passes when:

- ADR% over 20 daily bars is greater than 4%.
- rVOL is greater than 1.5x the prior 20-day average.
- Mansfield RS versus SPY ranks in the top 10% of eligible stocks.

The workflow also calculates Mansfield RS versus the stock's sector ETF and EMA 10/20.

### Stage 2: Structural Pullback Setup

For Stage 1 candidates, Merryl fetches 30-minute intraday bars and builds a session volume profile:

- POC: highest-volume rounded HLC3 price bin.
- Price bin width is dynamic and clamped from daily context:
  `min(max(0.01, max(latest_close * 0.0005, ATR20 * 0.05)), latest_close * 0.002)`.
  This avoids cent-level fragmentation on high-priced tickers and avoids over-wide bins on wild gap days.
- VAH/VAL: expansion from POC until at least 70% of session volume is captured.
  Equal-volume expansion intentionally chooses the upper-side bin first. This is recorded as `upper_on_equal_volume` in profile metadata.
- VWAP: session-anchored HLC3 volume-weighted price.

A long-side structural setup requires:

- EMA 10 above EMA 20.
- Latest price within plus/minus 0.75% of at least three of POC, VAL, VWAP, EMA 10, and EMA 20.
- Latest price not materially below EMA 20.

### Stage 3: Intraday Trigger Ready

For Stage 2 candidates, Merryl fetches 5-minute bars and detects:

- `orb_breakout`
- `hod_break`
- `volume_dryup_confirmation`
- `micro_cluster_break`

The final `intraday_execution_ready` label requires Stage 1, Stage 2, and at least one Stage 3 trigger event.

ORB lookback is derived from configuration:

```text
opening_range_bars = ceil(MERRYL_INTRADAY_OPENING_RANGE_MINUTES / trigger_timeframe_minutes)
```

For the default 5-minute trigger timeframe and 30-minute opening range, this equals 6 bars. If the trigger timeframe changes, the ORB window changes with it.

## 5. Outputs

Workflow:

```text
merryl run intraday --date latest
```

Outputs:

```text
reports/intraday/YYYY-MM-DD_intraday_execution_readiness.md
exports/intraday/YYYY-MM-DD_intraday_execution_readiness.csv
```

CLI output prints:

- score date
- database path
- profile timeframe
- trigger timeframe
- candidate count
- Stage 1 count
- Stage 2 count
- Stage 3 trigger count
- report path
- export path

## 6. Dashboard Boundary

The dashboard may show an `Execution Readiness` view, but it must only read stored SQLite rows.

The dashboard must not:

- fetch Alpaca or any provider data,
- recalculate readiness signals,
- add chart review tooling,
- add broker execution controls.

## 7. Validation

Required tests:

- ADR%, rVOL, EMA, Mansfield RS calculations.
- Volume profile POC, VAH, VAL, and VWAP from fixture intraday bars.
- Confluence count inside the 0.75% window.
- ORB, HOD break, dry-up confirmation, and micro-cluster trigger detection.
- `prices_intraday.vwap`, `volume_profiles`, `intraday_setups`, and `intraday_triggers` migrations are idempotent.
- Repeated intraday runs replace readiness rows instead of duplicating them.
- The workflow fails clearly when no prior daily score exists.
- The dashboard API exposes stored execution-readiness rows.

## 8. Deferred

The following remain out of scope:

- Live or paper order placement.
- Alerts.
- Position sizing.
- Stop-loss or trade management.
- Short-side setup support.
- 1-minute trigger default.
- Score-weight changes from intraday signals.
- Dashboard chart-review workflows.
