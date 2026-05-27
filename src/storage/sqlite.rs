use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::config::paths;

pub fn default_db_path() -> PathBuf {
    paths::db_path()
}

pub struct Database {
    pub(crate) conn: Connection,
}

#[derive(Debug)]
pub struct DbCounts {
    pub symbols: i64,
    pub prices_daily: i64,
    pub market_regime_scores: i64,
    pub score_dates: i64,
    pub sector_scores: i64,
    pub industry_scores: i64,
    pub stock_scores: i64,
    pub watchlist_rows: i64,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create database directory {}", parent.display())
            })?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("failed to open SQLite database {}", path.display()))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Self { conn })
    }

    pub fn counts(&self) -> Result<DbCounts> {
        Ok(DbCounts {
            symbols: self.count_table("symbols")?,
            prices_daily: self.count_table("prices_daily")?,
            market_regime_scores: self.count_table("market_regime_scores")?,
            score_dates: self.count_distinct_dates("sector_scores")?,
            sector_scores: self.count_table("sector_scores")?,
            industry_scores: self.count_table("industry_scores")?,
            stock_scores: self.count_table("stock_scores")?,
            watchlist_rows: self.count_table("watchlists")?,
        })
    }

    fn count_table(&self, table: &str) -> Result<i64> {
        self.conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .with_context(|| format!("failed to count {table} rows"))
    }

    fn count_distinct_dates(&self, table: &str) -> Result<i64> {
        self.conn
            .query_row(
                &format!("SELECT COUNT(DISTINCT date) FROM {table}"),
                [],
                |row| row.get(0),
            )
            .with_context(|| format!("failed to count distinct {table} dates"))
    }
}
