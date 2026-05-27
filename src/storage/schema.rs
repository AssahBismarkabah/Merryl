use anyhow::Result;

use super::sqlite::Database;

const MIGRATION_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS symbols (
    symbol TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    sector TEXT,
    industry TEXT,
    exchange TEXT NOT NULL,
    market_cap REAL,
    is_active INTEGER NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sector_map (
    sector TEXT PRIMARY KEY,
    sector_etf TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS industry_map (
    industry TEXT NOT NULL,
    sector TEXT NOT NULL,
    description TEXT NOT NULL,
    PRIMARY KEY (industry, sector)
);

CREATE TABLE IF NOT EXISTS prices_daily (
    symbol TEXT NOT NULL,
    date TEXT NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    adjusted_close REAL NOT NULL,
    volume REAL NOT NULL,
    source TEXT NOT NULL,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (symbol, date)
);

CREATE INDEX IF NOT EXISTS idx_prices_daily_date
    ON prices_daily(date);

CREATE TABLE IF NOT EXISTS prices_intraday (
    symbol TEXT NOT NULL,
    ts TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume REAL NOT NULL,
    source TEXT NOT NULL,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (symbol, ts, timeframe)
);

CREATE TABLE IF NOT EXISTS macro_series (
    series TEXT NOT NULL,
    date TEXT NOT NULL,
    value REAL NOT NULL,
    source TEXT NOT NULL,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (series, date)
);

CREATE TABLE IF NOT EXISTS market_regime_scores (
    date TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    score REAL NOT NULL,
    spy_return_20d REAL NOT NULL,
    spy_return_60d REAL NOT NULL,
    qqq_relative_return_vs_spy REAL NOT NULL,
    iwm_relative_return_vs_spy REAL NOT NULL,
    dia_relative_return_vs_spy REAL NOT NULL,
    components_json TEXT NOT NULL,
    explanation TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT,
    sector TEXT,
    event_date TEXT NOT NULL,
    event_type TEXT NOT NULL,
    headline TEXT NOT NULL,
    source TEXT NOT NULL,
    url TEXT,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS filings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    filing_date TEXT NOT NULL,
    filing_type TEXT NOT NULL,
    source TEXT NOT NULL,
    url TEXT,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sector_scores (
    date TEXT NOT NULL,
    sector TEXT NOT NULL,
    sector_etf TEXT NOT NULL,
    score REAL NOT NULL,
    rank INTEGER NOT NULL,
    return_1d REAL NOT NULL,
    return_5d REAL NOT NULL,
    return_20d REAL NOT NULL,
    return_60d REAL NOT NULL,
    relative_return_vs_spy REAL NOT NULL,
    relative_volume REAL NOT NULL,
    breadth_20d REAL NOT NULL,
    breadth_50d REAL NOT NULL,
    rank_change REAL NOT NULL,
    components_json TEXT NOT NULL,
    explanation TEXT NOT NULL,
    PRIMARY KEY (date, sector)
);

CREATE TABLE IF NOT EXISTS industry_scores (
    date TEXT NOT NULL,
    industry TEXT NOT NULL,
    sector TEXT NOT NULL,
    score REAL NOT NULL,
    rank INTEGER NOT NULL,
    components_json TEXT NOT NULL,
    PRIMARY KEY (date, industry, sector)
);

CREATE TABLE IF NOT EXISTS stock_scores (
    date TEXT NOT NULL,
    rank INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    sector TEXT NOT NULL,
    industry TEXT NOT NULL,
    score REAL NOT NULL,
    sector_score REAL NOT NULL,
    return_1d REAL NOT NULL,
    return_5d REAL NOT NULL,
    return_20d REAL NOT NULL,
    return_60d REAL NOT NULL,
    relative_return_vs_sector REAL NOT NULL,
    relative_return_vs_spy REAL NOT NULL,
    relative_volume REAL NOT NULL,
    avg_dollar_volume REAL NOT NULL,
    trend_state TEXT NOT NULL,
    catalyst_status TEXT NOT NULL,
    explanation TEXT NOT NULL,
    PRIMARY KEY (date, symbol)
);

CREATE TABLE IF NOT EXISTS watchlists (
    date TEXT NOT NULL,
    rank INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    score REAL NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (date, symbol)
);

CREATE TABLE IF NOT EXISTS backtest_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_name TEXT NOT NULL,
    from_date TEXT NOT NULL,
    to_date TEXT NOT NULL,
    config_json TEXT NOT NULL,
    metrics_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

impl Database {
    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(MIGRATION_SQL)?;
        Ok(())
    }
}
