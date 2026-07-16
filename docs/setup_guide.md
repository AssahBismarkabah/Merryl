# Setup Guide

## Prerequisites

| Dependency | Minimum Version | Purpose |
|---|---|---|
| [Rust](https://www.rust-lang.org/tools/install) | 1.85+ (edition 2024) | Core engine compilation |
| [Node.js](https://nodejs.org/) | 22+ | Dashboard frontend build |
| [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) | ships with Node.js | Dashboard dependency management |

### Installing Rust

```bash
# Recommended: install via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version   # should show 1.85 or later
cargo --version
```

Rust edition 2024 is required. If your toolchain is older, update it:

```bash
rustup update stable
```

### Installing Node.js

Download and install from [nodejs.org](https://nodejs.org/) (version 22 LTS or later), or use a version manager:

```bash
# macOS via Homebrew
brew install node@22

# Verify
node --version
npm --version
```

## API Keys

Merryl uses real market data sources. You need API keys for the following services:

### Alpaca Market Data (Required)

Alpaca provides daily OHLCV bars and recent news.

1. Sign up at [alpaca.markets](https://alpaca.markets/) (free tier works)
2. Navigate to the dashboard and generate an API Key ID + Secret Key pair
3. Use the standard API Key pair, not `client_secret`/`private_key_jwt` credentials

### FRED API Key (Required for Phase 5 macro/regime context)

FRED (Federal Reserve Economic Data) provides macro context: rates, volatility, inflation, employment, credit, and liquidity series.

1. Register at [fred.stlouisfed.org/docs/api/fred/](https://fred.stlouisfed.org/docs/api/fred/)
2. Request a free API key from the API key page
3. Keys are typically issued immediately

### Alpha Vantage API Key (Required for Phase 5C earnings calendar)

Alpha Vantage provides structured earnings calendar data.

1. Register at [alphavantage.co/support/#api-key](https://www.alphavantage.co/support/#api-key)
2. Request a free API key
3. Keys are typically issued immediately

### SEC EDGAR (Required for Phase 5C filing context)

SEC EDGAR does not require an API key, but it requires a compliant user agent string with contact information.

```text
# Format
Merryl/0.1 your-email@example.com
```

See the [SEC EDGAR API documentation](https://www.sec.gov/search-filings/edgar-application-programming-interfaces) for rate-limit and user-agent requirements.

## Environment Configuration

### 1. Create the .env file

```bash
cp .env.example .env
```

### 2. Fill in required values

```bash
# Required
ALPACA_API_KEY_ID=pk_xxxxxxxxxxxxxxxx
ALPACA_API_SECRET_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
FRED_API_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
ALPHA_VANTAGE_API_KEY=xxxxxxxxxx
MERRYL_SEC_USER_AGENT=Merryl/0.1 your-email@example.com
```

### 3. Optional overrides

| Variable | Default | Description |
|---|---|---|
| `ALPACA_FEED` | `iex` | Market data feed (`iex` or `sip`) |
| `ALPACA_DATA_URL` | `https://data.alpaca.markets` | Alpaca data base URL |
| `FRED_API_URL` | `https://api.stlouisfed.org` | FRED API base URL |
| `ALPHA_VANTAGE_API_URL` | `https://www.alphavantage.co` | Alpha Vantage base URL |
| `MERRYL_EARNINGS_CALENDAR_HORIZON` | `3month` | Earnings calendar lookahead |
| `MERRYL_SEC_FILINGS_LOOKBACK_DAYS` | `14` | Recent SEC filing lookback |
| `MERRYL_LOOKBACK_DAYS` | `420` | Calendar-day lookback for daily bars |
| `MERRYL_MACRO_LOOKBACK_DAYS` | `900` | Calendar-day lookback for FRED series |
| `MERRYL_ALPACA_REQUESTS_PER_MINUTE` | `180` | Alpaca rate limit |
| `MERRYL_INTRADAY_PROFILE_TIMEFRAME` | `30Min` | Intraday profile timeframe |
| `MERRYL_INTRADAY_TRIGGER_TIMEFRAME` | `5Min` | Intraday trigger timeframe |
| `MERRYL_INTRADAY_CANDIDATE_LIMIT` | `50` | Intraday candidate limit |
| `MERRYL_INTRADAY_OPENING_RANGE_MINUTES` | `30` | Intraday opening range window |

## Dashboard Frontend

The dashboard is a React + Vite application. Install dependencies and build it:

```bash
# Install dependencies
npm --prefix dashboard install

# Development server (hot reload)
npm --prefix dashboard run dev

# Production build
npm --prefix dashboard run build
```

## Database

Create or migrate the local SQLite database:

```bash
cargo run -- db migrate
```

This creates `data/market.db` with all required tables.

## Verify Setup

Run the doctor and status commands to verify everything is configured correctly:

```bash
set -a; source .env; set +a
cargo run -- doctor
cargo run -- status
```

## First Daily Run

```bash
set -a; source .env; set +a
cargo run -- run daily --date latest
```

If this completes without errors, the setup is ready. See the [CLI reference](cli_reference.md) for all available commands.
