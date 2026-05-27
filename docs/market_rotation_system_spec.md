# Market Rotation And Stock Discovery System

Version: 0.2
Date: 2026-05-27
Status: Source specification; implementation is complete through PDB-6 and the first Phase 4 dashboard/API slice
Purpose: Define what we are building, why it matters, what data is needed, what can be left out, and how to validate the system before relying on it.

## 1. Executive Summary

The goal is to build a market mapping system that helps us avoid randomly looking at stocks and instead focus attention where market participation is actually concentrating.

The system is not meant to be a magic buy/sell signal. It is meant to answer a practical trading and research question:

> Where is money moving in the stock market, which sectors are receiving attention, which stocks inside those sectors are leading, and which names are worth charting now?

The system should reduce thousands of stocks into a smaller, ranked watchlist of stocks that are more likely to matter because they sit inside active sectors, show relative strength, have volume participation, and ideally have a catalyst or macro reason behind the move.

The product should work from the top down:

```text
Macro regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> Chart timing and risk plan
```

The first version should focus on:

- Market regime.
- Sector and industry rotation.
- Relative strength.
- Volume acceleration.
- Breadth.
- Liquid stock discovery.
- Catalyst awareness.
- Backtesting and validation.

The first version should not depend on expensive or complex features like dark pool inference, 0DTE gamma modeling, full order book data, or AI prediction. Those can be added later after the basic market map works.

## 2. Core Idea

The stock market has structure. Money does not move randomly across all stocks at the same time. It rotates:

- From cash/bonds into equities when risk appetite improves.
- From growth sectors into defensive sectors when risk appetite weakens.
- From broad indices into specific themes when a narrative becomes dominant.
- From lagging stocks into leaders when institutions concentrate exposure.
- From crowded trades into new opportunities when positioning shifts.

The system should identify these rotations early enough to help us know where to focus.

The system should not try to tell us:

```text
Buy stock X now because it will go up.
```

It should tell us:

```text
Technology is outperforming SPY over 5D and 20D.
Breadth inside technology is expanding.
Volume is above normal.
Semiconductors are leading within technology.
NVDA, AMD, AVGO, and MU are leading the group.
These are the stocks worth charting today.
```

That is a useful decision support tool.

## 3. Foundational Market Model

The framework from the transcript and screenshots can be organized into six market questions:

### 3.1 What Is Moving?

Price is what moves.

For this system, price movement should be measured across:

- Broad index ETFs: SPY, QQQ, IWM, DIA.
- Sector ETFs: XLK, XLF, XLV, XLE, XLY, XLP, XLI, XLB, XLU, XLRE, XLC.
- Industry ETFs where available.
- Individual stocks.
- Intermarket assets: VIX, DXY, TLT, US10Y, GLD, USO/CL proxies.

Price alone is not enough. We need to know whether price is moving with participation.

### 3.2 Who Is Moving It?

Market participants can be grouped into:

- Big money: pension funds, sovereign funds, mutual funds, large asset managers, banks, large corporates.
- Smart money: hedge funds, investment banks, proprietary trading firms, HFT firms.
- Market makers: liquidity providers, options market makers, ETF arbitrage participants.
- Retail traders: individual traders and smaller accounts.

In practice, we cannot directly observe every participant. We infer participation from proxies:

- Volume.
- Relative volume.
- ETF flows.
- Options activity.
- Futures positioning.
- Sector breadth.
- Price persistence.
- News/catalyst reaction.
- Institutional filings for longer-term positioning.

### 3.3 Why Are They Moving It?

Market participants buy and sell for different reasons:

- Capital growth.
- Income/dividends.
- Capital preservation.
- Hedging.
- Speculation.
- Rebalancing.
- Risk management.
- Forced liquidation.
- Index inclusion/exclusion.
- Macro repricing.

The system should connect movement to possible reasons:

- Macro regime: growth, inflation, rates, liquidity, employment.
- Sector fundamentals: energy follows oil, real estate reacts to rates, banks react to rates and credit, technology reacts to growth expectations and capital expenditure.
- Company fundamentals: earnings, revenue growth, margins, guidance, balance sheet, product cycle, management.
- Catalyst: earnings, analyst upgrades, FDA decisions, regulation, mergers, macro data, geopolitical events.
- Positioning: crowded longs, short interest, options skew, ETF/fund flows.

