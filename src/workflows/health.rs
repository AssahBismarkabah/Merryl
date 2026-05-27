use anyhow::Result;

use crate::storage::{Database, default_db_path};

use super::messages;

pub fn status() -> Result<String> {
    let db_path = default_db_path();
    if !db_path.exists() {
        return Ok(messages::missing_database(db_path.display()));
    }

    let db = Database::open(&db_path)?;
    db.migrate()?;
    let counts = db.counts()?;

    Ok(messages::database_status(db_path.display(), &counts))
}
