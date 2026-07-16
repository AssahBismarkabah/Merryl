# Data Sources

Merryl uses real market data sources only. It does not generate fake production candles.

## Alpaca Market Data

**Purpose:** Daily OHLCV bars and recent news.

- **Documentation:** [Alpaca Market Data FAQ](https://docs.alpaca.markets/docs/market-data-faq)
- **API Type:** REST
- **Auth:** API Key ID + Secret Key
- **Rate Limit:** Configurable via `MERRYL_ALPACA_REQUESTS_PER_MINUTE` (default: 180)
- **Data:** 1Day bars with full adjustment, news headlines with relevance scores

**Configuration:**

| Env Variable | Default | Description |
|---|---|---|
| `ALPACA_API_KEY_ID` | (required) | API Key ID |
| `ALPACA_API_SECRET_KEY` | (required) | Secret Key |
| `ALPACA_FEED` | `iex` | Feed type (`iex` or `sip`) |
| `ALPACA_DATA_URL` | `https://data.alpaca.markets` | Base URL |

**Usage in Merryl:**

- Fetches daily bars for the full universe (S&P 500 + ETFs) with `MERRYL_LOOKBACK_DAYS` of history
- Fetches recent news for watchlist stocks with a 7-day lookback
- Supports batch requests (25 symbols per batch) with configurable rate limiting
- Retries failed requests once with a 1-second delay

## FRED (Federal Reserve Economic Data)

**Purpose:** Macro context: rates, volatility, inflation, employment, credit, and liquidity.

- **Documentation:** [FRED API](https://fred.stlouisfed.org/docs/api/fred/)
- **API Type:** REST
- **Auth:** API key (free)
- **Data:** JSON time series observations

**Tracked Series:**

| Series ID | Name | Frequency | Unit |
|---|---|---|---|
| `VIXCLS` | CBOE Volatility Index: VIX | Daily | Index |
| `DGS10` | 10-Year Treasury Rate | Daily | Percent |
| `DGS2` | 2-Year Treasury Rate | Daily | Percent |
| `T10Y2Y` | 10Y-2Y Treasury Spread | Daily | Percent |
| `DFF` | Effective Federal Funds Rate | Daily | Percent |
| `CPIAUCSL` | Consumer Price Index | Monthly | Index |
| `UNRATE` | Unemployment Rate | Monthly | Percent |
| `PAYEMS` | Total Nonfarm Employment | Monthly | Thousands |
| `BAMLC0A0CM` | Corp Bond OAS Spread | Daily | Percent |
| `DTWEXBGS` | Nominal Broad USD Index | Daily | Index |
| `WALCL` | Fed Total Assets | Weekly | Millions USD |

**Configuration:**

| Env Variable | Default | Description |
|---|---|---|
| `FRED_API_KEY` | (required) | FRED API key |
| `FRED_API_URL` | `https://api.stlouisfed.org` | Base URL |
| `MERRYL_MACRO_LOOKBACK_DAYS` | `900` | Lookback in calendar days |

**Usage in Merryl:**

- Stored as context/provenance only -- macro observations are not scoring inputs yet
- Validated for freshness (daily: 7 days, weekly: 14 days, monthly: 45 days)
- Flags: volatility stress, rate pressure, yield curve inversion, credit stress, dollar pressure, liquidity tightening, inflation pressure, labor cooling

## Alpha Vantage

**Purpose:** Structured earnings calendar context.

- **Documentation:** [Alpha Vantage API](https://www.alphavantage.co/documentation/)
- **API Type:** REST
- **Auth:** API key (free)
- **Endpoint:** `EARNINGS_CALENDAR` function

**Configuration:**

| Env Variable | Default | Description |
|---|---|---|
| `ALPHA_VANTAGE_API_KEY` | (required) | API key |
| `ALPHA_VANTAGE_API_URL` | `https://www.alphavantage.co` | Base URL |
| `MERRYL_EARNINGS_CALENDAR_HORIZON` | `3month` | Lookahead horizon |

**Usage in Merryl:**

- Fetches earnings calendar for the full universe
- Flags watchlist stocks with upcoming earnings as `event_context` or `event_risk`
- Stored as context flags, not scoring inputs

## SEC EDGAR

**Purpose:** Recent SEC filing events (8-K, 10-Q, 10-K).

- **Documentation:** [SEC EDGAR APIs](https://www.sec.gov/search-filings/edgar-application-programming-interfaces)
- **API Type:** REST
- **Auth:** User agent with contact information (no API key)
- **Rate Limit:** 10 requests per second (with 120ms sleep between requests)

**Configuration:**

| Env Variable | Default | Description |
|---|---|---|
| `MERRYL_SEC_USER_AGENT` | (required) | SEC-compliant user agent |
| `MERRYL_SEC_FILINGS_LOOKBACK_DAYS` | `14` | Lookback in calendar days |

**Usage in Merryl:**

- Resolves CIK numbers from the SEC company tickers index
- Fetches recent submissions for watchlist stocks
- Filters for target forms: 8-K, 10-Q, 10-K
- Flags stocks with recent filings as `event_context`

## Universe Data

**Purpose:** Define the set of stocks and ETFs to track.

**Sources:**

| Source | Type | Description |
|---|---|---|
| Wikipedia | Dynamic | S&P 500 constituent list fetched at runtime |
| `config/sector_etfs.toml` | Static | 11 sector ETFs (XLC, XLY, XLP, XLE, XLF, XLV, XLI, XLB, XLRE, XLK, XLU) |
| `config/universe_sp500.toml` | Static | Universe configuration and normalization rules |

**Broad Market ETFs:** SPY, QQQ, IWM, DIA

**Macro ETFs:** TLT (bonds), GLD (gold), USO (oil)

**Guardrails:**

- First implementation universe is S&P 500 only -- not a permanent product boundary
- Liquidity rule: keep highly liquid stocks first, then expand to Russell 1000/3000 or all liquid US stocks