### 3.4 How Is It Moving?

Money moves through orders. Orders create volume. Volume and price together reveal participation.

Useful measures:

- Total volume.
- Relative volume.
- Dollar volume.
- Up volume vs down volume.
- Advancers vs decliners.
- New highs vs new lows.
- Breakout volume.
- Gap volume.
- Volume by time period.
- Volume by price if charting tools support it.

For the first build, we do not need raw order book data. Daily and intraday OHLCV data is enough to build a useful rotation map.

### 3.5 Where Is It Moving?

This is the main question for the product.

The system must map movement across:

- Asset classes: stocks, bonds, commodities, cash proxies, currencies.
- Broad equity indices: SPY, QQQ, IWM, DIA.
- Sectors: technology, healthcare, financials, energy, etc.
- Industries/themes: semiconductors, cybersecurity, banks, oil services, biotech, homebuilders, defense, AI infrastructure, etc.
- Individual stocks.

The output should be a ranked map:

```text
Strongest asset class -> strongest index -> strongest sector -> strongest industry -> strongest stocks.
```

### 3.6 When Is It Moving?

Timing matters.

Different time horizons answer different questions:

- 1D: What is moving today?
- 5D: What is moving this week?
- 20D: What is rotating this month?
- 60D: What is in a sustained trend?
- 120D/252D: What is the long-term regime?

The system should support multiple timeframes because a sector can be weak long-term but strong short-term, or strong long-term but temporarily correcting.

## 4. What The Product Should Do

The product should be a daily and weekly decision support system.

### 4.1 Daily Use Case

At the start of the trading day or during market hours:

1. Check broad market regime.
2. See which sectors are leading today and this week.
3. See which industries inside those sectors are leading.
4. See the top stocks inside the leading industries.
5. Filter for liquidity and clean chart structure.
6. Export or display a "stocks worth charting" watchlist.

### 4.2 Weekly Use Case

At the end of each week:

1. Review sector rotation over 5D, 20D, 60D.
2. Identify sectors gaining strength.
3. Identify sectors losing strength.
4. Compare current rotation to macro regime.
5. Build a watchlist for the next week.
6. Review whether last week's watchlist produced useful names.

### 4.3 Long-Term Use Case

Over months:

1. Observe how sectors behave through macro cycles.
2. Track whether the system correctly identifies leadership changes.
3. Backtest whether the scores predict future relative returns.
4. Improve the model based on evidence, not opinion.

## 5. Product Output

The system should produce a small number of high-signal outputs.

### 5.1 Market Regime Dashboard

Shows whether the environment is:

- Risk-on.
- Risk-off.
- Defensive.
- Inflationary.
- Disinflationary.
- Rate-sensitive.
- Liquidity-driven.
- Unclear/mixed.

Inputs:

- SPY trend.
- QQQ vs SPY.
- IWM vs SPY.
- VIX trend.
- DXY trend.
- US10Y trend.
- TLT trend.
- GLD trend.
- Oil trend.
- Macro calendar and recent macro surprises.

### 5.2 Sector Rotation Dashboard

Shows all major sectors ranked by strength.

Each sector should show:

- 1D return.
- 5D return.
- 20D return.
- 60D return.
- Return vs SPY.
- Relative volume.
- Breadth.
- Trend state.
- Flow score.
- Change in rank.

### 5.3 Industry/Theme Dashboard

Inside each sector, show stronger groups.

Examples:

- Technology -> semiconductors, software, cybersecurity, hardware.
- Financials -> banks, insurers, asset managers, fintech.
- Energy -> oil producers, oil services, refiners.
- Healthcare -> biotech, pharma, medical devices, managed care.
- Consumer discretionary -> autos, restaurants, travel, retail.

Industry grouping is important because sectors can be too broad. For example, technology may be strong because semiconductors are strong while software is weak.

### 5.4 Stock Leadership Dashboard

For each strong sector or industry, show top stocks ranked by opportunity score.

Each stock should show:

- Sector.
- Industry.
- Price.
- 1D, 5D, 20D, 60D return.
- Return vs sector ETF.
- Return vs SPY.
- Relative volume.
- Dollar volume.
- Market cap.
- Trend state.
- Distance from 20/50/200-day moving averages.
- Earnings date.
- News/catalyst flag.
- Options activity flag if available.
- Watchlist rank.

