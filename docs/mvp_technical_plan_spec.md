# MVP Technical Plan

Version: 0.2
Date: 2026-05-27
Status: MVP technical plan; Rust core is implemented through PDB-6 and the first Phase 4 dashboard/API slice
Source documents:

- `market_rotation_system_spec.md`
- `phase_0_decisions_spec.md`

## 1. Purpose

This document translates the market rotation system specification and Phase 0 decisions into an implementation plan.

The goal is to build the first controlled version of the system without losing the original intent:

```text
Show where market participation is concentrating,
map that concentration from market regime to sector to industry/theme to stock,
then produce a small list of liquid, chart-worthy names with explainable reasons.
```

This is not a generic stock scanner. It is a market rotation intelligence system.

## 2. MVP Definition

The MVP should produce:

```text
Daily market map -> sector ranking -> stock ranking -> watchlist report
```

The first build starts with daily US equity data, but the architecture must remain expandable to:

- broader stock universes
- intraday/live data
- ETF fund flows
- options flow
- gamma exposure
- dark pool data
- macro series
- news/catalyst enrichment
- dashboard UI

## 2.1 Source Traceability And Preservation

This document does not replace `market_rotation_system_spec.md` or `phase_0_decisions_spec.md`. It translates them into a build plan.

The technical plan must preserve the complete original flow:

```text
Research idea
  -> Market structure model
    -> Phase 0 decisions
      -> MVP build plan
        -> Later advanced layers
```

| Source area | Original intent | Technical plan preservation |
|---|---|---|
| Central point | Show where participation is concentrating and produce chart-worthy names | Preserved in Purpose and MVP Definition |
| Top-down flow | Market regime -> sector -> industry/theme -> stock -> chart timing | Preserved in scoring modules, reports, phases, and dashboard plan |
| What is moving? | Prices across indices, sectors, industries, stocks, and intermarket assets | Preserved through symbols, prices_daily, broad ETFs, sector ETFs, and expandable asset types |
| Who is moving it? | Big money, smart money, market makers, retail, inferred through proxies | Preserved indirectly through volume, breadth, relative strength, events, and later advanced layers |
| Why is it moving? | Macro, sector fundamentals, company catalysts, positioning | Preserved through market regime, macro_series, events, catalyst fields, and later positioning/flow layers |
| How is it moving? | Orders, volume, relative volume, breadth, persistence | Preserved through indicators, volume metrics, breadth, trend, and score components |
| Where is it moving? | Asset class -> index -> sector -> industry/theme -> stock | Preserved through universe config, sector_map, industry_map, sector_scores, industry_scores, stock_scores |
| When is it moving? | 1D, 5D, 20D, 60D, 120D/252D context | Preserved through return windows, historical scores, and backtest labels |
| Daily use case | Market prep and chart-worthy names | Preserved through `run daily`, daily report, and watchlist CSV |
| Weekly use case | Review 5D/20D/60D rotation and prepare next-week watchlist | Preserved through historical scores and weekly review mode planned after initial reports |
| Long-term use case | Study sector behavior through macro cycles and improve by evidence | Preserved through backtesting, historical storage, and macro/event schema |
| Market regime | Risk-on, risk-off, defensive, mixed context | Preserved in Market Regime V1 |
| Sector rotation | Rank sectors by strength, volume, breadth, rank change | Preserved in Sector Flow Score and Phase 1 |
| Industry/theme strength | Identify active groups inside sectors | Preserved in industry_map, industry_scores, and Phase 2 |
| Stock leadership | Rank liquid leaders inside active sectors/industries | Preserved in Stock Opportunity Score and Phase 2 |
| Catalyst awareness | Keep asking why a move is happening | Preserved through events and catalyst_status, even if first values are pending_source |
| Money flow as proxy | Avoid pretending flow is perfectly visible | Preserved through explainable proxies, not direct claims of hidden money movement |
| Filings and ownership | SEC filings, insider activity, 13F/institutional context for longer-term research | Deferred from v1, preserved as a later filings/ownership layer |
| Advanced participation | ETF flows, options, gamma, dark pools, COT, intraday, filings/ownership | Deferred as dependencies but preserved in schema and Phase 5 |
| Backtesting | Validate score usefulness before trusting | Preserved in Phase 3 and backtest_results |
| Risk rule | Watchlist, not automatic trade signal | Preserved as a non-negotiable rule in this plan |
| Technical choices | S&P 500/daily/SQLite/Markdown+CSV are first slices, not final boundaries | Preserved through expandable universe, provider adapters, and intraday-ready schema |

