# Merryl Implementation Runbook

Version: 0.1  
Date: 2026-05-26  
Status: Phase 1 implementation notes

## Current Slice

The current implementation target is:

```text
Merryl daily run
  -> load S&P 500 anchor universe
  -> fetch real daily OHLCV data
  -> score market regime, sectors, industries, and stocks
  -> store results in SQLite
  -> write Markdown and CSV outputs
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

Generated database/report/export files are ignored by git.

## Output Format Policy

SQLite is the canonical local store for Merryl.

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
- Sector rank changes and new leaders require a prior dated run before they can show real movement.
- Catalyst and earnings fields are preserved as `pending_source` until a source is connected.
- Industry scoring currently uses a simple 20-day return proxy and should be expanded later.
- Backtesting is not implemented yet.

## Guardrails

- Keep the public CLI small.
- Do not expose internal scoring or ingest steps as public commands.
- Keep provider code replaceable.
- Keep watchlist output separate from trade signals.
- Preserve the top-down flow: market -> sector -> industry/theme -> stock.
