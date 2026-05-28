use anyhow::Result;
use rusqlite::Connection;
use tempfile::tempdir;

use merryl::domain::models::{
    IndustryScore, MacroObservation, MarketEvent, MarketRegimeScore, SectorScore, StockScore,
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

fn count_rows(conn: &Connection, table: &str) -> Result<i64> {
    Ok(
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })?,
    )
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