Any implementation task that weakens this mapping should be revised before code is written.

## 2.2 Review Modes To Preserve

The product must support three review modes over time:

```text
Daily: what is moving today and what should be charted now?
Weekly: what sectors/industries are gaining or losing strength over 5D/20D/60D?
Long-term: how does sector behavior relate to macro regime and score history?
```

V1 can start with the daily report, but the database and report structure must preserve weekly and long-term review.

Planned review/report modes:

```text
daily
weekly
long-term
```

These are product modes, not necessarily public CLI commands. Only `daily` must work in the first milestone. `weekly` and `long-term` can be added after historical scores exist, ideally through the dashboard/reporting layer rather than extra CLI commands.

## 3. Stack Decision

### 3.1 Recommended Core Stack

Use Rust as the core implementation language.

Important clarification:

```text
Rust is not being chosen only because the first build is small.
Rust is being chosen because it can support the long-term system.
```

The instruction to keep the first build controlled is about product scope, not Rust capability. Rust can handle the core system as it grows from daily reports into a larger data pipeline, backtesting engine, API backend, and dashboard backend.

Recommended stack:

```text
Language: Rust
Storage: SQLite
Output: Markdown + CSV
Primary UX target: local web dashboard
CLI role: minimal workflow runner and maintenance surface
Data source: adapter-based provider interface
First provider: Alpaca Market Data API daily OHLCV
Production provider later: higher Alpaca feed or Polygon if needed
```

Rust is a good fit because the first real product is a durable data pipeline and scoring engine:

- ingest market data
- store time series
- calculate deterministic scores
- generate reports
- keep results reproducible
- run daily without fragile notebook state

### 3.2 MATLAB Or Octave

MATLAB/Octave are not needed for this build.

They are useful for numerical experiments, matrix-heavy research, or academic-style prototyping. But this system is primarily:

- data ingestion
- database storage
- ranking
- scoring
- reporting
- backtesting
- eventually API/dashboard work

Rust plus SQLite is a better foundation for that. If we later need exploratory research, Python notebooks would be more practical than MATLAB/Octave because market-data and finance tooling is broader.

Decision:

```text
Do not use MATLAB or Octave for the MVP.
Use Rust for the core system.
Optionally use Python later only for exploratory notebooks or visualization experiments.
```

### 3.3 Rust Tradeoff

Rust is strong for reliability and speed, but slower than Python for quick data science iteration.

To avoid getting stuck:

- Keep v1 formulas simple.
- Store score components so results are explainable.
- Use CSV/Markdown outputs first.
- Avoid complex ML or advanced quant libraries in v1.
- Keep provider/data/scoring/report modules separated.

This is not because Rust cannot handle the advanced version. It is because the system should prove the core market-map idea before we add every advanced data layer.

Scope clarification:

```text
Controlled first slice = build only the features needed to prove the core model.
Rust core = long-term implementation choice for the system.
```

If a future subsystem is better handled by another tool, we can add it without replacing Rust:

```text
Rust: core engine, ingestion, scoring, backtesting, CLI, backend
Python: optional exploratory notebooks or one-off research
TypeScript/JavaScript: dashboard frontend later
```

## 4. System Shape

The implementation should be engine-first, with a local dashboard as the primary product interface target.

The CLI should exist, but only as a small automation and maintenance surface.

```text
Primary user experience: local dashboard for market review.
CLI purpose: run workflows, check status, migrate database, support automation.
```

This is the safer direction for this product because the core workflow is visual:

- compare sectors
- compare industries/themes
- inspect ranked stocks
- read explanations
- review historical scores
- eventually inspect charts and flow context

A CLI is useful for automation, but it is a poor primary interface for visual market intelligence.

## 4.0 Interface Decision

Recommended product architecture:

```text
Rust core engine
  -> SQLite local database
  -> local API/service boundary
  -> local dashboard for review
  -> tiny CLI for automation/maintenance
  -> Markdown/CSV exports for audit/share
```

This avoids building a large CLI that later has to be removed. The CLI remains useful long term, but it does not own the user experience.

Decision:

```text
Build the Rust core so it can serve both CLI and dashboard.
Design the dashboard as the main product surface.
Keep CLI small from day one.
```

The first milestone may still generate Markdown/CSV before the dashboard exists, but that is a validation output, not the final user interface.

