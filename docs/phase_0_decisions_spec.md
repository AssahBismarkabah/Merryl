# Phase 0 Decisions

Version: 0.1  
Date: 2026-05-26  
Status: Decision document before implementation  
Related spec: `market_rotation_system_spec.md`

## 1. Purpose

The main specification defines the market rotation and stock discovery system.

This document defines the decisions we need to make before building anything. The goal is to prevent the first implementation from becoming too broad, too expensive, or too complex.

The immediate next step is not to build the full dashboard. The immediate next step is to lock the first useful version:

```text
Daily market map -> sector ranking -> stock ranking -> watchlist report
```

## 2. Current Phase

We are in:

```text
Phase 0: Research And Spec
```

Phase 0 is complete when we have:

- A clear system specification.
- A clear MVP definition.
- A chosen market universe.
- A chosen data source strategy.
- A chosen storage strategy.
- A chosen first output format.
- A locked v1 scoring method.
- Clear acceptance criteria for Phase 1.

The main spec is already created. The remaining Phase 0 work is decision-making.

## 3. Main Decision To Make Now

We need to decide what the first version is optimized for.

Recommended first version:

```text
A market rotation intelligence system that starts with daily US equity data,
maps broad market -> sector -> industry/theme -> stock leadership,
and produces a chart-worthy watchlist from where participation is concentrating.
```

This first version may use S&P 500 names and daily data as the controlled starting slice, but that is not the final product boundary. The final intent remains a broader market map that can expand into more stocks, intraday/live data, options flow, fund flows, and advanced participation signals after the foundation is proven.