### 5.5 Watchlist Output

The system should output:

```text
Top sectors today
Top industries today
Top 20-50 stocks worth charting
Stocks newly appearing in leadership
Stocks losing leadership
Stocks with catalyst + volume + sector strength
```

This is the most important user-facing output.

## 6. Data Sources

Data should be selected based on reliability, cost, API access, latency, and historical coverage.

### 6.1 Market Price Data

Needed for:

- Stocks.
- ETFs.
- Sector ETFs.
- Index ETFs.
- Intraday and daily OHLCV.
- Historical backtesting.

Possible providers:

- Polygon.io: stock market data, REST APIs, WebSocket streams, historical data. Official docs: https://polygon.io/docs/rest/stocks/overview/
- Alpaca Market Data API: market data through HTTP and WebSocket, with SDKs. Official docs: https://docs.alpaca.markets/us/v1.1/docs/about-market-data-api
- Databento: real-time and historical data, strong for serious market data workflows. Official site: https://databento.com/
- Finnhub: stock, fundamentals, alternative data, REST and WebSocket. Official site: https://www.finnhub.io/
- Intrinio: equities, options, fundamentals, real-time and historical data. Official site: https://intrinio.com/access-methods
- Nasdaq Data Link: financial and alternative datasets through API. Official site: https://www.nasdaq.com/solutions/data/nasdaq-data-link

MVP recommendation:

- Use one primary price data provider.
- Start with daily OHLCV and later add intraday.
- Polygon or Alpaca are practical starting points for a custom app.

### 6.2 Sector And ETF Data

Needed for sector rotation.

Main ETFs:

- XLC: Communication Services.
- XLY: Consumer Discretionary.
- XLP: Consumer Staples.
- XLE: Energy.
- XLF: Financials.
- XLV: Health Care.
- XLI: Industrials.
- XLB: Materials.
- XLRE: Real Estate.
- XLK: Technology.
- XLU: Utilities.

Official sector ETF reference:

- State Street Select Sector SPDRs: https://www.ssga.com/sectorspdr/sectors

MVP recommendation:

- Use sector ETF price and volume first.
- Add constituent mapping from a reliable source.

### 6.3 Macro Data

Needed for market regime.

Key series:

- CPI.
- Core CPI.
- PCE inflation.
- Unemployment rate.
- Nonfarm payrolls.
- GDP.
- ISM manufacturing/services.
- Fed funds rate.
- Fed balance sheet.
- Treasury yields.
- Yield curve.
- Credit spreads.

Sources:

- FRED: https://fred.stlouisfed.org/docs/api/fred/overview.html
- Federal Reserve data releases.
- Bureau of Labor Statistics.
- Bureau of Economic Analysis.
- Treasury data.

MVP recommendation:

- Use FRED for most macro series.
- Start with a small macro set: CPI, unemployment, Fed funds, 10Y yield, 2Y yield, yield curve, DXY, VIX.

### 6.4 ETF And Fund Flow Data

Useful for seeing capital allocation shifts.

Sources:

- ETF.com tools: https://www.etf.com/tools
- Polygon ETF fund flows: https://polygon.io/docs/rest/partners/etf-global/fundflows
- EPFR: https://epfr.com/
- LSEG Lipper fund data: https://www.lseg.com/en/data-analytics/financial-data/fund-data
- Morningstar Direct: https://www.morningstar.com/business/products/direct

MVP recommendation:

- Leave true fund flow data optional at first.
- Use ETF price/volume rotation first.
- Add ETF flow data once the basic rotation score is working.

### 6.5 Options Flow And Gamma Data

Useful for more advanced flow confirmation and intraday behavior.

Sources/tools:

- Unusual Whales: https://unusualwhales.com/features
- SpotGamma: https://support.spotgamma.com/hc/en-us/articles/36233401585683-What-is-SpotGamma-Tape
- FlowAlgo: https://www.flowalgo.com/
- BlackBoxStocks: https://blackboxstocks.com/features/
- Market Chameleon: https://marketchameleon.com/Reports/UnusualOptionVolumeReport
- Cboe LiveVol: https://datashop.cboe.com/livevol-pro

MVP recommendation:

- Do not depend on options data in version 1.
- Add a simple "options activity flag" later.
- Add gamma/0DTE logic only after the daily/weekly sector system works.