```text
merryl
```

The CLI should expose user workflows, not internal modules.

Bad direction:

```text
merryl ingest
merryl score-sectors
merryl score-stocks
merryl calc-breadth
merryl calc-relative-volume
merryl write-sector-csv
merryl write-watchlist-csv
```

That becomes a command graveyard.

Better direction:

```text
merryl run daily --date latest
merryl run weekly --date latest
merryl run backtest --from 2020-01-01 --to 2026-05-26
merryl status
merryl doctor
merryl db migrate
```

Internal steps still exist, but they are called by the workflow engine:

```text
daily workflow:
  ingest daily prices
  calculate market regime
  score sectors
  score industries
  score stocks
  write reports and exports
```

The CLI should behave like a control surface, not like the architecture leaked into the terminal.

## 4.1 CLI Governance

To avoid command bloat, the public CLI should follow these rules:

- Expose workflows, not implementation steps.
- Keep top-level commands small: `run`, `status`, `doctor`, `db`.
- Add `report` only if re-rendering reports from stored data becomes a recurring maintenance workflow.
- Prefer config files for workflow details instead of adding many flags.
- Add a public command only when a real user workflow needs it.
- Do not add commands just because a Rust module exists.
- Keep internal pipeline steps testable through code/tests, not necessarily public CLI commands.
- Support `--help`, concise examples, stable exit codes, and machine-readable `--json` output where useful.
- Keep destructive or network-heavy actions explicit.
- Print what changed when a command changes state.

Command admission test:

```text
Before adding a command, answer yes to all:

1. Is this a user workflow rather than an internal step?
2. Will the command still make sense six months from now?
3. Can it be documented in one sentence?
4. Can it avoid more than a small set of flags?
5. Would config/workflow TOML be a cleaner fit?
```

If the answer to #5 is yes, do not add a command. Add or update a workflow config instead.

## 4.2 Workflow Configuration

Workflow complexity should live in config, not in an ever-growing CLI.

Example:

```text
config/workflows/daily.toml
config/workflows/weekly.toml
config/workflows/backtest.toml
```

The public command stays simple:

```text
merryl run daily --date latest
```

The workflow file defines the steps:

```text
name = "daily"
steps = [
  "ingest_daily_prices",
  "score_market_regime",
  "score_sectors",
  "score_industries",
  "score_stocks",
  "write_reports"
]
```

This lets the system grow without turning the CLI into a long list of commands.

## 4.3 CLI Boundary And Alternatives

The CLI should never become the primary exploration interface.

Warning signs:

- The command list keeps growing.
- Users need to remember too many flags.
- Most usage becomes visual inspection rather than automation.
- The main workflow is comparing sectors, stocks, charts, and explanations side by side.
- Reports are useful, but navigating them from files becomes slow.
- Adding a new feature requires adding another public command.
- The CLI starts exposing internal implementation steps.

If these appear, do not expand the CLI. Move that workflow into the dashboard/API layer.

Interface options and roles:

| Option | Best For | Tradeoff |
|---|---|---|
| Local web dashboard | Primary product interface: visual market map, sector tables, watchlists, filters, review workflow | More frontend work |
| Rust API backend + frontend | Long-term architecture with CLI as optional automation layer | Requires API design |
| Tauri desktop app | Later packaging path for local desktop app with Rust core and web UI | More packaging complexity |
| Scheduled job runner | Daily/weekly automatic reports with minimal manual commands | Less interactive |
| Notebook/research layer | Exploratory analysis and formula experiments | Not ideal as production core |

Decision rule:

```text
CLI is acceptable for automation and maintenance.
Dashboard is the primary interface for visual market exploration.
Keep the Rust core either way.
```

Possible long-term shape:

```text
Rust core engine
  -> CLI for automation and scheduled runs
  -> local API for app/dashboard access
  -> dashboard for daily market review
```

## 5. Folder Structure

Recommended structure:

```text
Merryl/
  market_rotation_system_spec.md
  phase_0_decisions_spec.md
  mvp_technical_plan.md
  Cargo.toml
  README.md
  .env.example
  config/
    app.toml
    sector_etfs.toml
    universe_sp500.toml
    workflows/
      daily.toml
      weekly.toml
      backtest.toml
  data/
    market.db
  exports/
    YYYY-MM-DD_sector_scores.csv
    YYYY-MM-DD_stock_watchlist.csv
  reports/
    YYYY-MM-DD_market_report.md
  src/
    main.rs
    config.rs
    db/
      mod.rs
      migrations.rs
      schema.rs
    providers/
      mod.rs
      daily_ohlcv.rs
      provider_trait.rs
      mock_provider.rs
      polygon_provider.rs
      alpaca_provider.rs
    universe/
      mod.rs
      symbols.rs
      sectors.rs
    indicators/
      mod.rs
      returns.rs
      moving_average.rs
      volume.rs
      breadth.rs
    scoring/
      mod.rs
      market_regime.rs
      sectors.rs
      industries.rs
      stocks.rs
    reports/
      mod.rs
      markdown.rs
      csv.rs
    api/
      mod.rs
      routes.rs
      view_models.rs
    backtest/
      mod.rs
      labels.rs
      deciles.rs
    errors.rs
```

For the first implementation, not every file must exist immediately. The structure shows where the system is going.

The structure should preserve room for later modules:

```text
events/
filings/
intraday/
flows/
options/
```

These do not need to exist in the first Rust scaffold, but the database and module boundaries should not block them.

## 6. Database Design

Use SQLite first.

The database must support the MVP while leaving room for advanced layers.

### 6.1 `symbols`

Purpose:

Store stocks, ETFs, indices, and later other assets.

Fields:

```text
symbol TEXT PRIMARY KEY
name TEXT
asset_type TEXT
sector TEXT
industry TEXT
exchange TEXT
market_cap REAL
is_active INTEGER
first_seen_date TEXT
last_seen_date TEXT
```

### 6.2 `prices_daily`

Purpose:

Store adjusted daily OHLCV.

Fields:

```text
symbol TEXT
date TEXT
open REAL
high REAL
low REAL
close REAL
adjusted_close REAL
volume REAL
source TEXT
PRIMARY KEY(symbol, date)
```

### 6.2.1 `prices_intraday`

Purpose:

Reserve room for intraday/live data after the daily system is proven.

V1 does not need to populate this table, but the schema direction should be clear.

Fields:

```text
symbol TEXT
timestamp TEXT
timeframe TEXT
open REAL
high REAL
low REAL
close REAL
volume REAL
vwap REAL
source TEXT
PRIMARY KEY(symbol, timestamp, timeframe)
```

### 6.3 `sector_map`

Purpose:

Map sectors to their ETF proxies.

Fields:

```text
sector TEXT PRIMARY KEY
sector_etf TEXT
description TEXT
```

Examples:

```text
Technology -> XLK
Financials -> XLF
Health Care -> XLV
Energy -> XLE
```

### 6.4 `industry_map`

Purpose:

Store industry/theme grouping for stocks.

Fields:

```text
industry TEXT
sector TEXT
description TEXT
PRIMARY KEY(industry, sector)
```

### 6.5 `macro_series`

Purpose:

Reserve room for macro data.

V1 may leave this sparse, but the schema should exist.

Fields:

```text
series_id TEXT
date TEXT
value REAL
source TEXT
PRIMARY KEY(series_id, date)
```

### 6.6 `events`

Purpose:

Reserve room for earnings, news, macro events, and catalysts.

V1 can mark catalyst fields as unknown until a source is added.

Fields:

```text
event_id TEXT PRIMARY KEY
symbol TEXT
event_date TEXT
event_type TEXT
title TEXT
source TEXT
importance INTEGER
raw_url TEXT
```

### 6.6.1 `filings`

Purpose:

Reserve room for SEC filings, insider transactions, 13F/institutional ownership, and other long-term participation context.

V1 does not need to populate this table.

Fields:

```text
filing_id TEXT PRIMARY KEY
symbol TEXT
filing_date TEXT
filing_type TEXT
source TEXT
title TEXT
raw_url TEXT
summary TEXT
```

### 6.7 `sector_scores`

Purpose:

Store explainable sector scores.

Fields:

```text
sector TEXT
date TEXT
score REAL
rank INTEGER
relative_return_component REAL
trend_component REAL
relative_volume_component REAL
breadth_component REAL
rank_improvement_component REAL
components_json TEXT
PRIMARY KEY(sector, date)
```

### 6.8 `industry_scores`

Purpose:

Store industry/theme scores.

Fields:

```text
industry TEXT
sector TEXT
date TEXT
score REAL
rank INTEGER
components_json TEXT
PRIMARY KEY(industry, sector, date)
```

