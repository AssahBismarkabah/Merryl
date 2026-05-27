# Backtest Scope Clarity

Version: 0.1
Date: 2026-05-27
Status: PDB-5 complete; PDB-6 is next

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/phase_3_backtest_validation_spec.md`

## 1. Purpose

PDB-5 exists to prevent the backtest from being mistaken for a trading-profitability simulator.

Merryl is currently a market rotation and watchlist engine. The backtest should answer:

```text
Did stronger same-day scores tend to have better forward behavior than weaker scores?
```

It should not imply:

```text
This score is a complete trading system.
```

## 2. Decision

Keep the current backtest as score-behavior validation.

Decision:

```text
Backtest output validates score behavior, not trade profitability.
```

The backtest can show:

- whether higher same-day scores were followed by stronger forward returns.
- whether sector, sector-component, stock, and industry/theme deciles behaved differently.
- whether relative forward returns were positive against the configured comparison policy.

The backtest does not prove:

- trade profitability.
- entry or exit timing quality.
- position sizing.
- portfolio construction.
- taxes.
- transaction costs.
- slippage.
- execution liquidity.
- that a score is a buy or sell signal.

## 3. Hit Rate Meaning

Backtest hit rate means:

```text
Share of observations with positive relative forward return under the row's comparison policy.
```

It is not a trade win rate.

For sectors, relative return is sector ETF forward return minus SPY forward return.

For stocks, primary relative return is stock forward return minus the stock's sector ETF forward return.

## 4. Metrics Allowed Before Dashboard

These can be added before dashboard without creating a portfolio/trade simulator:

- return dispersion by decile.
- decile membership turnover.
- path drawdown/runup from score-date close over each horizon, labeled as path behavior rather than trade P&L.

These metrics still describe score behavior, not executable trading profit.

## 5. Metrics Deferred Until Trade Model Exists

These require explicit trade-entry, sizing, portfolio, or execution assumptions and should not be added to the current backtest as if they are already valid:

- transaction costs.
- slippage.
- taxes.
- position sizing.
- portfolio P&L.
- Sharpe, Sortino, and other portfolio-risk metrics.

## 6. Implementation

The backtest result now stores `validation_scope` inside `metrics_json`.

The Markdown backtest report now includes:

- validation purpose.
- what the report can show.
- what the report does not prove.
- hit-rate definition.
- metrics allowed before dashboard.
- metrics deferred until a trade/portfolio model exists.

No public CLI command was added.

No scoring formula was changed.

## 7. Done When

PDB-5 is done when:

- backtest reports explicitly say they validate score behavior, not trade profitability. Complete.
- `metrics_json` stores the validation scope. Complete.
- hit-rate meaning is explicit. Complete.
- additional metrics are classified as allowed before dashboard or deferred until trade model. Complete.
- no output implies scores are direct trade entries or profit claims. Complete.