### 6.6 Positioning Data

Useful for futures, commodities, currencies, and index-level sentiment.

Source:

- CFTC Commitments of Traders reports: https://www.cftc.gov/MarketReports/CommitmentsofTraders/index.htm

MVP recommendation:

- Use COT later for macro/futures positioning.
- Not required for first stock-sector watchlist product.

### 6.7 Filings And Institutional Ownership

Useful for long-term positioning, but not for short-term flow.

Sources:

- SEC EDGAR filings: https://www.sec.gov/search-filings
- WhaleWisdom: https://whalewisdom.com/info/features
- OpenInsider: https://openinsider.com/charts

MVP recommendation:

- Leave out at first.
- Add later for long-term research pages.

### 6.8 News And Catalysts

Needed to understand why a move is happening.

Sources/tools:

- Benzinga Pro: https://www.benzinga.com/pro/
- Newsquawk: https://www.newsquawk.com/
- The Fly: https://app.thefly.com/
- SEC filings.
- Earnings calendars.
- Economic calendar.

MVP recommendation:

- Start with earnings date, major news headline, and macro calendar.
- Do not try to build a full news engine immediately.

## 7. Scoring Model

The system should use transparent scores. Every score must be explainable.

### 7.1 Market Regime Score

Purpose:

Determine whether the broad environment supports risk assets, defensive assets, or mixed positioning.

Example components:

```text
Market Regime Score =
  20% SPY trend
  15% QQQ vs SPY
  10% IWM vs SPY
  15% VIX trend
  10% US10Y direction
  10% DXY direction
  10% credit/rates pressure
  10% macro surprise/catalyst context
```

Interpretation:

- Positive: risk-on.
- Negative: risk-off.
- Near zero: mixed/unclear.

This score should not trade directly. It gives context.

### 7.2 Sector Flow Score

Purpose:

Rank sectors by where participation is concentrating.

Example:

```text
Sector Flow Score =
  25% relative strength vs SPY
  20% performance persistence across 5D/20D/60D
  20% relative volume
  15% breadth inside sector
  10% macro regime fit
  10% catalyst/options/fund-flow confirmation
```

Possible metrics:

- Sector return minus SPY return.
- Sector return percentile over all sectors.
- Sector volume vs 20-day average.
- Percent of sector constituents above 20-day moving average.
- Percent of sector constituents above 50-day moving average.
- Percent of sector constituents making 20-day highs.
- Rank change over time.

### 7.3 Industry Flow Score

Purpose:

Find the active groups inside strong sectors.

Example:

```text
Industry Flow Score =
  30% return vs sector
  20% return vs SPY
  20% breadth inside industry
  15% relative volume
  15% number of stocks making new highs or breakouts
```

This is important because sector-level signals can hide where the actual leadership is.

### 7.4 Stock Opportunity Score

Purpose:

Rank individual stocks worth charting.

Example:

```text
Stock Opportunity Score =
  30% sector flow score
  20% industry flow score
  20% relative strength vs sector
  15% relative volume and liquidity
  10% trend/chart structure
   5% catalyst/options confirmation
```

Filters before ranking:

- Minimum price.
- Minimum average dollar volume.
- Minimum market cap if desired.
- Exclude illiquid stocks.
- Exclude stocks with abnormal spreads if intraday data is available.
- Exclude low-quality data.

### 7.5 Catalyst Score

Purpose:

Determine whether a move has a reason.

Inputs:

- Earnings date.
- Earnings surprise.
- Guidance change.
- Analyst upgrade/downgrade.
- Major SEC filing.
- Product/regulatory news.
- Sector news.
- Macro event.
- Options activity.

MVP:

- Simple catalyst flag is enough.
- Do not need an NLP sentiment model at first.

## 8. Important Concepts And Definitions

### 8.1 Money Flow Is A Proxy

Every trade has a buyer and a seller. We cannot literally say "money entered" a stock from trade data alone.

What we can say:

- Price is rising relative to the market.
- Volume is above normal.
- Multiple stocks in the same group are moving.
- The sector ETF is outperforming.
- The move is persistent across timeframes.
- A catalyst exists.

That combination is a useful proxy for institutional participation.

### 8.2 Relative Strength Matters More Than Raw Return

