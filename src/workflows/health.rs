use std::path::Path;

use anyhow::Result;

use crate::config::paths;
use crate::data::AlpacaProvider;
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

pub fn doctor() -> Result<Vec<String>> {
    let mut checks = Vec::new();
    checks.push(messages::cli_name_check());

    for path in paths::REQUIRED_DOCS {
        checks.push(path_check(path));
    }

    checks.extend(AlpacaProvider::env_status());
    checks.push(path_check(paths::DAILY_WORKFLOW_CONFIG));
    checks.push(generated_path_check(paths::DB_PATH));
    checks.push(generated_path_check(paths::REPORTS_DIR));
    checks.push(generated_path_check(paths::EXPORTS_DIR));

    Ok(checks)
}

fn path_check(path: &str) -> String {
    if Path::new(path).exists() {
        messages::ok(path)
    } else {
        messages::missing(path)
    }
}

fn generated_path_check(path: &str) -> String {
    if Path::new(path).exists() {
        messages::ok(path)
    } else {
        messages::not_created_yet(path)
    }
}
