use anyhow::Result;
use rusqlite::Connection;
use tempfile::tempdir;

use merryl::domain::models::{
    IndustryScore, IntradayPrice, IntradaySetup, IntradayTrigger, MacroObservation, MarketEvent,
    MarketEventMetadata, MarketRegimeScore, SectorScore, StockScore, Symbol, VolumeProfile,
};
use merryl::storage::Database;

#[test]
fn stock_score_components_column_migration_is_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let db = Database::open(&db_path)?;

    db.migrate()?;
    db.migrate()?;

    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("PRAGMA table_info(stock_scores)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    assert!(columns.iter().any(|column| column == "components_json"));

    Ok(())
}

#[test]
fn macro_series_provenance_columns_migration_is_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let db = Database::open(&db_path)?;

    db.migrate()?;
    db.migrate()?;

    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("PRAGMA table_info(macro_series)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    for expected in [
        "series_name",
        "frequency",
        "units",
        "realtime_start",
        "realtime_end",
        "raw_json",
        "quality_status",
    ] {
        assert!(
            columns.iter().any(|column| column == expected),
            "expected macro_series column {expected}, got {columns:#?}"
        );
    }

    Ok(())
}

#[test]
fn macro_observation_upserts_are_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    db.upsert_macro_observations(&[macro_observation("2026-05-27", 18.3)])?;
    db.upsert_macro_observations(&[macro_observation("2026-05-27", 19.4)])?;
    drop(db);

    let conn = Connection::open(db_path)?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM macro_series", [], |row| row.get(0))?;
    let value: f64 = conn.query_row(
        "SELECT value FROM macro_series WHERE series = 'VIXCLS' AND date = '2026-05-27'",
        [],
        |row| row.get(0),
    )?;
    let source: String = conn.query_row(
        "SELECT source FROM macro_series WHERE series = 'VIXCLS' AND date = '2026-05-27'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(count, 1);
    assert_eq!(value, 19.4);
    assert_eq!(source, "fred:VIXCLS");

    Ok(())
}

#[test]
fn active_symbols_can_restore_cached_universe() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    db.upsert_symbols(&[
        symbol("MSFT", "Microsoft Corporation", "stock", true),
        symbol("SPY", "SPDR S&P 500 ETF Trust", "broad_etf", true),
        symbol("OLD", "Old Removed Company", "stock", false),
    ])?;

    let symbols = db.active_symbols()?;
    let tickers = symbols
        .iter()
        .map(|symbol| symbol.symbol.as_str())
        .collect::<Vec<_>>();

    assert_eq!(tickers, vec!["MSFT", "SPY"]);

    Ok(())
}

#[test]
fn recent_news_events_replace_is_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    let events = vec![MarketEvent {
        symbol: "NVDA".to_string(),
        sector: Some("Technology".to_string()),
        event_date: "2026-05-26".to_string(),
        event_type: "news".to_string(),
        headline: "NVDA announces new AI platform".to_string(),
        source: "alpaca_news:benzinga".to_string(),
        url: Some("https://example.com/nvda".to_string()),
        metadata: MarketEventMetadata::default(),
    }];

    db.replace_recent_news_events("2026-05-20", "2026-05-27", &events)?;
    db.replace_recent_news_events("2026-05-20", "2026-05-27", &events)?;
    drop(db);

    let conn = Connection::open(db_path)?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;

    assert_eq!(count, 1);

    Ok(())
}

#[test]
fn event_provenance_columns_migration_is_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let db = Database::open(&db_path)?;

    db.migrate()?;
    db.migrate()?;

    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("PRAGMA table_info(events)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    for expected in [
        "event_time",
        "source_event_id",
        "effective_date",
        "processed_at",
        "fetched_at",
        "actual",
        "estimate",
        "surprise",
        "fiscal_period",
        "raw_json",
        "quality_status",
    ] {
        assert!(
            columns.iter().any(|column| column == expected),
            "expected events column {expected}, got {columns:#?}"
        );
    }

    let index_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_events_source_event_id'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(index_count, 1);

    Ok(())
}

#[test]
fn structured_event_upserts_are_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    let first = structured_event("2026-06-12", 2.14);
    let second = structured_event("2026-06-12", 2.18);

    db.upsert_structured_events(&[first])?;
    db.upsert_structured_events(&[second])?;
    drop(db);

    let conn = Connection::open(db_path)?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
    let estimate: f64 = conn.query_row(
        "SELECT estimate FROM events WHERE source_event_id = 'alpha_vantage:earnings_calendar:MSFT:2026-06-12'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(count, 1);
    assert_eq!(estimate, 2.18);

    Ok(())
}