The first build must preserve the original structure:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> Chart timing and risk plan
```

The build should be narrow enough to execute, but not so narrow that it becomes a basic stock screener.

## 3.1 Alignment Guardrails

These guardrails prevent drift from the original system idea:

- Do not build a random stock scanner.
- Do not build a pure chart-pattern tool.
- Do not build an auto-trading bot.
- Do not ignore macro/sector context.
- Do not ignore catalyst awareness; v1 may use simple flags/placeholders, but the product must preserve the question "why is this moving?"
- Do not reduce the product to only daily trading; the system must support daily, weekly, and long-term review modes over time.
- Do not make S&P 500 the permanent universe.
- Do not make daily data the permanent timeframe.
- Do not require options/dark pool/gamma data in v1, but keep the architecture open for them.
- Do not let the first UI/dashboard distract from proving the market map and scoring logic.
- Always preserve the top-down path: market -> sector -> industry/theme -> stock.

## 3.2 Mapping To The Main Spec

This Phase 0 document does not replace `market_rotation_system_spec.md`. It narrows only the first implementation slice while preserving the full system direction.

| Main spec area | Original intent | Phase 0 handling |
|---|---|---|
| Executive summary | Map where participation is concentrating and reduce thousands of stocks to chart-worthy names | Preserved as market rotation intelligence + watchlist output |
| Core idea | Identify rotation from broad markets into sectors, themes, and leaders | Preserved through top-down flow |
| Foundational model | What, who, why, how, where, when | Preserved as scoring, macro context, sector/industry/stock mapping, and later advanced layers |
| Daily use case | Daily market prep and chart-worthy names | V1 focus |
| Weekly use case | Review 5D/20D/60D sector rotation and build next-week watchlist | Preserved; reports should support weekly review once historical scores exist |
| Long-term use case | Observe sector behavior through macro cycles and improve by evidence | Preserved through historical storage and backtesting |
| Market regime | Risk-on/risk-off/defensive/mixed context | Preserved with lightweight v1 regime model |
| Sector rotation | Rank major sectors by strength, volume, breadth, and rank changes | Phase 1 core |
| Industry/theme strength | Show strong groups inside sectors | Phase 2 core, schema must support expansion |
| Stock leadership | Rank liquid stocks inside active sectors/industries | Phase 2 core |
| Catalyst awareness | Understand why a sector/stock is moving | Preserved as simple flags/placeholders first, richer news/events later |
| Advanced participation | ETF flows, options flow, gamma, dark pools, positioning, intraday | Deferred as dependencies, preserved as later layers |
| Backtesting | Validate whether scores predict future relative returns | Phase 3 core |
| Risk rule | Watchlist, not automatic trade signal | Preserved as a non-negotiable rule |

## 3.3 Non-Negotiable Central Point

The central point remains:

```text
Show where market participation is concentrating,
map that concentration from market regime to sector to industry/theme to stock,
then produce a small list of liquid, chart-worthy names with explainable reasons.
```

Any implementation choice that weakens this central point should be rejected.

## 4. Decisions Table

| Decision | Recommended V1 Choice | Why | Defer |
|---|---|---|---|
| Primary market | US equities | Most data availability and sector structure | Forex, crypto, international stocks |
| First universe | S&P 500 anchor universe | Liquid, cleaner sector mapping, easier validation | Russell 3000/all liquid US stocks |
| Timeframe | Daily data first, intraday-ready schema | Enough for rotation and first validation | 1-minute/5-minute intraday execution layer |
| First output | Markdown report + CSV | Fast to build and inspect | Full web dashboard |
| Storage | SQLite | Simple, local, queryable, no server needed | Postgres/cloud database |
| Sector model | SPDR sector ETFs | Clear ETF proxies for each sector | Custom sector baskets |
| Stock grouping | GICS sector/industry where available | Standard market classification | Custom AI theme classification |
| Scoring | Transparent weighted scores | Explainable and backtestable | ML model |
| Macro | Minimal regime indicators | Context without overcomplication | Full macro engine |
| Catalyst awareness | Simple event/catalyst fields, even if sparse at first | Preserves the "why is it moving?" question | Full news/NLP engine |
| Options flow | Exclude as v1 dependency | Cost/complexity too high early | Add as confirmation layer |
| News | Simple catalyst flag later | Full news parsing can distract early | NLP news sentiment |
| Backtesting | Required before dashboard polish | Prevents building on weak assumptions | Live alerts/trading automation |

## 5. Decision 1: Market Universe

### Options

Option A: S&P 500 first  
Option B: Russell 1000 first  
Option C: All US stocks above liquidity threshold  

### Recommendation

Start with:

```text
S&P 500 anchor universe first
```

Reasons:

- Cleaner data.
- Better liquidity.
- Easier sector mapping.
- Easier to validate.
- Less noise from penny stocks and low-quality names.
- Enough stocks to produce useful watchlists.

### Acceptance

The first build should support at least:

- All current S&P 500 constituents.
- Major sector ETFs.
- Broad market ETFs.

Later we can expand to Russell 1000 or all liquid US stocks.

Important distinction:

```text
S&P 500 first means first build scope.
It does not mean the product is only for S&P 500 stocks.
```

The data model and scoring logic should be designed so the universe can later expand to:

- Russell 1000.
- Russell 3000.
- All US stocks above a liquidity threshold.
- Theme baskets.
- Custom watchlists.

## 6. Decision 2: Data Provider

### Options

Option A: Polygon  
Option B: Alpaca  
Option C: Finnhub  
Option D: Databento  
Option E: Temporary free data for prototyping  

### Recommendation

Use a two-step approach:

```text
First implementation: use a real accessible daily data source through a provider adapter.
Production v1: use Polygon or Alpaca for cleaner API-based data.
```

Polygon is a strong candidate for market data. Alpaca is practical if brokerage/API integration may matter later. Databento is serious but may be more than we need for the first daily rotation system.

### Decision Needed

Choose one:

```text
1. Prototype fast with free/available daily data first.
2. Start directly with a paid/official API provider.
```

Recommended:

```text
Use a real accessible provider first, then upgrade the provider/feed once the model proves useful.
```

Guardrail:

The implementation should use a data-provider interface so the scoring system is not locked to one temporary source. If we prototype with a free or easy source, it must be treated as a replaceable adapter, not the permanent foundation.

Implementation selection:

```text
Use Alpaca Market Data API first.
Require API keys through environment variables.
Do not generate fake market candles for the daily workflow.
```

## 7. Decision 3: Storage

### Options

Option A: CSV files  
Option B: SQLite  
Option C: Postgres  
Option D: Cloud database  

### Recommendation

Use:

```text
SQLite
```

Reasons:

- Simple local setup.
- Easy to query.
- Better than scattered CSV files.
- Good enough for daily S&P 500 data.
- Can migrate to Postgres later.

CSV exports should still be generated for review.

## 8. Decision 4: First Output

### Options

Option A: Markdown daily report  
Option B: CSV watchlist  
Option C: Notebook  
Option D: Web dashboard  

### Recommendation

Build first output as:

```text
Markdown report + CSV exports
```

The first output should produce:

- `reports/YYYY-MM-DD_market_report.md`
- `exports/YYYY-MM-DD_sector_scores.csv`
- `exports/YYYY-MM-DD_stock_watchlist.csv`

This lets us inspect usefulness before spending time on UI.

## 9. Decision 5: V1 Scoring Formula

The first scoring method should be simple, explainable, and stable.

### Sector Flow Score V1

```text
Sector Flow Score =
  30% relative return vs SPY
  20% 20-day trend strength
  20% relative volume
  20% breadth inside sector
  10% rank improvement