A stock up 3% is not impressive if its sector is up 5%.

The system should constantly compare:

- Stock vs sector.
- Stock vs SPY.
- Sector vs SPY.
- Industry vs sector.

Relative strength identifies leaders.

### 8.3 Breadth Confirms Rotation

If XLK is up because only one mega-cap stock is moving, the sector may not have broad participation.

Breadth metrics:

- Percent of stocks in sector above 20-day moving average.
- Percent above 50-day moving average.
- Percent with positive 5D return.
- Percent with positive 20D return.
- Advance/decline ratio.
- New highs vs new lows.

### 8.4 Volume Confirms Participation

Price movement without volume is weaker.

Volume metrics:

- Current volume vs 20-day average.
- Dollar volume.
- Volume percentile.
- Breakout volume.
- Gap volume.
- Sector ETF volume vs average.

### 8.5 Macro Gives Context, Not Certainty

Macro analysis should explain the environment, but it should not paralyze the system.

Useful macro framing:

- Growth rising/falling.
- Inflation rising/falling.
- Rates rising/falling.
- Liquidity expanding/contracting.
- Risk appetite improving/weakening.

The system should support scenarios:

```text
Scenario A: inflation falling, growth stable, Fed easing -> risk-on/growth sectors favored.
Scenario B: inflation rising, rates rising -> energy/materials/defensives may outperform.
Scenario C: recession risk rising -> staples, healthcare, utilities, bonds may outperform.
Scenario D: credit/liquidity stress -> risk-off, volatility up, broad equity weakness.
```

## 9. MVP Scope

The first version should be simple enough to build and useful enough to trust gradually.

### 9.1 MVP Must Have

- Daily OHLCV data for SPY, QQQ, IWM, DIA.
- Daily OHLCV data for sector ETFs.
- Daily OHLCV data for S&P 500 or broad US stock universe.
- Sector and industry mapping for stocks.
- Sector ranking.
- Stock ranking.
- Watchlist generation.
- Historical backtest.
- Basic dashboard or report.

### 9.2 MVP Data Fields

For each ETF/stock:

- Symbol.
- Name.
- Sector.
- Industry.
- Date.
- Open, high, low, close.
- Volume.
- Average volume.
- Dollar volume.
- Market cap if available.
- Returns: 1D, 5D, 20D, 60D.
- Relative return vs SPY.
- Relative return vs sector.
- Moving averages: 20D, 50D, 200D.
- Trend state.
- Score.

### 9.3 MVP Outputs

Daily report:

```text
Market regime: risk-on / risk-off / mixed
Top sectors: ranked list
Weakest sectors: ranked list
Sector rank changes
Top industries inside top sectors
Top stocks worth charting
New leaders
Stocks with high relative volume
Stocks breaking out
```

### 9.4 MVP Interface

Can start as:

- CSV report.
- Markdown report.
- Notebook.
- Simple web dashboard.

Do not start with a polished terminal. First prove the signal.

## 10. What Can Be Left Out Initially

Leave these out until the basic system works:

- Dark pool data.
- Full Level 2/order book.
- 0DTE gamma modeling.
- Dealer hedging models.
- Complex options flow interpretation.
- AI prediction.
- Automated trade execution.
- Broker integration.
- Intraday scalping tools.
- Insider filings.
- 13F filings.
- Full NLP news analysis.
- Portfolio optimization.

Reason:

These can be useful later, but they add cost and complexity before the core question is solved:

```text
Can we identify active sectors and useful stocks to chart?
```

## 11. Advanced Features For Later

Once the MVP works, add:

### 11.1 ETF Fund Flow Layer

Purpose:

Confirm whether capital is actually flowing into sector/theme ETFs.

Use:

- ETF inflow/outflow data.
- AUM changes.
- Creation/redemption estimates.

### 11.2 Options Flow Layer

Purpose:

Confirm unusual speculative or hedging activity.

Use:

- Unusual options volume.
- Call/put premium.
- Sweep/block trades.
- Open interest changes.
- Implied volatility changes.
- Gamma exposure by index/stock if available.

### 11.3 News/Catalyst Layer

Purpose:

Explain why a stock or sector is moving.

Use:

- Earnings surprises.
- Guidance.
- Analyst actions.
- Regulatory/news events.
- M&A.
- Macro calendar.

### 11.4 Intraday Layer

