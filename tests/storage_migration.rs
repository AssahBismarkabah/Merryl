use anyhow::Result;
use rusqlite::Connection;
use tempfile::tempdir;

use merryl::domain::models::MarketEvent;
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