### 6.9 `stock_scores`

Purpose:

Store individual stock opportunity scores.

Fields:

```text
symbol TEXT
date TEXT
score REAL
rank INTEGER
sector TEXT
industry TEXT
sector_score REAL
industry_score REAL
relative_strength_component REAL
relative_volume_component REAL
trend_component REAL
liquidity_component REAL
catalyst_status TEXT
components_json TEXT
PRIMARY KEY(symbol, date)
```

### 6.10 `watchlists`

Purpose:

Store generated watchlists.

Fields:

```text
date TEXT
symbol TEXT
rank INTEGER
score REAL
reason TEXT
PRIMARY KEY(date, symbol)
```

### 6.11 `backtest_results`

Purpose:

Store score validation results.

Fields:

```text
run_id TEXT
date TEXT
entity_type TEXT
entity_id TEXT
score REAL
rank INTEGER
forward_1d_return REAL
forward_5d_return REAL
forward_10d_return REAL
forward_20d_return REAL
forward_60d_return REAL
forward_return_vs_spy REAL
forward_return_vs_sector REAL
PRIMARY KEY(run_id, date, entity_type, entity_id)
```

## 7. Data Provider Design

Use a provider interface so the system is not locked to one source.

Conceptual interface:

```text
DailyOhlcvProvider
  fetch_symbols()
  fetch_daily_prices(symbol, start, end)
  fetch_latest_daily_prices(symbols)
```

Initial providers:

- `alpaca_provider`: first real daily OHLCV provider.
- `polygon_provider`: production candidate if Polygon becomes a better fit.
- test fixtures: allowed only inside tests; not a fallback for daily runs.

Provider rule:

```text
Scoring code must not know which provider produced the data.
```

This keeps the system replaceable and prevents early data choices from becoming permanent architecture.

## 8. Universe Configuration

V1 starts with an S&P 500 anchor universe.

But the system must support future universe expansion.

Config should allow:

```text
universe = "sp500"
include_sector_etfs = true
include_broad_etfs = true
min_price = 5.00
min_avg_dollar_volume = 20000000
```

Future universes:

- Russell 1000.
- Russell 3000.
- all US stocks above liquidity threshold.
- custom watchlists.
- theme baskets.

## 9. Market Regime V1

V1 regime should be lightweight and explainable.

Inputs:

- SPY trend.
- QQQ vs SPY.
- IWM vs SPY.
- VIX trend if available.
- TLT trend if available.
- US 10Y yield if available.
- DXY if available.
- GLD trend later if available.
- Oil/energy proxy later if available.

Output:

```text
Risk-on
Risk-off
Defensive
Mixed
```

Rule:

```text
Market regime gives context. It does not block sector or stock rankings.
```

## 10. Scoring V1

All scores must be explainable and stored with components.

### 10.1 Sector Flow Score

V1 formula:

```text
Sector Flow Score =
  30% relative return vs SPY
  20% 20-day trend strength
  20% relative volume
  20% breadth inside sector
  10% rank improvement
```

Required metrics:

- 1D, 5D, 20D, 60D returns.
- Sector return minus SPY return.
- Sector ETF volume vs 20-day average.
- Percent of sector constituents above 20-day moving average.
- Percent of sector constituents above 50-day moving average.
- Rank change from prior period.

### 10.2 Industry Flow Score

Industry scoring can be simpler in the first implementation if industry mappings are incomplete.

V1 formula:

```text
Industry Flow Score =
  30% return vs sector
  20% return vs SPY
  20% breadth inside industry
  15% relative volume
  15% stocks making highs/breakouts
```

If industry data is sparse, Phase 2 can initially rank by sector-only stock leadership while preserving the industry schema.

### 10.3 Stock Opportunity Score

V1 formula:

```text
Stock Opportunity Score =
  30% sector flow score
  25% stock relative strength vs sector
  20% stock relative volume
  15% trend structure
  10% liquidity quality
```

Stock filters:

- Minimum price.
- Minimum average dollar volume.
- Active symbol only.
- Valid sector mapping.
- Valid daily data.

### 10.4 Catalyst Fields

V1 does not need a full news engine, but it should preserve the catalyst concept.

Possible values:

```text
known
unknown
earnings_soon
macro_sensitive
news_flagged
pending_source
```

First implementation can default to:

```text
pending_source
```

This keeps the "why is it moving?" question alive without blocking the MVP.

Current implementation update:

```text
recent_news:N
earnings:YYYY-MM-DD
filing:FORM
```

These labels are used when source-backed event context exists for a current watchlist symbol:

- Alpaca News for recent headlines.
- Alpha Vantage Earnings Calendar for upcoming earnings dates.
- SEC EDGAR submissions for recent 8-K, 10-Q, and 10-K filing events.

Event context remains explanation/review context only. It is not a stock score input.

## 11. Reports

The first report should be Markdown plus CSV exports.

### 11.1 Markdown Report

Path:

```text
reports/YYYY-MM-DD_market_report.md
```

Required sections:

```text
# Market Report: YYYY-MM-DD

## Market Regime
## Top Sectors
## Weak Sectors
## Sector Rank Changes
## Top Industries Or Themes
## Top Stocks Worth Charting
## New Leaders
## High Relative Volume Names
## Catalyst / Event Flags
## Notes For Chart Review
```

### 11.2 Sector CSV

Path:

```text
exports/YYYY-MM-DD_sector_scores.csv
```

Fields:

```text
date
sector
sector_etf
score
rank
return_1d
return_5d
return_20d
return_60d
relative_return_vs_spy
relative_volume
breadth_20d
breadth_50d
rank_change
explanation
```

### 11.3 Watchlist CSV

Path:

```text
exports/YYYY-MM-DD_stock_watchlist.csv
```

Fields:

```text
date
rank
symbol
name
sector
industry
score
sector_score
return_1d
return_5d
return_20d
return_60d
relative_return_vs_sector
relative_return_vs_spy
relative_volume
avg_dollar_volume
trend_state
catalyst_status
explanation
```

## 12. CLI Commands

Public CLI target surface:

```text
merryl run daily --date YYYY-MM-DD
merryl run weekly --date YYYY-MM-DD
merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD
merryl status
merryl doctor
merryl db migrate
```

This is the target command surface, implemented incrementally.

First milestone:

```text
merryl run daily --date latest
```

Weekly and backtest workflows should only become usable after historical scores and validation data exist.

Internal pipeline steps, not public commands:

```text
ingest_daily_prices
score_market_regime
score_sectors
score_industries
score_stocks
write_reports
write_exports
```

These steps should be testable in Rust, but they should not automatically become CLI commands.

## 13. Phase 1 Implementation Tasks

Phase 1: Data Foundation.

Goal:

```text
We can rank sectors historically and today.
```

Tasks:

1. Create Rust project.
2. Add config loading.
3. Create SQLite database.
4. Add database migrations.
5. Add symbol table.
6. Add sector ETF mapping.
7. Add daily price table.
8. Add provider trait.
9. Add first data provider.
10. Load broad ETFs.
11. Load sector ETFs.
12. Load S&P 500 anchor universe.
13. Calculate returns.
14. Calculate relative returns vs SPY.
15. Calculate relative volume.
16. Calculate sector breadth.
17. Calculate sector flow score.
18. Store score components.
19. Generate sector CSV.
20. Generate basic market report.

Phase 1 acceptance:

- Daily OHLCV loads.
- Data stores in SQLite.
- Sector ETF prices are available.
- S&P 500 anchor symbols are available.
- Sector rankings are generated.
- Each sector score is explainable.
- Schema leaves room for macro, events, intraday, and advanced confirmation layers.

## 14. Phase 2 Implementation Tasks

Phase 2: Stock Ranking.

Goal:

```text
We get a useful list of stocks to chart every day.
```

Tasks:

1. Calculate stock returns.
2. Calculate stock relative strength vs sector ETF.
3. Calculate stock relative strength vs SPY.
4. Calculate stock relative volume.
5. Calculate average dollar volume.
6. Apply liquidity filters.
7. Calculate trend state.
8. Add industry/theme grouping where available.
9. Calculate stock opportunity score.
10. Store stock score components.
11. Generate top 20-50 stock watchlist.
12. Generate watchlist CSV.
13. Add watchlist section to Markdown report.
14. Carry catalyst fields as `pending_source` if no source exists yet, or `recent_news:N` when recent news is connected.

Phase 2 acceptance:

- Top stock watchlist is generated.
- Stocks are ranked inside market/sector context.
- Each stock has a reason.
- Illiquid stocks are filtered.
- Output is usable for chart review.

## 15. Phase 3 Implementation Tasks

Phase 3: Backtesting.

Goal:

```text
We know whether the scoring method has useful forward behavior.
```

