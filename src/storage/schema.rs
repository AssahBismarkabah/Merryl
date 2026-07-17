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
    vwap REAL,
    source TEXT NOT NULL,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (symbol, ts, timeframe)
);

CREATE TABLE IF NOT EXISTS macro_series (
    series TEXT NOT NULL,
    date TEXT NOT NULL,
    value REAL NOT NULL,
    source TEXT NOT NULL,
    series_name TEXT NOT NULL DEFAULT '',
    frequency TEXT NOT NULL DEFAULT '',
    units TEXT NOT NULL DEFAULT '',
    realtime_start TEXT NOT NULL DEFAULT '',
    realtime_end TEXT NOT NULL DEFAULT '',
    raw_json TEXT NOT NULL DEFAULT '{}',
    quality_status TEXT NOT NULL DEFAULT 'ok',
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
    event_time TEXT,
    event_type TEXT NOT NULL,
    headline TEXT NOT NULL,
    source TEXT NOT NULL,
    url TEXT,
    source_event_id TEXT,
    effective_date TEXT,
    processed_at TEXT,
    fetched_at TEXT,
    actual REAL,
    estimate REAL,
    surprise REAL,
    fiscal_period TEXT,
    raw_json TEXT NOT NULL DEFAULT '{}',
    quality_status TEXT NOT NULL DEFAULT 'ok',
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
    components_json TEXT NOT NULL,
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

CREATE TABLE IF NOT EXISTS volume_profiles (
    symbol TEXT NOT NULL,
    date TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    poc REAL NOT NULL,
    vah REAL NOT NULL,
    val REAL NOT NULL,
    vwap REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    total_volume REAL NOT NULL,
    source TEXT NOT NULL,
    components_json TEXT NOT NULL,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (symbol, date, timeframe)
);

CREATE TABLE IF NOT EXISTS intraday_setups (
    date TEXT NOT NULL,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    sector TEXT NOT NULL,
    industry TEXT NOT NULL,
    direction TEXT NOT NULL,
    stage1_passed INTEGER NOT NULL,
    stage2_passed INTEGER NOT NULL,
    stage3_passed INTEGER NOT NULL,
    primary_label TEXT NOT NULL,
    adr_pct REAL NOT NULL,
    rvol_ratio REAL NOT NULL,
    mansfield_rs_spy REAL NOT NULL,
    mansfield_rs_sector REAL NOT NULL,
    ema_10 REAL NOT NULL,
    ema_20 REAL NOT NULL,
    latest_price REAL NOT NULL,
    confluence_count INTEGER NOT NULL,
    confluence_json TEXT NOT NULL,
    trigger_count INTEGER NOT NULL,
    components_json TEXT NOT NULL,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (date, symbol)
);

CREATE TABLE IF NOT EXISTS intraday_triggers (
    date TEXT NOT NULL,
    symbol TEXT NOT NULL,
    ts TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    trigger_price REAL NOT NULL,
    reference_level REAL NOT NULL,
    volume_spike REAL NOT NULL,
    price_action TEXT NOT NULL,
    components_json TEXT NOT NULL,
    source TEXT NOT NULL,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (date, symbol, ts, trigger_type)
);

CREATE TABLE IF NOT EXISTS screener_cache (
    sector TEXT NOT NULL,
    ticker TEXT NOT NULL,
    company TEXT NOT NULL,
    industry TEXT NOT NULL,
    market_cap TEXT NOT NULL,
    pe_ratio TEXT NOT NULL,
    price TEXT NOT NULL,
    change TEXT NOT NULL,
    volume TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (sector, ticker)
);
"#;

impl Database {
    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(MIGRATION_SQL)?;
        self.add_column_if_missing(
            "stock_scores",
            "components_json",
            "ALTER TABLE stock_scores ADD COLUMN components_json TEXT NOT NULL DEFAULT '{}'",
        )?;
        self.add_column_if_missing(
            "prices_intraday",
            "vwap",
            "ALTER TABLE prices_intraday ADD COLUMN vwap REAL",
        )?;
        self.add_column_if_missing(
            "macro_series",
            "series_name",
            "ALTER TABLE macro_series ADD COLUMN series_name TEXT NOT NULL DEFAULT ''",
        )?;
        self.add_column_if_missing(
            "macro_series",
            "frequency",
            "ALTER TABLE macro_series ADD COLUMN frequency TEXT NOT NULL DEFAULT ''",
        )?;
        self.add_column_if_missing(
            "macro_series",
            "units",
            "ALTER TABLE macro_series ADD COLUMN units TEXT NOT NULL DEFAULT ''",
        )?;
        self.add_column_if_missing(
            "macro_series",
            "realtime_start",
            "ALTER TABLE macro_series ADD COLUMN realtime_start TEXT NOT NULL DEFAULT ''",
        )?;
        self.add_column_if_missing(
            "macro_series",
            "realtime_end",
            "ALTER TABLE macro_series ADD COLUMN realtime_end TEXT NOT NULL DEFAULT ''",
        )?;
        self.add_column_if_missing(
            "macro_series",
            "raw_json",
            "ALTER TABLE macro_series ADD COLUMN raw_json TEXT NOT NULL DEFAULT '{}'",
        )?;
        self.add_column_if_missing(
            "macro_series",
            "quality_status",
            "ALTER TABLE macro_series ADD COLUMN quality_status TEXT NOT NULL DEFAULT 'ok'",
        )?;
        self.add_column_if_missing(
            "events",
            "event_time",
            "ALTER TABLE events ADD COLUMN event_time TEXT",
        )?;
        self.add_column_if_missing(
            "events",
            "source_event_id",
            "ALTER TABLE events ADD COLUMN source_event_id TEXT",
        )?;
        self.add_column_if_missing(
            "events",
            "effective_date",
            "ALTER TABLE events ADD COLUMN effective_date TEXT",
        )?;
        self.add_column_if_missing(
            "events",
            "processed_at",
            "ALTER TABLE events ADD COLUMN processed_at TEXT",
        )?;
        self.add_column_if_missing(
            "events",
            "fetched_at",
            "ALTER TABLE events ADD COLUMN fetched_at TEXT",
        )?;
        self.add_column_if_missing(
            "events",
            "actual",
            "ALTER TABLE events ADD COLUMN actual REAL",
        )?;
        self.add_column_if_missing(
            "events",
            "estimate",
            "ALTER TABLE events ADD COLUMN estimate REAL",
        )?;
        self.add_column_if_missing(
            "events",
            "surprise",
            "ALTER TABLE events ADD COLUMN surprise REAL",
        )?;
        self.add_column_if_missing(
            "events",
            "fiscal_period",
            "ALTER TABLE events ADD COLUMN fiscal_period TEXT",
        )?;
        self.add_column_if_missing(
            "events",
            "raw_json",
            "ALTER TABLE events ADD COLUMN raw_json TEXT NOT NULL DEFAULT '{}'",
        )?;
        self.add_column_if_missing(
            "events",
            "quality_status",
            "ALTER TABLE events ADD COLUMN quality_status TEXT NOT NULL DEFAULT 'ok'",
        )?;
        self.conn.execute_batch(
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS idx_events_source_event_id
                ON events(source, source_event_id)
                WHERE source_event_id IS NOT NULL AND source_event_id != '';
            "#,
        )?;
        Ok(())
    }

    fn add_column_if_missing(&self, table: &str, column: &str, sql: &str) -> Result<()> {
        if !self.column_exists(table, column)? {
            self.conn.execute_batch(sql)?;
        }
        Ok(())
    }

    fn column_exists(&self, table: &str, column: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

        for row in rows {
            if row? == column {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
