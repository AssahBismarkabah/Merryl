# Merryl Implementation Runbook

Version: 0.3
Date: 2026-05-28
Status: Daily scoring, Phase 3 backtesting, pre-dashboard stability, Phase 4 dashboard/API stabilization, Phase 5A/B FRED macro ingestion and macro/regime validation, Phase 5C structured catalyst/event context, and Phase 5C event-context validation

## Current Slice

The current implementation target is:

```text
Merryl daily run
  -> load S&P 500 anchor universe
  -> fetch real daily OHLCV data
  -> fetch real FRED macro series context
  -> score the valid historical market window
  -> fetch recent Alpaca News, Alpha Vantage earnings calendar, and SEC EDGAR filing events for the current watchlist
  -> store macro observations, event context, market regime, sector, industry, stock, and watchlist rows in SQLite
  -> explain Market Regime V1 with broad equity ETFs plus TLT, GLD, and USO context
  -> show as-of FRED macro flags as a non-scoring context overlay beside Market Regime V1
  -> explain industry/theme strength with return, relative return, volume, breadth, and 20D-high components
  -> label catalyst/event context as recent news, earnings date, filing event, or pending source
  -> write Markdown and CSV outputs for the latest/requested score date

Merryl backtest run
  -> read historical scores and daily prices from SQLite
  -> calculate forward 1D, 5D, 10D, 20D, and 60D returns from future trading bars
  -> group daily sector and stock scores into deciles
  -> group sector forward behavior by same-day sector component decile
  -> group stock forward behavior by same-day industry/theme score decile
  -> validate ETF-proxy regime labels against stored FRED macro context using as-of macro snapshots
  -> validate stored event/catalyst watchlist labels against forward stock behavior
  -> summarize hit rate, average return, median return, and relative return behavior
  -> store metrics in backtest_results
  -> write Markdown and CSV backtest summaries plus macro/regime and event-context validation outputs

Merryl doctor run
  -> verify required docs, workflow config, credentials, and generated paths
  -> verify required market symbols and sector maps exist
  -> verify required ETF price coverage and latest-date alignment
  -> verify FRED macro series coverage
  -> verify Alpha Vantage key and SEC EDGAR source configuration
  -> verify historical score-date coverage
  -> verify latest regime, sector, industry, stock, and watchlist row coverage

Merryl dashboard run
  -> start a localhost-only Rust axum server
  -> read dashboard-ready JSON from SQLite
  -> serve the Vite React dashboard build from dashboard/dist when present
  -> show the stored market map without fetching new data or recalculating scores
```

The implementation does not generate fake market candles. If real data credentials are missing, the daily run stops.

## Code Structure

Source code is grouped by responsibility:

```text
src/config/      centralized constants, paths, env var names, defaults, scoring weights
src/domain/      shared domain models
src/data/        provider traits, Alpaca/FRED/Alpha Vantage/SEC adapters, S&P 500 universe, sector ETF mapping
src/storage/     SQLite connection, schema migration, write repositories
src/scoring/     indicators, sector scoring, industry scoring, stock scoring, market orchestration
src/dashboard/   read-only dashboard DTOs, repositories, and local axum server
src/output/      report paths, Markdown rendering, CSV exports
src/workflows/   user workflows such as daily run, status, and doctor
dashboard/       Vite React TypeScript dashboard frontend
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

Current market-data provider:

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

Current macro provider:

```text
FRED API
```

Required environment variable:

```text
FRED_API_KEY
```

Optional environment variables:

```text
FRED_API_URL=https://api.stlouisfed.org
MERRYL_MACRO_LOOKBACK_DAYS=900
```

FRED macro observations are stored in `macro_series` with provenance and coverage metadata. They are not part of the Market Regime V1 score yet.

Current event/catalyst providers:

```text
Alpaca News
Alpha Vantage Earnings Calendar
SEC EDGAR submissions
```

Required environment variables:

```text
ALPHA_VANTAGE_API_KEY
MERRYL_SEC_USER_AGENT=Merryl/0.1 your-email@example.com
```

Optional environment variables:

```text
ALPHA_VANTAGE_API_URL=https://www.alphavantage.co
MERRYL_EARNINGS_CALENDAR_HORIZON=3month
MERRYL_SEC_FILINGS_LOOKBACK_DAYS=14
```

Event rows are stored in `events` with source IDs, raw JSON, effective dates, fetched timestamps, quality status, and optional estimate/surprise fields. Catalyst/event context does not change any score formula.

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

Start the local dashboard:

```text
cargo run -- dashboard
```

Optional dashboard port:

```text
cargo run -- dashboard --port 8787
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
Macro Context Overlay
Macro Context Coverage
Top Sectors
Weak Sectors
Sector Rank Changes
Top Industries Or Themes
Top Stocks Worth Charting
New Leaders
High Relative Volume Names
Catalyst / Event Flags
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

