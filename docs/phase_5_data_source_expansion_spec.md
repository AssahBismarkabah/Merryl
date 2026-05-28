# Phase 5 Data Source Expansion Spec

Version: 0.2
Date: 2026-05-28
Status: Phase 5A/B macro ingestion, macro/regime validation, non-scoring macro context overlay, and Phase 5C structured catalyst context complete

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/implementation_spec.md`
- `docs/sector_score_review_spec.md`
- `docs/market_regime_v1_spec.md`
- `docs/market_regime_formula_decision_checkpoint_spec.md`
- `docs/phase_5b_macro_regime_validation_spec.md`
- `docs/catalyst_earnings_source_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`
- `docs/phase_5c_source_coverage_review_spec.md`
- `docs/phase_4_dashboard_stabilization_spec.md`

## 1. Purpose

Phase 5 turns first-build proxies into better external context without changing Merryl's job.

Merryl remains:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> Watchlist for chart review elsewhere
```

Phase 5 is not a dashboard charting phase. Merryl should not become the place where trades are executed, positions are managed, or charts are analyzed manually. The system should continue to answer:

```text
Where is market participation concentrating,
why might it be moving,
which sectors and industries matter,
and which liquid stocks deserve chart review?
```

The goal of this document is to identify every first-build proxy, state what it should become concretely, and decide which data-source upgrades should be investigated or implemented first.

## 2. Current First-Build Proxies

The current build is useful, but several layers are intentionally proxy-based.

| Layer | Current first-build proxy | What it really represents | Concrete target state |
|---|---|---|---|
| Market price coverage | Alpaca daily OHLCV, default IEX feed unless configured otherwise | Tradable price/volume history | Consolidated market-data coverage where precision matters, preferably Alpaca SIP or another official consolidated feed |
| Market regime | ETF proxies: SPY, QQQ, IWM, DIA, TLT, GLD, USO | Risk appetite, growth/defensive tilt, rates sensitivity, inflation/commodity pressure | Macro-aware regime using VIX, rates, yield curve, dollar, credit spreads, liquidity, CPI/employment/Fed context, and release calendar |
| Sector rotation | SPDR sector ETF returns, relative volume, breadth | Where equity participation is concentrating by sector | Sector map plus ETF fund-flow confirmation and regime-aware validation |
| Industry/theme strength | GICS industry baskets from current stock universe | Which groups inside sectors are leading | Industry score plus catalyst/news and, where available, industry ETF/fund-flow confirmation |
| Stock catalysts | Alpaca recent news count/flag on watchlist symbols | Why a stock may be moving now | Structured earnings calendar, major event flags, filing events, and recent-news context |
| Company fundamentals | Mostly absent from current daily scoring | Quality/earnings/revenue context behind leaders | Lightweight fundamental context from SEC/structured fundamentals, used as explanation/context first |
| Options activity | Not connected | Speculative/hedging confirmation and crowdedness | Later confirmation-only options activity flag; not a core score driver until validated |
| Institutional/filing context | Not connected | Longer-term ownership and positioning context | SEC filings, 13F/insider/ownership layer later, mainly for long-term review |
| Universe | S&P 500 anchor universe | Liquid first-build stock universe | Expandable universe such as Russell 1000/3000 or all liquid US stocks after data-quality and mapping are stable |
| Intraday/live data | Not connected | Timing/recency layer | Later optional daily-refresh enhancement, not needed for the current end-of-day market map |

The first valid score date lacking a prior rank-change baseline is not a Phase 5 data-source issue. It is an expected rolling-window behavior.

## 3. Source Verification Snapshot

This section records current candidate source facts checked on 2026-05-28. Access, pricing, and licensing must be rechecked before implementation or purchase.