Purpose:

Help with timing entries after the watchlist is created.

Use:

- Intraday relative volume.
- Opening range.
- VWAP.
- Gap behavior.
- Intraday sector leadership.
- Market internals.

### 11.5 Positioning Layer

Purpose:

Add futures and macro sentiment context.

Use:

- CFTC COT.
- Asset manager positioning.
- Leveraged fund positioning.
- Retail sentiment.
- Futures open interest.

## 12. Backtesting And Validation

This is mandatory. The system must be tested before being trusted.

### 12.1 Basic Backtest Question

When a sector has a high sector flow score, does it outperform SPY over the next:

- 1 day?
- 5 days?
- 10 days?
- 20 days?
- 60 days?

When a stock has a high opportunity score, does it outperform:

- SPY?
- Its sector ETF?
- Its industry group?

### 12.2 Labels

For each date and symbol, calculate:

- Forward 1D return.
- Forward 5D return.
- Forward 10D return.
- Forward 20D return.
- Forward 60D return.
- Forward return vs SPY.
- Forward return vs sector ETF.
- Maximum adverse excursion.
- Maximum favorable excursion.

### 12.3 Evaluation Metrics

Track:

- Average forward return by score decile.
- Hit rate.
- Median return.
- Volatility.
- Max drawdown.
- Sharpe/Sortino if portfolio simulation is added.
- Turnover.
- Transaction costs.
- Slippage.
- Sector concentration.
- False positives.

### 12.4 Avoiding Bad Backtests

The system must avoid:

- Lookahead bias.
- Survivorship bias.
- Using future sector membership.
- Ignoring delisted stocks if testing broad history.
- Overfitting weights.
- Optimizing too many parameters.
- Ignoring transaction costs.
- Testing only bull markets.

### 12.5 Walk-Forward Validation

Recommended process:

1. Build scores on historical data.
2. Test fixed weights over past periods.
3. Split data into training, validation, and test periods.
4. Walk forward by year or quarter.
5. Check whether results survive different regimes.
6. Paper trade/watchlist test before using with real money.

## 13. Risk And Trading Rules

The system should not create trades by itself in the first version.

It creates attention.

Trading decisions still need:

- Chart setup.
- Entry trigger.
- Stop-loss.
- Position sizing.
- Risk per trade.
- Invalidating condition.
- Market context.
- Earnings/catalyst risk check.

Suggested rule:

```text
No trade should be taken only because a stock is highly ranked.
The score only says the stock deserves attention.
The chart and risk plan decide the trade.
```

## 14. Example Daily Workflow

Before market open:

1. Read macro calendar.
2. Check futures/index ETFs.
3. Check VIX, yields, DXY, oil, gold.
4. Review sector flow rankings.
5. Review industries gaining strength.
6. Generate top stock watchlist.
7. Remove stocks with bad liquidity.
8. Flag earnings/news risk.
9. Chart only the best names.

During market:

1. Check whether premarket sector strength persists.
2. Watch top sectors vs SPY.
3. Watch top stocks vs sector ETF.
4. Avoid stocks losing relative strength.
5. Update watchlist if new volume appears.

After market close:

1. Save daily scores.
2. Compare ranked names vs actual movement.
3. Note false positives.
4. Update backtest dataset.
5. Prepare tomorrow's watchlist.

## 15. Example Scenarios

### Scenario 1: Risk-On Growth Rotation

Signals:

- SPY above 50D and 200D.
- QQQ outperforming SPY.
- VIX falling.
- Yields stable or falling.
- XLK, XLC, XLY outperforming.
- Breadth expanding inside growth sectors.

Likely watchlist:

- Semiconductors.
- Software.
- Internet.
- Consumer discretionary leaders.

### Scenario 2: Defensive Rotation

Signals:

- SPY weakening.
- VIX rising.
- QQQ underperforming.
- XLP, XLV, XLU outperforming.
- Growth sectors losing breadth.

Likely watchlist:

- Healthcare.
- Staples.
- Utilities.
- Low volatility names.

### Scenario 3: Inflation/Energy Rotation

Signals:

- Oil rising.
- Inflation expectations rising.
- XLE outperforming.
- Materials outperforming.
- Long-duration growth under pressure.

Likely watchlist:

- Energy producers.
- Oil services.
- Materials.
- Commodity-linked stocks.