Macro/regime validation outputs:

```text
reports/validations/YYYY-MM-DD_YYYY-MM-DD_macro_regime_validation.md
exports/validations/YYYY-MM-DD_YYYY-MM-DD_macro_regime_validation.csv
```

Event context validation outputs:

```text
reports/validations/YYYY-MM-DD_YYYY-MM-DD_event_context_validation.md
exports/validations/YYYY-MM-DD_YYYY-MM-DD_event_context_validation.csv
```

Phase 3 validation:

```text
docs/phase_3_backtest_validation_spec.md
```

Industry-specific validation:

```text
docs/industry_specific_validation_spec.md
```

Sector score review:

```text
docs/sector_score_review_spec.md
```

Pre-dashboard data quality:

```text
docs/data_quality_reproducibility_spec.md
```

Sector formula decision checkpoint:

```text
docs/sector_formula_decision_checkpoint_spec.md
```

Market regime V1 review:

```text
docs/market_regime_v1_spec.md
```

Market regime formula decision checkpoint:

```text
docs/market_regime_formula_decision_checkpoint_spec.md
```

Phase 5B macro regime validation:

```text
docs/phase_5b_macro_regime_validation_spec.md
```

Spec completeness gate:

```text
docs/spec_completeness_gate_spec.md
```

Catalyst and earnings source decision:

```text
docs/catalyst_earnings_source_spec.md
```

Phase 5C structured catalyst source plan:

```text
docs/phase_5c_structured_catalyst_source_spec.md
```

Phase 5C event context validation checkpoint:

```text
docs/phase_5c_event_context_validation_spec.md
```

Phase 5C source coverage review:

```text
docs/phase_5c_source_coverage_review_spec.md
```

Watchlist convergence review:

```text
docs/watchlist_convergence_review_spec.md
```

Backtest scope clarity:

```text
docs/backtest_scope_clarity_spec.md
```

Pre-dashboard stability backlog:

```text
docs/pre_dashboard_stability_backlog_spec.md
```

Phase 4 dashboard/API plan:

```text
docs/phase_4_dashboard_api_spec.md
```

Phase 5 data-source expansion plan:

```text
docs/phase_5_data_source_expansion_spec.md
```

Dashboard frontend:

```text
dashboard/
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

- Market regime scoring still uses daily ETF price proxies: SPY, QQQ, IWM, DIA, TLT, GLD, and USO. FRED macro series for volatility, rates, yield curve, inflation, employment, credit spread, dollar proxy, and liquidity context are stored and now have a macro/regime validation report, but they are not scoring inputs until a separate formula decision and fresh comparison backtest.
- The first valid score date in a fetched window has no prior rank-change baseline.
- Event context is connected through Alpaca News, Alpha Vantage Earnings Calendar, and SEC EDGAR submissions for the current top watchlist. Watchlist rows can show `recent_news:N`, `earnings:YYYY-MM-DD`, `filing:FORM`, combined labels, or `pending_source`. This remains context only and is not a scoring input.
- Sector ranking is useful as a market-map and attention layer, but PDB-2 labels it as map-only / not yet a proven forward-return predictor. PDB-3.5 removed the neutral rank-change placeholder from sector scoring. Current rank-change is stored and reported, but it is not a scoring component.
- Industry scoring now uses transparent price, relative return, volume, breadth, and 20D-high components. Industry-specific validation is supportive, but it still does not include news/catalyst or industry ETF/fund-flow confirmation.
- Backtesting validates score behavior, not trade profitability. Reports and stored metrics now include validation scope. It does not model transaction costs, slippage, taxes, position sizing, portfolio constraints, or portfolio P&L.
- Data quality checks now run through `doctor` and can catch missing core symbols, ETF price coverage, score-date coverage, latest score row coverage, and replacement-write duplication before report/dashboard use.
- Phase 4 dashboard/API planning is locked and the first read-only slice is implemented: local browser dashboard first, Rust `axum` API, Vite React TypeScript frontend, Tauri later only if packaging is needed, and no Electron in the first dashboard path.

## Current Next Step

The Phase 3 validation checkpoint found that stock scoring has useful forward behavior, sector scoring is mixed, and the industry/theme layer needed hardening before dashboard work. The industry/theme scoring hardening pass is implemented. PDB-1 industry-specific validation, PDB-2 sector score review, PDB-3 market regime V1 review, PDB-3.5 sector formula decision checkpoint, PDB-3.6 spec completeness gate, PDB-4 catalyst/news connection, PDB-5 backtest scope clarity, and PDB-6 data quality/reproducibility are complete.

PDB-3.6 confirmed that the first-build boundaries are aligned with the source specs when they are stated precisely: S&P 500 anchor universe, daily data, GICS industries, SPDR sector ETFs, Alpaca daily prices, Markdown/CSV outputs, and ETF-proxy regime coverage are acceptable first-build scope. PDB-4 connects real recent news catalyst context while keeping structured earnings calendar data explicit as not connected. PDB-5 clarifies that backtests validate score behavior, not trade profitability. PDB-6 adds a direct pre-dashboard data quality gate through `doctor` and storage idempotency tests.

Current implementation priority:

```text
Review the implemented final watchlist classification layer from `docs/watchlist_convergence_review_spec.md`.
```

The first read-only dashboard/API slice from `docs/pre_dashboard_stability_backlog_spec.md` and `docs/phase_4_dashboard_api_spec.md` is implemented. Phase 4.1 dashboard stabilization is complete for the current pass. Phase 5 planning is recorded, and the first Phase 5A/B implementation is complete: FRED macro observations are fetched during the daily workflow, stored with provenance, counted in status, and checked by doctor/dashboard data health without changing scoring weights.

The Phase 5C implementation is recorded in `docs/phase_5c_structured_catalyst_source_spec.md`. It keeps the no-paid-source constraint explicit: preserve Alpaca News, use Alpha Vantage Earnings Calendar only with a free API key, use SEC EDGAR submissions for filing events, and do not add Finnhub, Polygon/Massive, ETF Global, Cboe DataShop, options flow, fund flows, or scoring-weight changes in the first implementation.

The Phase 5C coverage checkpoint is recorded in `docs/phase_5c_source_coverage_review_spec.md`. It accepts Phase 5C as source-backed context for the current ranked stock surface, but it does not approve catalyst/event data as a score input.

The watchlist convergence checkpoint is recorded in `docs/watchlist_convergence_review_spec.md`. It confirms the connected sources are converging toward the final filtered watchlist and implements explicit classification labels before any new provider, paid source, universe expansion, or scoring formula change.

The Phase 5B macro/regime validation implementation is recorded in `docs/phase_5b_macro_regime_validation_spec.md`. It reuses `merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD`, writes macro/regime validation outputs, uses only stored SQLite data, and keeps Market Regime V1 scoring unchanged.

The Market Regime formula decision checkpoint is recorded in `docs/market_regime_formula_decision_checkpoint_spec.md`. It rejects an immediate scoring change, implements a non-scoring macro context overlay in the daily report and dashboard API, and requires a separate fresh comparison before macro data can alter Market Regime V1 scoring.

The stabilization plan is recorded in `docs/phase_4_dashboard_stabilization_spec.md`.

Phase 4.1 implementation completed so far:

```text
selected-date dashboard loading
  -> dashboard data-fidelity tests
  -> compact Market date selector
  -> Overview/Regime/Validation ergonomics pass
```

The Phase 5 planning and implementation reference is recorded in `docs/phase_5_data_source_expansion_spec.md`.

The dashboard must remain a reader over the controlled market-map chain. Do not turn Merryl into a charting platform, trade execution surface, portfolio simulator, or alert engine while expanding data sources.

## Guardrails

- Keep the public CLI small.
- Do not expose internal scoring or ingest steps as public commands.
- Keep provider code replaceable.
- Keep watchlist output separate from trade signals.
- Preserve the top-down flow: market -> sector -> industry/theme -> stock.