#[test]
fn score_replacement_writes_are_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    let date = "2026-05-27";
    let regime = market_regime_score(date);
    let sectors = vec![sector_score(date)];
    let industries = vec![industry_score(date)];
    let stocks = vec![stock_score(date)];

    db.replace_market_regime(&regime)?;
    db.replace_market_regime(&regime)?;
    db.replace_sector_scores(date, &sectors)?;
    db.replace_sector_scores(date, &sectors)?;
    db.replace_industry_scores(date, &industries)?;
    db.replace_industry_scores(date, &industries)?;
    db.replace_stock_scores(date, &stocks)?;
    db.replace_stock_scores(date, &stocks)?;
    db.replace_watchlist(date, &stocks)?;
    db.replace_watchlist(date, &stocks)?;
    drop(db);

    let conn = Connection::open(db_path)?;

    assert_eq!(count_rows(&conn, "market_regime_scores")?, 1);
    assert_eq!(count_rows(&conn, "sector_scores")?, 1);
    assert_eq!(count_rows(&conn, "industry_scores")?, 1);
    assert_eq!(count_rows(&conn, "stock_scores")?, 1);
    assert_eq!(count_rows(&conn, "watchlists")?, 1);

    Ok(())
}

#[test]
fn intraday_schema_migrations_are_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let db = Database::open(&db_path)?;

    db.migrate()?;
    db.migrate()?;

    let conn = Connection::open(db_path)?;
    let intraday_columns = table_columns(&conn, "prices_intraday")?;
    assert!(intraday_columns.iter().any(|column| column == "vwap"));

    for table in ["volume_profiles", "intraday_setups", "intraday_triggers"] {
        assert_eq!(count_sqlite_table(&conn, table)?, 1, "missing {table}");
    }

    Ok(())
}

#[test]
fn intraday_price_and_readiness_writes_are_idempotent() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    db.upsert_intraday_prices(&[intraday_price("2026-03-01T14:30:00Z", 100.0)])?;
    db.upsert_intraday_prices(&[intraday_price("2026-03-01T14:30:00Z", 101.0)])?;
    db.replace_intraday_readiness(
        "2026-03-01",
        &[volume_profile("LEAD", 100.0)],
        &[intraday_setup("LEAD", "high_momentum_universe", false, 0)],
        &[],
    )?;
    db.replace_intraday_readiness(
        "2026-03-01",
        &[volume_profile("LEAD", 101.0)],
        &[intraday_setup("LEAD", "intraday_execution_ready", true, 1)],
        &[intraday_trigger("LEAD")],
    )?;
    drop(db);

    let conn = Connection::open(db_path)?;
    assert_eq!(count_rows(&conn, "prices_intraday")?, 1);
    assert_eq!(count_rows(&conn, "volume_profiles")?, 1);
    assert_eq!(count_rows(&conn, "intraday_setups")?, 1);
    assert_eq!(count_rows(&conn, "intraday_triggers")?, 1);

    let close: f64 = conn.query_row("SELECT close FROM prices_intraday", [], |row| row.get(0))?;
    let label: String = conn.query_row("SELECT primary_label FROM intraday_setups", [], |row| {
        row.get(0)
    })?;
    let trigger_count: i64 =
        conn.query_row("SELECT trigger_count FROM intraday_setups", [], |row| {
            row.get(0)
        })?;

    assert_eq!(close, 101.0);
    assert_eq!(label, "intraday_execution_ready");
    assert_eq!(trigger_count, 1);

    Ok(())
}

fn count_rows(conn: &Connection, table: &str) -> Result<i64> {
    Ok(
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })?,
    )
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    Ok(stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?)
}

fn count_sqlite_table(conn: &Connection, table: &str) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [table],
        |row| row.get(0),
    )?)
}

fn intraday_price(ts: &str, close: f64) -> IntradayPrice {
    IntradayPrice {
        symbol: "LEAD".to_string(),
        ts: ts.to_string(),
        timeframe: "5Min".to_string(),
        open: close,
        high: close,
        low: close,
        close,
        volume: 1000.0,
        vwap: Some(close),
        source: "test-fixture".to_string(),
    }
}

fn volume_profile(symbol: &str, poc: f64) -> VolumeProfile {
    VolumeProfile {
        symbol: symbol.to_string(),
        date: "2026-03-01".to_string(),
        timeframe: "30Min".to_string(),
        poc,
        vah: poc + 1.0,
        val: poc - 1.0,
        vwap: poc,
        high: poc + 1.0,
        low: poc - 1.0,
        total_volume: 1000.0,
        source: "test-fixture".to_string(),
        components_json: "{}".to_string(),
    }
}