### Scenario 4: Small-Cap Risk Appetite

Signals:

- IWM outperforming SPY.
- Credit stress falling.
- Yields supportive.
- Breadth improving.

Likely watchlist:

- Small-cap leaders with liquidity.
- Regional banks if financials confirm.
- Industrials and cyclicals.

## 16. Data Architecture

### 16.1 Tables

Recommended core tables:

```text
symbols
prices_daily
prices_intraday_later
sector_map
industry_map
macro_series
sector_scores
industry_scores
stock_scores
watchlists
events
backtest_results
```

### 16.2 Symbol Table

Fields:

```text
symbol
name
asset_type
sector
industry
exchange
market_cap
is_active
first_seen_date
last_seen_date
```

### 16.3 Daily Price Table

Fields:

```text
symbol
date
open
high
low
close
adjusted_close
volume
vwap_optional
source
```

### 16.4 Score Tables

Fields:

```text
symbol_or_group
date
score_type
score
rank
components_json
```

Every score should store components so we can explain why something ranked high.

## 17. Build Phases

### Phase 0: Research And Spec

Current phase.

Output:

- This document.
- Decisions on MVP data provider.
- Decision on stock universe.
- Decision on dashboard/report format.

### Phase 1: Data Foundation

Build:

- Download daily prices.
- Store prices.
- Build sector/industry mapping.
- Calculate returns and relative strength.
- Generate sector ranking.

Success:

- We can rank sectors historically and today.

### Phase 2: Stock Ranking

Build:

- Rank stocks inside sectors.
- Add liquidity filters.
- Add relative volume.
- Add trend state.
- Generate daily watchlist.

Success:

- We get a useful list of stocks to chart every day.

### Phase 3: Backtesting

Build:

- Historical score calculation.
- Forward return labels.
- Score decile analysis.
- Sector and stock performance tests.

Success:

- We know whether the scoring method has predictive usefulness.

### Phase 4: Dashboard

Build:

- Market regime page.
- Sector rotation page.
- Industry page.
- Stock leadership page.
- Watchlist page.

Success:

- Daily workflow is visual and easy to use.

### Phase 5: Advanced Data

Add:

- ETF fund flows.
- Options activity.
- News/catalyst data.
- Intraday data.
- Alerts.

Success:

- The system becomes more context-aware without losing explainability.

## 18. Key Design Principles

### 18.1 Explainability First

Every score must be explainable.

Bad:

```text
AI says this stock is bullish.
```

Good:

```text
This stock ranks #3 because its sector is #1, its industry is #2, it is outperforming its sector by 6.4% over 20D, and volume is 2.1x normal.
```

### 18.2 Watchlist, Not Signal

The system should create watchlists, not automatic trades.

### 18.3 Top-Down First

Do not begin with individual stocks.

Start with:

```text
Market -> sector -> industry -> stock.
```

### 18.4 Evidence Over Opinion

Any rule added to the system should be backtested or reviewed with historical examples.

### 18.5 Simple Before Complex

Use price, volume, relative strength, breadth, and sector mapping first.

Add options/dark pool/gamma later only if they improve decision quality.

## 19. What Is Major Vs Optional

### Major

- Daily price data.
- Sector ETF data.
- Sector membership.
- Relative strength.
- Relative volume.
- Breadth.
- Liquidity filters.
- Scoring.
- Watchlist generation.
- Backtesting.

### Useful But Second Priority

- Macro data.
- Earnings calendar.
- News/catalyst feed.
- ETF fund flows.
- Industry/theme mapping.
- Alerts.

### Advanced

- Options flow.
- Gamma exposure.
- Dark pool prints.
- Intraday order flow.
- COT positioning.
- Insider/13F filings.
- ML/AI scoring.
- Automated execution.

## 20. Practical First Version

The first useful version can be:

```text
One script or app that runs daily and produces:

1. Market regime summary.
2. Sector ranking table.
3. Industry ranking table.
4. Top 50 stocks worth charting.
5. Explanation for each ranked stock.
6. CSV/Markdown output.
7. Historical score storage for backtesting.
```

If this version works, everything else can be layered on top.

## 21. Example Output Format

