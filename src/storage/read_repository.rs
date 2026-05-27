use std::collections::{HashMap, HashSet};

use anyhow::Result;
use rusqlite::{OptionalExtension, params};

use super::sqlite::Database;

impl Database {
    pub fn latest_sector_ranks_before(&self, date: &str) -> Result<HashMap<String, usize>> {
        let Some(previous_date) = self.latest_score_date_before("sector_scores", date)? else {
            return Ok(HashMap::new());
        };

        let mut stmt = self
            .conn
            .prepare("SELECT sector, rank FROM sector_scores WHERE date = ?1")?;
        let rows = stmt.query_map(params![previous_date], |row| {
            let sector: String = row.get(0)?;
            let rank: i64 = row.get(1)?;
            Ok((sector, rank as usize))
        })?;

        let mut ranks = HashMap::new();
        for row in rows {
            let (sector, rank) = row?;
            ranks.insert(sector, rank);
        }

        Ok(ranks)
    }

    pub fn latest_watchlist_symbols_before(&self, date: &str) -> Result<HashSet<String>> {
        let Some(previous_date) = self.latest_score_date_before("watchlists", date)? else {
            return Ok(HashSet::new());
        };

        let mut stmt = self
            .conn
            .prepare("SELECT symbol FROM watchlists WHERE date = ?1")?;
        let rows = stmt.query_map(params![previous_date], |row| row.get::<_, String>(0))?;

        let mut symbols = HashSet::new();
        for row in rows {
            symbols.insert(row?);
        }

        Ok(symbols)
    }

    fn latest_score_date_before(&self, table: &str, date: &str) -> Result<Option<String>> {
        let sql = format!("SELECT MAX(date) FROM {table} WHERE date < ?1");
        Ok(self
            .conn
            .query_row(&sql, params![date], |row| row.get(0))
            .optional()?
            .flatten())
    }
}
