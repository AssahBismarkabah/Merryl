# Merryl

*Most stock tools hand you a pile of tickers. Merryl starts one level higher: where is participation concentrating, which parts of the market are leading, and which stocks are actually worth charting from there?*

The idea: build a local market map that keeps you from randomly scanning names. Merryl pulls real market and context data, scores the market from the top down, stores the history in SQLite, writes a daily report, and serves a small read-only dashboard. You still do chart review elsewhere. Merryl just helps decide what deserves attention first.

More background is in the [market rotation system spec](docs/market_rotation_system_spec.md).

## How it works

Merryl follows the order a serious market review should follow. It looks at the broad market first, then sector rotation, then industries and themes, then individual stocks. By the time a ticker appears in the watchlist, it has already passed through market, sector, industry, liquidity, relative strength, volume, and context checks.

The final output is not just "top stocks today". It is a classified watchlist: sector leaders, industry leaders, high-relative-volume names, event-context names, and actionability buckets that separate extended leaders from earlier, cleaner, or pullback candidates.

The repo has a few pieces that matter:

- **`src/data/`** - provider adapters for real source data: Alpaca, FRED, Alpha Vantage, SEC EDGAR, universe files, and sector maps.
- **`src/scoring/`** - deterministic market, sector, industry, and stock scoring.
- **`src/actionability.rs`** - labels leaders as extended, actionable, pulling back, compressing, early, or event-watch candidates.
- **`src/storage/`** - SQLite schema and repositories. This is the local system of record.
- **`src/output/`** - Markdown reports and CSV exports.
- **`src/workflows/`** - daily run, backtest, doctor, and status workflows.
- **`src/dashboard/`** and **`dashboard/`** - local API and React dashboard over stored data.
- **`docs/`** - system specs, decisions, validation checkpoints, and implementation notes.

By design, the dashboard is only a reader. It does not fetch new market data and it does not recalculate scores. The Rust workflow is the source of truth.

## Why it exists

A normal screener can tell you what is up today, what has unusual volume, or what crossed a moving average. That is useful, but it is flat. It does not tell you whether the move sits inside a stronger sector, whether the industry group is rotating, whether macro context agrees, or whether the stock has already made the obvious move.

Merryl is built around a different question:

> Where is money moving, why might it be moving, and which names are worth charting from that part of the market?

That makes Merryl a market-rotation tool first and a watchlist tool second.

## Data sources

Merryl uses real sources only. It does not generate fake production candles.

- [Alpaca Market Data](https://docs.alpaca.markets/docs/market-data-faq) - daily OHLCV and recent news.
- [FRED](https://fred.stlouisfed.org/docs/api/fred/) - macro context such as rates, volatility, inflation, employment, credit, and liquidity series.
- [Alpha Vantage](https://www.alphavantage.co/documentation/) - structured earnings calendar context.
- [SEC EDGAR APIs](https://www.sec.gov/search-filings/edgar-application-programming-interfaces) - recent filing events.

The first universe is the S&P 500 with major broad-market and sector ETFs. That is a controlled starting point, not the final product boundary.

## Quick start

**Requirements:** Rust, Node.js/npm, and API keys for Alpaca, FRED, and Alpha Vantage. SEC EDGAR requires a user agent with contact information.

```bash
# 1. Configure local environment
cp .env.example .env

# 2. Fill the required values in .env
ALPACA_API_KEY_ID=
ALPACA_API_SECRET_KEY=
FRED_API_KEY=
ALPHA_VANTAGE_API_KEY=
MERRYL_SEC_USER_AGENT=Merryl/0.1 your-email@example.com

# 3. Install dashboard dependencies
npm --prefix dashboard install

# 4. Build the dashboard
npm --prefix dashboard run build

# 5. Create or migrate the database
cargo run -- db migrate

# 6. Run the daily workflow
set -a; source .env; set +a
cargo run -- run daily --date latest
```

If those commands work, the local setup is ready.

## Running Merryl

The normal daily loop is:

```bash
set -a; source .env; set +a
cargo run -- run daily --date latest
cargo run -- doctor
cargo run -- status
cargo run -- dashboard
```

Each daily run writes:

```text
data/market.db
reports/YYYY-MM-DD_market_report.md
exports/YYYY-MM-DD_sector_scores.csv
exports/YYYY-MM-DD_stock_watchlist.csv
```

Backtests and validation read from SQLite only. They do not fetch new prices or quietly change formulas:

```bash
cargo run -- run backtest --from YYYY-MM-DD --to YYYY-MM-DD
```

## Static dashboard

Merryl can also publish a zero-server dashboard snapshot through GitHub Actions and GitHub Pages. The Action runs the same Rust workflows, builds the React dashboard in static mode, exports dashboard JSON from SQLite, and publishes `dashboard/dist`.

This is not a live hosted Merryl server. GitHub Pages only serves generated files.

Required repository Secrets:

```text
ALPACA_API_KEY_ID
ALPACA_API_SECRET_KEY
FRED_API_KEY
ALPHA_VANTAGE_API_KEY
MERRYL_SEC_USER_AGENT
```

Manual local export:

```bash
npm --prefix dashboard run build
cargo run -- dashboard --export-static dashboard/dist/static-data
```

More detail is in the [static dashboard deployment spec](docs/static_dashboard_deployment_spec.md).

## Design choices

- **Top-down first.** The system starts with regime, sector, and industry context, then narrows into stocks.
- **Local-first, static when hosted.** SQLite remains the local system of record. The hosted path publishes generated dashboard snapshots to GitHub Pages instead of running a stateful cloud server.
- **Real data only.** Missing credentials and source failures should be visible. The daily workflow should not replace real data with dummy candles.
- **Small command surface.** The CLI runs workflows and maintenance checks. Internal ingest and scoring steps are not exposed as a long list of public commands.
- **Context before scoring changes.** Macro data, news, earnings, filings, and actionability labels are stored and validated before they are allowed to change formulas.
- **Watchlist, not advice.** Merryl helps decide what deserves review. It does not execute trades, size positions, manage risk, or replace chart review.

## Main docs

- [Market rotation system spec](docs/market_rotation_system_spec.md)
- [MVP technical plan](docs/mvp_technical_plan_spec.md)
- [Phase 0 decisions](docs/phase_0_decisions_spec.md)
- [Implementation runbook](docs/implementation_spec.md)
- [Static dashboard deployment](docs/static_dashboard_deployment_spec.md)
- [Phase 5 data-source expansion](docs/phase_5_data_source_expansion_spec.md)
- [Structured catalyst source spec](docs/phase_5c_structured_catalyst_source_spec.md)
- [Watchlist convergence review](docs/watchlist_convergence_review_spec.md)
- [Watchlist actionability filter](docs/watchlist_actionability_extension_filter_spec.md)

## Boundary

Merryl is a decision-support tool for market review. It does not provide financial advice, execute trades, manage positions, or replace chart/risk review.
