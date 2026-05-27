use std::collections::{HashMap, HashSet};

use anyhow::Result;
use rusqlite::{OptionalExtension, params};

use crate::domain::models::{DailyPrice, SectorMap, SectorScore, StockScore};

use super::sqlite::Database;

impl Database {
    pub fn sector_scores_between(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<SectorScore>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, sector, sector_etf, score, rank, return_1d, return_5d, return_20d,
                   return_60d, relative_return_vs_spy, relative_volume, breadth_20d,
                   breadth_50d, rank_change, explanation
            FROM sector_scores
            WHERE date BETWEEN ?1 AND ?2
            ORDER BY date, rank
            "#,
        )?;
        let rows = stmt.query_map(params![from_date, to_date], |row| {
            Ok(SectorScore {
                date: row.get(0)?,
                sector: row.get(1)?,
                sector_etf: row.get(2)?,
                score: row.get(3)?,
                rank: row.get::<_, i64>(4)? as usize,
                return_1d: row.get(5)?,
                return_5d: row.get(6)?,
                return_20d: row.get(7)?,
                return_60d: row.get(8)?,
                relative_return_vs_spy: row.get(9)?,
                relative_volume: row.get(10)?,
                breadth_20d: row.get(11)?,
                breadth_50d: row.get(12)?,
                rank_change: row.get(13)?,
                explanation: row.get(14)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn stock_scores_between(&self, from_date: &str, to_date: &str) -> Result<Vec<StockScore>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, rank, symbol, name, sector, industry, score, sector_score, return_1d,
                   return_5d, return_20d, return_60d, relative_return_vs_sector,
                   relative_return_vs_spy, relative_volume, avg_dollar_volume, trend_state,
                   catalyst_status, components_json, explanation
            FROM stock_scores
            WHERE date BETWEEN ?1 AND ?2
            ORDER BY date, rank
            "#,
        )?;
        let rows = stmt.query_map(params![from_date, to_date], |row| {
            Ok(StockScore {
                date: row.get(0)?,
                rank: row.get::<_, i64>(1)? as usize,
                symbol: row.get(2)?,
                name: row.get(3)?,
                sector: row.get(4)?,
                industry: row.get(5)?,
                score: row.get(6)?,
                sector_score: row.get(7)?,
                return_1d: row.get(8)?,
                return_5d: row.get(9)?,
                return_20d: row.get(10)?,
                return_60d: row.get(11)?,
                relative_return_vs_sector: row.get(12)?,
                relative_return_vs_spy: row.get(13)?,
                relative_volume: row.get(14)?,
                avg_dollar_volume: row.get(15)?,
                trend_state: row.get(16)?,
                catalyst_status: row.get(17)?,
                components_json: row.get(18)?,
                explanation: row.get(19)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn sector_maps(&self) -> Result<Vec<SectorMap>> {
        let mut stmt = self
            .conn
            .prepare("SELECT sector, sector_etf, description FROM sector_map ORDER BY sector")?;
        let rows = stmt.query_map([], |row| {
            Ok(SectorMap {
                sector: row.get(0)?,
                sector_etf: row.get(1)?,
                description: row.get(2)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn daily_prices(&self) -> Result<Vec<DailyPrice>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT symbol, date, open, high, low, close, adjusted_close, volume, source
            FROM prices_daily
            ORDER BY symbol, date
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DailyPrice {
                symbol: row.get(0)?,
                date: row.get(1)?,
                open: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                close: row.get(5)?,
                adjusted_close: row.get(6)?,
                volume: row.get(7)?,
                source: row.get(8)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

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