fn intraday_setup(
    symbol: &str,
    label: &str,
    stage3_passed: bool,
    trigger_count: usize,
) -> IntradaySetup {
    IntradaySetup {
        date: "2026-03-01".to_string(),
        symbol: symbol.to_string(),
        name: "Leader Inc.".to_string(),
        sector: "Technology".to_string(),
        industry: "Software".to_string(),
        direction: "long".to_string(),
        stage1_passed: true,
        stage2_passed: stage3_passed,
        stage3_passed,
        primary_label: label.to_string(),
        adr_pct: 0.05,
        rvol_ratio: 2.0,
        mansfield_rs_spy: 1.1,
        mansfield_rs_sector: 1.05,
        ema_10: 100.0,
        ema_20: 99.0,
        latest_price: 100.5,
        confluence_count: 3,
        confluence_json: r#"["poc","val","vwap"]"#.to_string(),
        trigger_count,
        components_json: "{}".to_string(),
    }
}

fn intraday_trigger(symbol: &str) -> IntradayTrigger {
    IntradayTrigger {
        date: "2026-03-01".to_string(),
        symbol: symbol.to_string(),
        ts: "2026-03-01T15:00:00Z".to_string(),
        timeframe: "5Min".to_string(),
        trigger_type: "orb_breakout".to_string(),
        direction: "long".to_string(),
        trigger_price: 101.0,
        reference_level: 100.0,
        volume_spike: 2.0,
        price_action: "fixture trigger".to_string(),
        components_json: "{}".to_string(),
        source: "test-fixture".to_string(),
    }
}

fn symbol(ticker: &str, name: &str, asset_type: &str, is_active: bool) -> Symbol {
    Symbol {
        symbol: ticker.to_string(),
        name: name.to_string(),
        asset_type: asset_type.to_string(),
        sector: None,
        industry: None,
        exchange: "US".to_string(),
        market_cap: None,
        is_active,
    }
}

fn market_regime_score(date: &str) -> MarketRegimeScore {
    MarketRegimeScore {
        date: date.to_string(),
        label: "risk_on".to_string(),
        score: 75.0,
        spy_return_20d: 0.10,
        spy_return_60d: 0.20,
        qqq_relative_return_vs_spy: 0.01,
        iwm_relative_return_vs_spy: 0.02,
        dia_relative_return_vs_spy: 0.0,
        components_json: "{}".to_string(),
        explanation: "fixture regime".to_string(),
    }
}

fn sector_score(date: &str) -> SectorScore {
    SectorScore {
        date: date.to_string(),
        sector: "Technology".to_string(),
        sector_etf: "XLK".to_string(),
        score: 90.0,
        rank: 1,
        return_1d: 0.0,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        breadth_20d: 0.5,
        breadth_50d: 0.5,
        rank_change: 0.0,
        explanation: "fixture sector score".to_string(),
    }
}

fn industry_score(date: &str) -> IndustryScore {
    IndustryScore {
        date: date.to_string(),
        industry: "Software".to_string(),
        sector: "Technology".to_string(),
        score: 80.0,
        rank: 1,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_sector: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        breadth_20d: 0.5,
        breadth_50d: 0.5,
        high_20d_rate: 0.0,
        member_count: 1,
        components_json: "{}".to_string(),
    }
}

fn stock_score(date: &str) -> StockScore {
    StockScore {
        date: date.to_string(),
        rank: 1,
        symbol: "MSFT".to_string(),
        name: "Microsoft Corporation".to_string(),
        sector: "Technology".to_string(),
        industry: "Software".to_string(),
        score: 90.0,
        sector_score: 90.0,
        return_1d: 0.0,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_sector: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        avg_dollar_volume: 1_000_000.0,
        trend_state: "fixture".to_string(),
        catalyst_status: "pending_source".to_string(),
        components_json: "{}".to_string(),
        explanation: "fixture stock score".to_string(),
    }
}

fn macro_observation(date: &str, value: f64) -> MacroObservation {
    MacroObservation {
        series: "VIXCLS".to_string(),
        series_name: "CBOE Volatility Index: VIX".to_string(),
        date: date.to_string(),
        value,
        source: "fred:VIXCLS".to_string(),
        frequency: "Daily".to_string(),
        units: "Index".to_string(),
        realtime_start: date.to_string(),
        realtime_end: date.to_string(),
        raw_json: format!(r#"{{"series_id":"VIXCLS","date":"{date}","value":"{value}"}}"#),
        quality_status: "ok".to_string(),
    }
}

fn structured_event(date: &str, estimate: f64) -> MarketEvent {
    MarketEvent {
        symbol: "MSFT".to_string(),
        sector: Some("Technology".to_string()),
        event_date: date.to_string(),
        event_type: "earnings".to_string(),
        headline: "Expected earnings for Microsoft Corporation".to_string(),
        source: "alpha_vantage:earnings_calendar".to_string(),
        url: None,
        metadata: MarketEventMetadata {
            source_event_id: Some(format!("alpha_vantage:earnings_calendar:MSFT:{date}")),
            effective_date: Some(date.to_string()),
            estimate: Some(estimate),
            raw_json: Some("{}".to_string()),
            ..MarketEventMetadata::default()
        },
    }
}