```

### Stock Opportunity Score V1

```text
Stock Opportunity Score =
  30% sector flow score
  25% stock relative strength vs sector
  20% stock relative volume
  15% trend structure
  10% liquidity quality
```

### What V1 Excludes As Required Dependencies

V1 should not require these to work:

- Options flow.
- Dark pools.
- Gamma exposure.
- News NLP.
- 13F filings.
- Insider filings.
- Automated trade execution.

These are not ignored forever. They are deferred until the foundation works.

The architecture should still leave room for them as later confirmation layers:

```text
Core score first:
price + relative strength + volume + breadth + sector context

Advanced confirmation later:
fund flows + options flow + gamma + dark pools + news/catalysts + intraday order flow
```

## 10. Decision 6: Market Regime V1

The first market regime model should be lightweight.

Use:

- SPY trend.
- QQQ vs SPY.
- IWM vs SPY.
- VIX trend.
- TLT trend.
- US 10Y yield if available.
- DXY if available.

Output:

```text
Risk-on
Risk-off
Defensive
Mixed
```

The regime label is context. It should not block the sector/stock ranking.

Macro preservation rule:

```text
V1 can start with market-based regime proxies,
but the schema and technical plan should leave room for macro_series and events.
```

This preserves the original macro intent without forcing a full macro engine before the rotation map is proven.

## 11. Decision 7: Phase 1 Acceptance Criteria

Phase 1 should be considered complete only when the system can:

1. Load daily OHLCV data for broad ETFs, sector ETFs, and S&P 500 stocks.
2. Store the data in SQLite.
3. Maintain a symbol table with sector and industry mapping.
4. Calculate 1D, 5D, 20D, and 60D returns.
5. Calculate returns relative to SPY.
6. Calculate sector scores.
7. Generate a sector ranking report for the latest date.
8. Generate historical sector rankings.
9. Explain each sector score using component values.
10. Include schema room for later `macro_series`, `events`, `prices_intraday`, and advanced confirmation layers even if they are not populated yet.

Phase 1 does not need:

- Stock watchlist ranking.
- Dashboard UI.
- Options data.
- News data.
- Intraday data.

## 12. Phase 2 Acceptance Criteria

Phase 2 should be considered complete when the system can:

1. Rank stocks inside each sector.
2. Apply liquidity filters.
3. Calculate relative strength vs sector ETF.
4. Calculate relative volume.
5. Calculate trend structure.
6. Generate top 20-50 stocks worth charting.
7. Explain why each stock appears.
8. Export the watchlist to CSV and Markdown.
9. Carry earnings/catalyst fields when available, even if the first version only marks them as unknown or pending.

## 13. Phase 3 Acceptance Criteria

Phase 3 should be considered complete when we can answer:

```text
Do high-ranked sectors outperform low-ranked sectors over the next 5D/20D?
Do high-ranked stocks outperform their sector over the next 5D/20D?
```

Required outputs:

- Score decile analysis.
- Forward return analysis.
- Hit rate.
- Average and median forward returns.
- Relative performance vs SPY and sector.
- Basic transaction cost assumptions if portfolio simulation is added.

## 14. The Next Document To Create

After this decision document, the next document should be:

```text
mvp_technical_plan.md
```

That document should translate the decisions into implementation details:

- Folder structure.
- Database schema.
- Data ingestion scripts.
- Scoring modules.
- Report generation.
- Backtest scripts.
- Config format.
- CLI commands.
- Exact Phase 1 tasks.

This current document answers:

```text
What are we choosing?
```

The next technical plan answers:

```text
How exactly will we build it?
```

## 15. Recommended Locked Decisions

Unless we decide otherwise, the recommended locked decisions are:

```text
Market: US equities
Universe: S&P 500 anchor universe first, expandable later
Data timeframe: daily first, intraday/live-ready later
Output: Markdown + CSV
Storage: SQLite
Provider: Alpaca Market Data API first, with provider interface kept replaceable
Scoring: transparent weighted scores
Main use: market rotation map + chart-worthy watchlist for daily/weekly preparation
V1 does not require: options, dark pools, gamma, intraday order flow, AI prediction
```

## 16. Immediate Next Step

The immediate next step is to confirm or revise the recommended locked decisions above.

Once those decisions are accepted, create:

```text
mvp_technical_plan.md
```

Then implementation can begin with Phase 1: Data Foundation.