| Source | Relevant capability | Current role in Phase 5 |
|---|---|---|
| Alpaca Market Data | Historical and real-time equities data. Alpaca documents IEX as the no-subscription feed and SIP as consolidated all-US-exchange coverage. | Keep current provider; consider SIP upgrade only if IEX coverage becomes a material scoring-quality issue. |
| Alpaca News | Historical/real-time news under the Market Data API. | Keep current recent-news context; do not treat news count as full catalyst intelligence. |
| FRED API | Economic series observations, release dates, and many macro/credit/rates series. | First candidate for macro/regime expansion because it is official, broad, and aligned with the current `macro_series` schema. |
| SEC EDGAR APIs | Company submissions and XBRL company facts; SEC states these APIs require no API keys and are updated as filings disseminate. | First candidate for filing/fundamental context, not for real-time catalyst scoring. |
| Alpha Vantage Earnings Calendar | Upcoming earnings calendar endpoint available with a free API key and daily free-call limits. | First candidate for Phase 5C structured earnings context because it can be used without paid access. |
| Finnhub Earnings Calendar | Historical and upcoming earnings calendar; also has economic calendar/economic data endpoints, with some premium access. | Not the first Phase 5C source under the no-paid-source constraint; revisit only if free access is confirmed and Alpha Vantage is unsuitable. |
| ETF Global via Massive/Polygon partner API | ETF fund-flow endpoint with processed/effective dates and daily updates. | Candidate for sector/industry ETF fund-flow confirmation; likely paid and should not be first unless budget is approved. |
| Massive/Polygon market data | REST APIs for stocks/options/futures/indices/economy and ETF Global partner datasets. | Candidate if Merryl needs a broader paid provider across several Phase 5 layers. |
| Cboe DataShop | Official marketplace for options, equities, ETFs, indices, futures, and historical/intraday datasets, including Open-Close Volume Summary and Option EOD Summary. | Candidate for later options activity confirmation; paid/complex and not first implementation. |

Primary source URLs:

- Alpaca historical stock data and feed descriptions: `https://docs.alpaca.markets/us/docs/historical-stock-data-1`
- Alpaca market data overview: `https://docs.alpaca.markets/us/v1.1/docs/about-market-data-api`
- FRED API documentation: `https://fred.stlouisfed.org/docs/api/fred/series/series_observations.html`
- SEC EDGAR API documentation: `https://www.sec.gov/edgar/sec-api-documentation`
- Alpha Vantage support/free key and limits: `https://www.alphavantage.co/support/`
- Alpha Vantage earnings calendar docs: `https://www.alphavantage.co/documentation/`
- Finnhub earnings calendar docs: `https://finnhub.io/docs/api/earnings-calendar`
- Finnhub economic calendar docs: `https://finnhub.io/docs/api/economic-calendar`
- ETF Global fund flows API via Massive: `https://massive.com/docs/rest/partners/etf-global/fundflows`
- Massive/Polygon REST API quickstart: `https://massive.com/docs/rest/quickstart`
- Cboe DataShop: `https://datashop.cboe.com/`

## 4. Phase 5 Non-Negotiable Guardrails

Phase 5 must not weaken the current validated flow.

- Do not turn Merryl into a charting platform.
- Do not add trade execution, portfolio simulation, or position sizing.
- Do not treat options flow, dark pools, or gamma as required core inputs.
- Do not change scoring weights just because a new source is available.
- Do not make a paid source mandatory until its value is proven.
- Do not add many public CLI commands for each data source.
- Do not present sector ranking as a proven standalone forward-return signal.
- Do not present backtests as trade profitability.
- Do not hide source limitations; every new source needs freshness, coverage, and provenance metadata.

New data should enter through provider adapters and stored tables first. Scoring changes require a separate validation document and fresh backtest.

## 5. Concrete Proxy-To-Target Decisions

### 5.1 Market Regime Proxy

Current proxy:

```text
SPY, QQQ, IWM, DIA, TLT, GLD, USO daily ETF behavior
```

Concrete target:

```text
ETF-proxy regime
  + VIX/risk stress
  + rates and yield curve
  + dollar pressure
  + credit/liquidity stress
  + inflation/employment/Fed context
  + economic release calendar
```

First candidate source:

```text
FRED API
```

Initial target series:

| Need | Candidate series/source |
|---|---|
| Equity volatility/risk stress | `VIXCLS` where available through FRED |
| 10Y yield | `DGS10` |
| 2Y yield | `DGS2` |
| Yield curve | `T10Y2Y` or calculated from `DGS10 - DGS2` |
| Fed funds | `DFF` or `FEDFUNDS` |
| Inflation | CPI series such as `CPIAUCSL` |
| Employment | `UNRATE`, `PAYEMS` |
| Credit spread | ICE/BofA corporate spread series such as `BAMLC0A0CM` if coverage is acceptable |
| Dollar pressure | FRED dollar index proxy where acceptable, or a paid DXY source later |
| Liquidity | Fed balance-sheet/liquidity proxies where acceptable |

Implementation stance:

- Store macro series first.
- Display/label macro coverage and as-of macro context flags in dashboard/report.
- Do not change Market Regime V1 scoring until data freshness and historical coverage are validated.

Acceptance before scoring changes:

- At least 2 years of history for chosen daily/weekly/monthly macro series where available.
- Clear frequency handling for monthly releases versus daily prices.
- Vintage/revision policy documented.
- Backtest comparison against current ETF-proxy regime.

### 5.2 Catalyst And Earnings Proxy

Current proxy:

```text
recent_news:N from Alpaca News for top watchlist symbols
structured earnings calendar = not connected
```

Concrete target:

```text
Recent news context
  + upcoming earnings date
  + last earnings date and surprise where available
  + 8-K / major filing events
  + analyst/upgrades/regulatory event flags later if source is available
```

First candidate sources under the no-paid-source constraint:

```text
Alpha Vantage Earnings Calendar
SEC EDGAR submissions/company facts
Alpaca News remains the current recent-news provider
```

Implementation stance:

- Add structured earnings date as the first catalyst upgrade if access is acceptable.
- Add SEC filings as a slower but official context layer.
- Keep catalyst fields as context, not automatic scoring weights.

Acceptance before scoring changes:

- Earnings date coverage for current watchlist names.
- Clear handling for unknown/unconfirmed earnings dates.
- Report/dashboard labels distinguish news, earnings, and filings.
- Backtest can compare watchlist names with and without catalyst flags.

### 5.3 Sector Flow Proxy

Current proxy:

```text
Sector ETF price/volume behavior
```

Concrete target:

```text
Sector ETF price/volume
  + sector ETF fund inflow/outflow
  + flow persistence
  + flow alignment with relative strength and breadth
```

Candidate sources:

```text
ETF Global fund flows through Massive/Polygon partner API
Other paid ETF flow vendors only if ETF Global is unsuitable
```

Implementation stance:

- Treat ETF flows as confirmation for sector participation, not as direct hidden-money truth.
- Start with SPDR sector ETFs before expanding to industry ETFs.
- Do not add fund flows until access/pricing is approved.

Acceptance before scoring changes:

- Daily or near-daily flow coverage for the sector ETF list.
- Effective-date and processed-date handling.
- Validation that flows add value versus price/volume-only sector ranking.

### 5.4 Industry And Theme Proxy

Current proxy:

```text
GICS industry baskets built from current universe
```

Concrete target:

```text
GICS industry baskets
  + optional industry ETF confirmation where liquid ETFs exist
  + catalyst/news concentration inside the industry
  + later custom theme baskets
```

Candidate sources:

```text
Current sector/industry map
ETF Global / ETF reference data for industry ETFs
News and earnings data from the catalyst layer
```

Implementation stance:

- Preserve GICS as the base grouping.
- Add source-backed confirmation columns before changing scoring.
- Dynamic AI theme classification is deferred.

Acceptance before scoring changes:

- Industry source labels remain explainable.
- Industry scores can be compared with and without new confirmation fields.
- Dashboard/report continues to show industry as an attention layer, not a trade signal.

### 5.5 Stock Leadership Proxy

Current proxy:

```text
Daily relative strength, relative volume, liquidity, trend state, sector/industry context
```

Concrete target:

```text
Current stock opportunity score
  + structured catalyst context
  + optional fundamental quality/context
  + optional filing/ownership context
```

Candidate sources:

```text
SEC EDGAR APIs for filings and company facts
Finnhub or another provider for structured earnings calendar
Alpaca News for recent news
```