Tasks:

1. Generate historical daily scores.
2. Calculate forward 1D, 5D, 10D, 20D, 60D returns.
3. Calculate forward returns vs SPY.
4. Calculate forward returns vs sector ETF.
5. Group scores into deciles.
6. Compare high-score vs low-score sectors.
7. Compare high-score stocks vs sector.
8. Export backtest summaries.
9. Store results in `backtest_results`.

Acceptance:

- We can see whether high-ranked sectors outperform low-ranked sectors.
- We can see whether high-ranked stocks outperform their sector.
- We can identify false positives.
- We can decide whether scoring weights need revision.

## 16. Phase 4 Implementation Tasks

Phase 4: Dashboard.

The dashboard is the primary product interface target.

Do not overbuild it before the scoring/report workflow is proven, but design the core so the dashboard does not require rewriting the engine.

Dashboard pages:

- Market regime.
- Sector rotation.
- Industry/theme strength.
- Stock leadership.
- Watchlist.
- Historical score/backtest review.

Likely stack direction:

```text
Rust core/backend + lightweight web frontend
```

The exact frontend stack should be chosen later, after the report workflow proves useful. Tauri can be evaluated later if the local web dashboard should become a packaged desktop app.

## 17. Phase 5 Advanced Layers

Add only after the MVP is working:

- ETF fund flows.
- Options activity.
- Gamma exposure.
- Dark pool prints.
- News/catalyst feed.
- Intraday data.
- Alerts.
- COT/futures positioning.
- SEC filings.
- Insider activity.
- 13F/institutional ownership.

Rule:

```text
Advanced layers must improve explainability or ranking quality.
They should not replace the core rotation map.
```

## 18. Testing Plan

Start with practical tests:

- Can the database initialize?
- Can symbols be inserted and updated?
- Can prices be inserted without duplicates?
- Are returns calculated correctly?
- Are sector mappings correct?
- Are missing prices handled safely?
- Are score components stored?
- Does report generation work with partial data?
- Does backtest avoid using future data?

Later:

- Provider integration tests.
- Snapshot tests for reports.
- Backtest regression tests.
- Performance tests for larger universes.

## 19. Risk Controls

Implementation risks:

- Bad or incomplete data.
- Provider lock-in.
- Survivorship bias.
- Lookahead bias.
- Overcomplicated scoring.
- Treating rankings as trade signals.
- Building UI before proving usefulness.

Controls:

- Store raw components.
- Keep provider interface abstract.
- Use explicit dates in all calculations.
- Do not use future values in scoring.
- Generate reports before dashboards.
- Keep scoring explainable.
- Backtest before trusting.

Non-negotiable product rule:

```text
The system creates attention, not trades.
No trade should be taken only because a stock is highly ranked.
The chart, risk plan, and invalidation decide the trade.
```

## 20. First Build Milestone

The first milestone is complete when this command works:

```text
merryl run daily --date latest
```

And produces:

```text
reports/YYYY-MM-DD_market_report.md
exports/YYYY-MM-DD_sector_scores.csv
exports/YYYY-MM-DD_stock_watchlist.csv
data/market.db
```

The report must include:

- Market regime summary.
- Ranked sectors.
- Weak sectors.
- Rank changes.
- Top stocks worth charting.
- Explanations.
- Clear wording that the output is a watchlist, not a trade signal.

## 21. Final Technical Decision

For the MVP:

```text
Use Rust.
Use SQLite.
Use Markdown + CSV reports.
Use local dashboard as the primary product interface target.
Use a small workflow-oriented CLI only for automation and maintenance.
Use adapter-based market data providers.
Do not use MATLAB or Octave.
Do not build the full dashboard before the engine/report workflow proves useful.
Do not require options/dark pool/gamma data in v1.
Keep the architecture open for advanced participation data later.
Preserve daily, weekly, and long-term review modes.
Treat all rankings as watchlist/attention outputs, not trade signals.
Do not expose every internal pipeline step as a public command.
```

This matches the original system direction while keeping the first build achievable.

## 22. CLI Design References

The CLI design should be informed by:

- Command Line Interface Guidelines: https://clig.dev/
- Microsoft command-line design guidance: https://learn.microsoft.com/en-us/dotnet/standard/commandline/design-guidance
- Rust `clap` documentation: https://docs.rs/clap/latest/clap/
- Twelve-Factor config guidance: https://www.12factor.net/config
