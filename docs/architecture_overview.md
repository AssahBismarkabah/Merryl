# Architecture Overview

## Philosophy

Most stock tools hand you a pile of tickers. Merryl starts one level higher: where is participation concentrating, which parts of the market are leading, and which stocks are actually worth charting from there?

The system builds a local market map that keeps you from randomly scanning names. It pulls real market and context data, scores the market from the top down, stores the history in SQLite, writes a daily report, and serves a small read-only dashboard. You still do chart review elsewhere. Merryl just helps decide what deserves attention first.

## Top-Down Flow

Merryl follows the order a serious market review should follow:

```
1. Broad Market Regime  -->  risk-on / risk-off / neutral
2. Sector Rotation      -->  rank 11 sectors by composite score
3. Industries / Themes  -->  rank industry groups within sectors
4. Individual Stocks    -->  score and rank S&P 500 constituents
5. Actionability        -->  classify leaders: extended, actionable, pullback, etc.
6. Context              -->  overlay news, earnings, filings, macro data
```

By the time a ticker appears in the watchlist, it has already passed through market, sector, industry, liquidity, relative strength, volume, and context checks.

## System Components

### `src/data/` -- Provider Adapters

Adapters for external data sources. Each adapter handles authentication, rate limiting, request formatting, and response parsing.

| Module | Source | Data |
|---|---|---|
| `alpaca.rs` | Alpaca Market Data | Daily OHLCV bars, recent news |
| `fred.rs` | FRED API | Macro series (rates, VIX, CPI, employment, etc.) |
| `alpha_vantage.rs` | Alpha Vantage | Earnings calendar |
| `sec_edgar.rs` | SEC EDGAR | Recent filings (8-K, 10-Q, 10-K) |
| `sector_map.rs` | Local config | Sector-to-ETF mappings |
| `universe.rs` | Wikipedia / local | S&P 500 constituent list |

### `src/scoring/` -- Scoring Engine

Deterministic scoring modules. No machine learning -- all scores are computed from price and volume data using fixed formulas.

| Module | Purpose |
|---|---|
| `market.rs` | Broad market regime score (SPY, QQQ, IWM, DIA) |
| `regime.rs` | Risk-on/risk-off classification with macro overlay |
| `sectors.rs` | Sector composite scores (relative return, trend, volume, breadth) |
| `industries.rs` | Industry group scores within sectors |
| `stocks.rs` | Individual stock scores (sector context, relative strength, trend, volume, liquidity) |
| `catalysts.rs` | Event context flags (news, earnings, filings) |
| `indicators.rs` | Technical indicator calculations |
| `history.rs` | Historical score tracking and rank changes |
| `explanations.rs` | Score explanation text generation |

### `src/actionability.rs` -- Actionability Classification

Labels watchlist stocks into review-priority buckets:

| Label | Meaning |
|---|---|
| `extended_leader` | Already made a large move, extended from moving averages |
| `actionable_leader` | Strong score, reasonable setup, not extended |
| `pullback_leader` | Pulled back from a high, potential re-entry |
| `base_compression_candidate` | Tight range, potential breakout setup |
| `early_rotation_candidate` | Early stage rotation, low 5-day return |
| `event_watch_unconfirmed` | Has catalyst context but unconfirmed price action |
| `unclassified_leader` | Strong score but no clear actionability signal |

### `src/classification.rs` -- Watchlist Classification

Multi-label classifier that tags each stock with context labels:

- `sector_leader` -- stock in a top-3 ranked sector
- `industry_leader` -- stock in a top-10 ranked industry
- `relative_strength_leader` -- positive relative strength vs SPY
- `volume_confirmed` -- above-average relative volume
- `new_leader` -- newly entered the watchlist
- `event_context` -- has news, earnings, or filing context
- `event_risk` -- upcoming event that could move the stock
- `macro_conflict_context` -- macro regime conflicts with sector/stock score

### `src/storage/` -- SQLite Persistence

| Module | Purpose |
|---|---|
| `schema.rs` | Table definitions and migrations |
| `sqlite.rs` | Database connection and transaction management |
| `write_repository.rs` | Write operations (insert scores, prices, events) |
| `read_repository.rs` | Read operations (query scores, history, exports) |
| `quality.rs` | Data quality checks and gap detection |

### `src/output/` -- Reports and Exports

| Module | Purpose |
|---|---|
| `markdown.rs` | Daily market report generation |
| `csv.rs` | CSV export generation |
| `reports.rs` | Report orchestration |
| `backtest.rs` | Backtest report formatting |
| `intraday.rs` | Intraday readiness report formatting |
| `actionability_validation.rs` | Actionability validation reports |
| `event_context_validation.rs` | Event context validation reports |
| `macro_regime_validation.rs` | Macro regime validation reports |
| `formatting.rs` | Shared formatting utilities |
| `paths.rs` | Output path generation |

### `src/workflows/` -- Orchestration

| Module | Purpose |
|---|---|
| `daily.rs` | Daily market rotation workflow |
| `backtest.rs` | Backtest workflow |
| `intraday.rs` | Intraday execution readiness workflow |
| `doctor.rs` | Diagnostic checks |
| `health.rs` | Status reporting |
| `date_args.rs` | Date argument resolution |
| `messages.rs` | Workflow message types |

### `src/dashboard/` -- Local API Server

| Module | Purpose |
|---|---|
| `server.rs` | Axum HTTP server |
| `repository.rs` | Dashboard-specific database queries |
| `models.rs` | Dashboard response types |

### `dashboard/` -- React Frontend

React + Vite + TypeScript dashboard with:

- Market regime overview
- Sector rotation heatmap
- Industry group rankings
- Watchlist with actionability buckets
- Catalyst/event context display
- Historical score explorer

## Design Choices

- **Top-down first.** The system starts with regime, sector, and industry context, then narrows into stocks.
- **Local-first, static when hosted.** SQLite remains the local system of record. The hosted path publishes generated dashboard snapshots to GitHub Pages instead of running a stateful cloud server.
- **Real data only.** Missing credentials and source failures should be visible. The daily workflow should not replace real data with dummy candles.
- **Small command surface.** The CLI runs workflows and maintenance checks. Internal ingest and scoring steps are not exposed as a long list of public commands.
- **Context before scoring changes.** Macro data, news, earnings, filings, and actionability labels are stored and validated before they are allowed to change formulas.
- **Watchlist, not advice.** Merryl helps decide what deserves review. It does not execute trades, size positions, manage risk, or replace chart review.


The daily workflow is the single source of truth. The dashboard is always a reader -- it never fetches data or recalculates scores.