Implementation stance:

- Preserve the current stock score because it has the strongest validation behavior.
- Add context first; scoring changes only after validation.
- Avoid fundamental-heavy ranking until the daily rotation model proves a real need.

Acceptance before scoring changes:

- Component storage clearly separates price/volume score from catalyst/fundamental context.
- Backtest proves whether new context improves forward behavior or false-positive filtering.

### 5.6 Options Activity Proxy

Current proxy:

```text
No options activity connected
```

Concrete target:

```text
Unusual options activity / open-close volume / call-put pressure as confirmation only
```

Candidate sources:

```text
Cboe DataShop
Massive/Polygon options endpoints
Market Chameleon or other vendor only after licensing review
```

Implementation stance:

- Defer from first Phase 5 implementation.
- Do not add 0DTE gamma, dealer positioning, or dark-pool inference yet.
- If added later, start with simple unusual-options flag and validate it.

Acceptance before scoring changes:

- Official/licensed source.
- Historical availability for backtest.
- Clear distinction between speculative attention and bullish/bearish direction.

### 5.7 Universe Proxy

Current proxy:

```text
S&P 500 anchor universe
```

Concrete target:

```text
Expandable liquid US equity universe
```

Candidate directions:

```text
Russell 1000 / Russell 3000 if source/licensing is acceptable
All US listed stocks filtered by liquidity if provider reference data is reliable
Custom watchlists and theme baskets later
```

Implementation stance:

- Do not expand the universe before source quality and data health gates are ready.
- Expansion must preserve sector/industry mapping and liquidity filters.
- Survivorship-bias handling becomes more important after expansion.

Acceptance before implementation:

- Reliable ticker source.
- Sector/industry mapping coverage.
- Liquidity filter rules.
- Data coverage and missing-map doctor checks updated.

### 5.8 Intraday/Live Data Proxy

Current proxy:

```text
Daily OHLCV only
```

Concrete target:

```text
Optional intraday recency layer for daily review
```

Candidate sources:

```text
Alpaca SIP or delayed SIP
Massive/Polygon minute aggregates
```

Implementation stance:

- Defer unless daily review clearly needs intraday recency.
- Do not use intraday data for execution logic in Phase 5.
- Do not build alerts in the first Phase 5 implementation.

Acceptance before implementation:

- Clear user workflow that daily data cannot satisfy.
- Historical intraday coverage sufficient for validation.
- Storage growth impact understood.

## 6. Recommended Phase 5 Order

Phase 5 should be split into smaller decision-backed passes.

### Phase 5A: Source Contracts And Provenance

Purpose:

```text
Prepare the system to ingest new sources without making scoring changes.
```

Build:

- Source/provider registry.
- Source freshness metadata.
- Effective date versus processed date support.
- Provenance fields for macro/events/flows.
- Doctor checks for new source freshness and coverage.

Why first:

- Prevents a pile of one-off provider code.
- Keeps reports honest about what is connected and what is not.

### Phase 5B: Macro/Regime Expansion

Purpose:

```text
Replace ETF-only regime with macro-aware context.
```

First source:

```text
FRED API
```

Build:

- Macro series ingestion.
- Regime context rows.
- Report/dashboard display of macro coverage.
- Validation comparison against current regime.

### Phase 5C: Structured Catalyst And Earnings Calendar

Purpose:

```text
Turn catalyst awareness from recent-news-only into structured event context.
```

First candidates under the no-paid-source constraint:

```text
Alpha Vantage earnings calendar
SEC EDGAR submissions
```

Build:

- Earnings/event table ingestion.
- Watchlist catalyst display: news, earnings, filing, unknown.
- Backtest grouping by catalyst availability.

### Phase 5D: ETF Fund Flows

Purpose:

```text
Improve sector participation evidence.
```

First candidate:

```text
ETF Global fund flows through Massive/Polygon partner API
```

Build only if:

- Access/pricing is approved.
- Sector ETF coverage is clear.
- Historical data supports validation.

### Phase 5E: Universe Expansion

Purpose:

```text
Find more liquid leaders after source quality is stable.
```

Build only after:

- Provider coverage is strong enough.
- Sector/industry mapping is reliable.
- Doctor can detect missing maps and data gaps at larger scale.

### Phase 5F: Options/Intraday Confirmation

Purpose:

```text
Add advanced participation confirmation only after the simpler data layers prove useful.
```

Default decision:

```text
Defer.
```

## 7. First Phase 5 Implementation Recommendation

Recommended first implementation after this planning document:

```text
Phase 5A + Phase 5B foundation
```

Meaning:

1. Add source/provenance support if missing.
2. Add FRED macro series ingestion.
3. Store macro data without changing score weights.
4. Display macro coverage as context.
5. Validate whether macro context improves regime interpretation.

Reason:

- It addresses the largest current limitation: ETF-proxy market regime.
- It uses an official, broad, low-friction macro source.
- It improves the top of the market-map chain without turning Merryl into a charting or trading tool.

Second recommended implementation:

```text
Phase 5C structured earnings/catalyst calendar
```

Reason:

- It improves the "why is this moving?" question.
- It directly improves watchlist review without changing the core score.
- It can start with free sources: existing Alpaca News, Alpha Vantage Earnings Calendar, and SEC EDGAR submissions.

Do not start Phase 5 with ETF fund flows, options flow, intraday data, or universe expansion unless the user explicitly chooses those after reviewing cost and complexity.

## 7.1 Implemented Phase 5A/B Boundary

Implemented on 2026-05-28:

- Added a FRED provider adapter under `src/data/`.
- Required `FRED_API_KEY` for the daily workflow.
- Fetched configured macro series during `merryl run daily --date latest`.
- Stored macro observations in `macro_series` with series name, frequency, units, realtime dates, raw JSON, source, and quality status.
- Added idempotent SQLite migration coverage for macro provenance columns.
- Added doctor/data-health checks for required FRED macro series coverage.
- Added Markdown report and dashboard data-health coverage for macro series.
- Added status output for stored macro observation count.
- Kept Market Regime V1 scoring unchanged.

Current configured FRED series:

```text
VIXCLS
DGS10
DGS2
T10Y2Y
DFF
CPIAUCSL
UNRATE
PAYEMS
BAMLC0A0CM
DTWEXBGS
WALCL
```

Real run verification on 2026-05-28:

```text
merryl run daily --date latest
  -> macro observations stored: 4843

merryl doctor
  -> ok: FRED macro coverage present (11/11)
```

What this does not change:

- It does not make macro data a scoring input yet.
- It does not replace the ETF-proxy regime score.
- It does not add a macro calendar or release-event model.
- It does not change sector, industry, or stock scoring weights.

Next checkpoint before any macro scoring change:

```text
docs/phase_5b_macro_regime_validation_spec.md
```

This checkpoint must compare ETF-proxy regime behavior with FRED-aware context over historical scored dates before any Market Regime V1 scoring change.

Implemented macro/regime validation on 2026-05-28:

- Reused `merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD`.
- Added no new public CLI command.
- Read stored `market_regime_scores`, `sector_scores`, and `macro_series` rows from SQLite.
- Built as-of macro snapshots using only observations on or before each score date.
- Wrote validation outputs under `reports/validations/` and `exports/validations/`.
- Kept Market Regime V1 score weights unchanged.

Live validation output:

```text
reports/validations/2025-07-01_2026-05-28_macro_regime_validation.md
exports/validations/2025-07-01_2026-05-28_macro_regime_validation.csv
```

Live validation summary:

| Metric | Value |
|---|---:|
| Macro regime snapshots | 229 |
| Complete macro snapshots | 229 |
| Missing macro snapshots | 0 |
| Risk-on dates with active macro stress flags | 103 |

This validates macro context availability and disagreement visibility. It does not approve macro data as a scoring input.

Formula decision checkpoint:

```text
docs/market_regime_formula_decision_checkpoint_spec.md
```

Decision:

```text
Do not change Market Regime V1 scoring yet.
The non-scoring macro context overlay is implemented.
```

Implemented overlay result:

- Daily reports include a `Macro Context Overlay` section beside Market Regime.
- Dashboard API exposes the same context under `regime.macro_context`.
- Macro flags are explicitly labeled as context, not Market Regime V1 score inputs.
- Market Regime V1 score and label remain ETF-proxy based and unchanged.

## 7.2 Implemented Phase 5C Boundary

Planning and first implementation recorded on 2026-05-28:

```text
docs/phase_5c_structured_catalyst_source_spec.md
```

The Phase 5C implementation decision is:

- Use free sources only.
- Keep existing Alpaca News for recent headline context.
- Use Alpha Vantage Earnings Calendar as the first structured earnings source with a free API key.
- Use SEC EDGAR submissions as the first filing-event source.
- Do not use Finnhub, Polygon/Massive, ETF Global, Cboe DataShop, options flow, or fund-flow vendors in the first Phase 5C implementation.
- Do not change sector, industry, stock, or regime score weights.
- Do not add a new public CLI command.

Phase 5C is a catalyst/event-context upgrade, not a scoring rewrite.

Implemented components:

- `src/data/alpha_vantage.rs`
- `src/data/sec_edgar.rs`
- additive `events` provenance columns and source-event idempotency index
- structured catalyst status composition
- `Catalyst / Event Flags` report section
- doctor/status source visibility
- tests for parsers, migration, idempotency, status composition, and report output

Current live-data note:

```text
Phase 5C live daily verification passed on 2026-05-28 with Alpaca News, Alpha Vantage Earnings Calendar, and SEC EDGAR submissions connected.
```

Post-implementation coverage checkpoint:

```text
docs/phase_5c_source_coverage_review_spec.md
```

The coverage checkpoint accepted Phase 5C as source-backed context for the current ranked stock surface. It did not approve any catalyst/event scoring-weight change.

Current Phase 5 target:

```text
Review the implemented macro overlay and Phase 5C catalyst coverage before considering any scoring formula change or paid source.
```

Do not move to paid ETF fund flows, options flow, intraday data, or universe expansion before a separate decision changes the order.

## 8. Data Model Requirements

New source data should be stored separately before it affects scoring.

Required metadata for every new source table:

```text
source_name
source_url_or_endpoint
symbol_or_series_id
observation_date
effective_date
processed_at
fetched_at
value
raw_json
quality_status
```

For macro data:

```text
series_id
series_name
frequency
units
source_name
observation_date
value
release_date
vintage_date
fetched_at
```

For events/catalysts:

```text
symbol
event_type
event_date
event_time
source_name
headline_or_label
actual
estimate
surprise
url
raw_json
fetched_at
```

For ETF flows:

```text
etf_symbol
effective_date
processed_date
flow_value
assets
source_name
raw_json
fetched_at
```

## 9. Validation Requirements

No new source should change scoring until it passes a validation checkpoint.

Required validation questions:

- Does this source cover the symbols/series Merryl actually uses?
- Is the data available historically for backtesting?
- Is the data delayed, revised, or restated?
- Can the system distinguish missing data from neutral data?
- Does the new context improve false-positive filtering?
- Does it improve the top-down flow, or is it only interesting noise?
- Can the dashboard/report explain it without implying a trade signal?

Every scoring change must create or update a validation document.

## 10. Out Of Scope For First Phase 5 Implementation

These remain deferred until a specific decision says otherwise:

- Trade execution.
- Portfolio simulation.
- Position sizing.
- Alerts.
- 0DTE gamma/dealer positioning.
- Dark-pool inference.
- Full NLP sentiment engine.
- AI-generated theme classification.
- Intraday execution layer.
- Cloud deployment or multi-user authentication.
- Replacing the current stock score before new evidence exists.

## 11. Acceptance Criteria For This Planning Phase

This document is accepted when:

- Every current first-build proxy has a concrete target state.
- Each target state maps back to the original market structure model.
- Candidate sources are listed with access/cost risk called out.
- The first recommended implementation does not require a paid source.
- The plan preserves the current validated scoring flow.
- The plan does not add dashboard charting or trade execution.
- `docs/implementation_spec.md` points to this document as the current Phase 5 planning reference.
