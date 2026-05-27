# Merryl Implementation Runbook

Version: 0.1  
Date: 2026-05-26  
Status: Daily scoring and Phase 3 backtesting implementation notes

## Current Slice

The current implementation target is:

```text
Merryl daily run
  -> load S&P 500 anchor universe
  -> fetch real daily OHLCV data
  -> score the valid historical market window
  -> store market regime, sector, industry, stock, and watchlist rows in SQLite
  -> explain industry/theme strength with return, relative return, volume, breadth, and 20D-high components
  -> write Markdown and CSV outputs for the latest/requested score date

Merryl backtest run
  -> read historical scores and daily prices from SQLite
  -> calculate forward 1D, 5D, 10D, 20D, and 60D returns from future trading bars
  -> group daily sector and stock scores into deciles
  -> group stock forward behavior by same-day industry/theme score decile
  -> summarize hit rate, average return, median return, and relative return behavior
  -> store metrics in backtest_results
  -> write Markdown and CSV backtest summaries
```

The implementation does not generate fake market candles. If real data credentials are missing, the daily run stops.

## Code Structure

Source code is grouped by responsibility:

```text
src/config/      centralized constants, paths, env var names, defaults, scoring weights
src/domain/      shared domain models
src/data/        provider trait, Alpaca adapter, S&P 500 universe, sector ETF mapping
src/storage/     SQLite connection, schema migration, write repositories
src/scoring/     indicators, sector scoring, industry scoring, stock scoring, market orchestration
src/output/      report paths, Markdown rendering, CSV exports
src/workflows/   user workflows such as daily run, status, and doctor
```

Rules for new code:

- Put constants and tunable numbers in `src/config/mod.rs` first.
- Keep provider-specific API details inside `src/data/`.
- Keep database schema in `src/storage/schema.rs`.
- Keep persistence writes in `src/storage/write_repository.rs`.
- Keep formulas in `src/scoring/`, split by market level.
- Keep generated folders/files out of git; commands create them when needed.
- Put tests in `/tests`, not inside source modules.

## Data Provider

Current provider:

```text
Alpaca Market Data API
```

Required environment variables:

```text
ALPACA_API_KEY_ID
ALPACA_API_SECRET_KEY
```

Optional environment variables:

```text
ALPACA_FEED=iex
ALPACA_DATA_URL=https://data.alpaca.markets
MERRYL_LOOKBACK_DAYS=420
```

The default feed is `iex`, which is the practical free-tier starting point. If the account supports a different feed later, set `ALPACA_FEED`.

## Commands

Doctor:

```text
cargo run -- doctor
```

Create or migrate the local database:

```text
cargo run -- db migrate
```

Run the daily workflow:

```text
cargo run -- run daily --date latest
```

Run the backtest workflow:

```text
cargo run -- run backtest --from YYYY-MM-DD --to YYYY-MM-DD
```

Check stored data counts:

```text
cargo run -- status
```

The installed binary command name is:

```text
merryl
```

During development, `cargo run -- ...` invokes that same command surface.

## Outputs

Database:

```text
data/market.db
```

Report:

```text
reports/YYYY-MM-DD_market_report.md
```

Current report sections:

```text
Market Regime
Top Sectors
Weak Sectors
Sector Rank Changes
Top Industries Or Themes
Top Stocks Worth Charting
New Leaders
High Relative Volume Names
Catalyst / Earnings Flags
Notes For Chart Review
Why These Names
```

CSV exports:

```text
exports/YYYY-MM-DD_sector_scores.csv
exports/YYYY-MM-DD_stock_watchlist.csv
```

Backtest outputs:

```text
reports/backtests/YYYY-MM-DD_YYYY-MM-DD_backtest_report.md
exports/backtests/YYYY-MM-DD_YYYY-MM-DD_backtest_summary.csv
```

Phase 3 validation:

```text
docs/phase_3_backtest_validation_spec.md
```

Industry-specific validation:

```text
docs/industry_specific_validation_spec.md
```

Pre-dashboard stability backlog:

```text
docs/pre_dashboard_stability_backlog_spec.md
```

Generated database/report/export files are ignored by git.

## Output Format Policy

SQLite is the canonical local store for Merryl.

The daily workflow stores historical score rows for every valid date in the fetched window. A valid score date has at least 60 benchmark bars available, matching the longest current scoring lookback.

The backtest workflow uses only stored SQLite data. It does not fetch new prices and does not change scoring formulas.

Markdown and CSV are Phase 0 outputs:

- Markdown is the human market-review report.
- CSV is the portable spreadsheet/export format for sector scores and watchlists.

This is acceptable for the first implementation because the daily use case needs readable review and easy inspection. Later formats should be added only when a real workflow requires them:

- JSON from a local API for the dashboard.
- Parquet only if large historical analytics or columnar research becomes necessary.
- PDF only if static sharing becomes a real requirement.

Do not treat Markdown or CSV as the system-of-record.

## Current Limitations

- Market regime V1 uses broad ETF price proxies only: SPY, QQQ, IWM, and DIA.
- The first valid score date in a fetched window has no prior rank-change baseline.
- Catalyst and earnings fields are preserved as `pending_source` until a source is connected.
- Industry scoring now uses transparent price, relative return, volume, breadth, and 20D-high components. Industry-specific validation is supportive, but it still does not include news/catalyst or industry ETF/fund-flow confirmation.
- Backtesting validates score behavior, not trade profitability. It does not model transaction costs, slippage, taxes, position sizing, portfolio constraints, or maximum adverse/favorable excursion yet.

## Current Next Step

The Phase 3 validation checkpoint found that stock scoring has useful forward behavior, sector scoring is mixed, and the industry/theme layer needed hardening before dashboard work. The industry/theme scoring hardening pass is implemented, and PDB-1 industry-specific validation is complete.

Next implementation priority:

```text
PDB-2: Sector score review.
```

This comes from `docs/pre_dashboard_stability_backlog_spec.md`.

The goal is to review sector score component behavior and decide whether to keep the current sector score, adjust the formula, or label sector ranking as map-only until more evidence exists. Do not start the full Phase 4 dashboard before this checkpoint and the other pre-dashboard blockers are either resolved or explicitly accepted as visible V1 limitations.

## Guardrails

- Keep the public CLI small.
- Do not expose internal scoring or ingest steps as public commands.
- Keep provider code replaceable.
- Keep watchlist output separate from trade signals.
- Preserve the top-down flow: market -> sector -> industry/theme -> stock.
