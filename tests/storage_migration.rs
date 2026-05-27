use anyhow::Result;
use rusqlite::Connection;
use tempfile::tempdir;

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