```text
Date: 2026-05-26
Market regime: Risk-on, but rate-sensitive

Top sectors:
1. XLK Technology - Flow Score 87
2. XLC Communication Services - Flow Score 81
3. XLI Industrials - Flow Score 73

Weak sectors:
1. XLU Utilities - Flow Score 32
2. XLP Staples - Flow Score 35

Top industry groups:
1. Semiconductors
2. Software infrastructure
3. Aerospace/defense

Top stocks to chart:
1. NVDA - Sector strong, industry strong, high relative volume, outperforming XLK
2. AVGO - Sector strong, industry strong, above 20D/50D/200D
3. AMD - Relative strength improving, volume above average
4. MU - Breakout candidate, semiconductor group confirmation

Notes:
- Avoid names reporting earnings within 24 hours unless specifically trading earnings risk.
- Confirm chart structure before any trade.
```

## 22. Questions To Decide Before Building

Before implementation, decide:

1. Which market universe first?
   - S&P 500 only.
   - Russell 1000.
   - All US stocks above liquidity threshold.

2. Which data provider first?
   - Polygon.
   - Alpaca.
   - Finnhub.
   - Other.

3. What timeframe first?
   - Daily only.
   - Daily + 15-minute.
   - Daily + 5-minute.

4. What output first?
   - Markdown report.
   - CSV files.
   - Notebook.
   - Web dashboard.

5. What is the primary use?
   - Swing trading watchlist.
   - Day trading prep.
   - Long-term sector allocation.
   - All of the above, with separate views.

Recommended answers:

```text
Universe: S&P 500 first.
Data: one reliable OHLCV API.
Timeframe: daily first.
Output: Markdown + CSV first.
Use: swing/day prep watchlist, not automatic trade execution.
```

## 23. Final Product Vision

The mature system should become a personal market intelligence terminal.

It should tell us:

- What regime the market is in.
- Which sectors are gaining money/attention.
- Which sectors are losing money/attention.
- Which industries are leading.
- Which stocks are leading inside those industries.
- Which names are liquid and chart-worthy.
- Which moves have catalysts.
- Which signals have historically worked.
- Which signals should be ignored.

The system should make the user faster, more selective, and less random.

It should replace:

```text
Let me look at random charts and hope something is moving.
```

With:

```text
Here are the 30 names where participation, sector strength, and relative performance are aligned. Chart these first.
```

That is the real value.

## 24. Source References

Market data and APIs:

- Polygon stock docs: https://polygon.io/docs/rest/stocks/overview/
- Alpaca market data docs: https://docs.alpaca.markets/us/v1.1/docs/about-market-data-api
- Databento: https://databento.com/
- Finnhub: https://www.finnhub.io/
- Intrinio access methods: https://intrinio.com/access-methods
- Nasdaq Data Link: https://www.nasdaq.com/solutions/data/nasdaq-data-link

Sector ETF reference:

- State Street Select Sector SPDRs: https://www.ssga.com/sectorspdr/sectors

Macro data:

- FRED API overview: https://fred.stlouisfed.org/docs/api/fred/overview.html
- FRED description: https://fredhelp.stlouisfed.org/fred/about/about-fred/what-is-fred/

Fund flow and ETF flow:

- ETF.com tools: https://www.etf.com/tools
- Polygon ETF fund flows: https://polygon.io/docs/rest/partners/etf-global/fundflows
- EPFR: https://epfr.com/
- LSEG Lipper fund data: https://www.lseg.com/en/data-analytics/financial-data/fund-data
- Morningstar Direct: https://www.morningstar.com/business/products/direct

Options and flow tools:

- Unusual Whales features: https://unusualwhales.com/features
- SpotGamma Tape overview: https://support.spotgamma.com/hc/en-us/articles/36233401585683-What-is-SpotGamma-Tape
- FlowAlgo: https://www.flowalgo.com/
- BlackBoxStocks: https://blackboxstocks.com/features/
- Market Chameleon unusual options: https://marketchameleon.com/Reports/UnusualOptionVolumeReport
- Cboe LiveVol: https://datashop.cboe.com/livevol-pro

Positioning and filings:

- CFTC Commitments of Traders: https://www.cftc.gov/MarketReports/CommitmentsofTraders/index.htm
- SEC EDGAR search: https://www.sec.gov/search-filings
- WhaleWisdom features: https://whalewisdom.com/info/features
- OpenInsider charts: https://openinsider.com/charts
