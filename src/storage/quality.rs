use anyhow::{Context, Result};
use rusqlite::params;

use super::sqlite::Database;

#[derive(Debug, Clone)]
pub struct DataQualitySnapshot {
    pub symbol_coverage: RequiredSymbolCoverage,
    pub missing_sector_maps: Vec<String>,
    pub price_coverage: Vec<RequiredPriceCoverage>,
    pub latest_benchmark_price_date: Option<String>,
    pub score_dates: i64,
    pub latest_score_date: Option<String>,
    pub latest_score_coverage: LatestScoreCoverage,
}

#[derive(Debug, Clone)]
pub struct RequiredSymbolCoverage {
    pub required_count: usize,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RequiredPriceCoverage {
    pub symbol: String,
    pub bar_count: i64,
    pub first_date: Option<String>,
    pub latest_date: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LatestScoreCoverage {
    pub market_regime_rows: i64,
    pub sector_rows: i64,
    pub industry_rows: i64,
    pub stock_rows: i64,
    pub watchlist_rows: i64,
}

impl Database {
    pub fn data_quality_snapshot(
        &self,
        required_symbols: &[&str],
        required_sector_maps: &[(&str, &str)],
    ) -> Result<DataQualitySnapshot> {
        let latest_score_date = self.latest_table_date("sector_scores")?;
        let latest_score_coverage = match latest_score_date.as_deref() {
            Some(date) => self.latest_score_coverage(date)?,
            None => LatestScoreCoverage::default(),
        };

        Ok(DataQualitySnapshot {
            symbol_coverage: self.required_symbol_coverage(required_symbols)?,
            missing_sector_maps: self.missing_sector_maps(required_sector_maps)?,
            price_coverage: self.required_price_coverage(required_symbols)?,
            latest_benchmark_price_date: self.latest_symbol_price_date("SPY")?,
            score_dates: self.count_distinct_dates_for_table("sector_scores")?,
            latest_score_date,
            latest_score_coverage,
        })
    }

    fn required_symbol_coverage(
        &self,
        required_symbols: &[&str],
    ) -> Result<RequiredSymbolCoverage> {
        let mut missing = Vec::new();

        for symbol in required_symbols {
            if !self.symbol_exists(symbol)? {
                missing.push((*symbol).to_string());
            }
        }

        Ok(RequiredSymbolCoverage {
            required_count: required_symbols.len(),
            missing,
        })
    }

    fn required_price_coverage(
        &self,
        required_symbols: &[&str],
    ) -> Result<Vec<RequiredPriceCoverage>> {
        required_symbols
            .iter()
            .map(|symbol| self.price_coverage(symbol))
            .collect()
    }

    fn missing_sector_maps(&self, required_sector_maps: &[(&str, &str)]) -> Result<Vec<String>> {
        let mut missing = Vec::new();

        for (sector, sector_etf) in required_sector_maps {
            if !self.sector_map_exists(sector, sector_etf)? {
                missing.push(format!("{sector}/{sector_etf}"));
            }
        }

        Ok(missing)
    }

    fn latest_score_coverage(&self, date: &str) -> Result<LatestScoreCoverage> {
        Ok(LatestScoreCoverage {
            market_regime_rows: self.count_rows_for_date("market_regime_scores", date)?,
            sector_rows: self.count_rows_for_date("sector_scores", date)?,
            industry_rows: self.count_rows_for_date("industry_scores", date)?,
            stock_rows: self.count_rows_for_date("stock_scores", date)?,
            watchlist_rows: self.count_rows_for_date("watchlists", date)?,
        })
    }

    fn symbol_exists(&self, symbol: &str) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE symbol = ?1",
                params![symbol],
                |row| row.get(0),
            )
            .with_context(|| format!("failed to check required symbol {symbol}"))?;

        Ok(count > 0)
    }

    fn sector_map_exists(&self, sector: &str, sector_etf: &str) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sector_map WHERE sector = ?1 AND sector_etf = ?2",
                params![sector, sector_etf],
                |row| row.get(0),
            )
            .with_context(|| format!("failed to check sector map {sector}/{sector_etf}"))?;

        Ok(count > 0)
    }

    fn price_coverage(&self, symbol: &str) -> Result<RequiredPriceCoverage> {
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*), MIN(date), MAX(date)
                FROM prices_daily
                WHERE symbol = ?1
                "#,
                params![symbol],
                |row| {
                    Ok(RequiredPriceCoverage {
                        symbol: symbol.to_string(),
                        bar_count: row.get(0)?,
                        first_date: row.get(1)?,
                        latest_date: row.get(2)?,
                    })
                },
            )
            .with_context(|| format!("failed to check price coverage for {symbol}"))
    }

    fn latest_symbol_price_date(&self, symbol: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT MAX(date) FROM prices_daily WHERE symbol = ?1",
                params![symbol],
                |row| row.get(0),
            )
            .with_context(|| format!("failed to check latest price date for {symbol}"))
    }

    fn latest_table_date(&self, table: &str) -> Result<Option<String>> {
        self.conn
            .query_row(&format!("SELECT MAX(date) FROM {table}"), [], |row| {
                row.get(0)
            })
            .with_context(|| format!("failed to check latest {table} date"))
    }

    fn count_rows_for_date(&self, table: &str, date: &str) -> Result<i64> {
        self.conn
            .query_row(
                &format!("SELECT COUNT(*) FROM {table} WHERE date = ?1"),
                params![date],
                |row| row.get(0),
            )
            .with_context(|| format!("failed to count {table} rows for {date}"))
    }

    fn count_distinct_dates_for_table(&self, table: &str) -> Result<i64> {
        self.conn
            .query_row(
                &format!("SELECT COUNT(DISTINCT date) FROM {table}"),
                [],
                |row| row.get(0),
            )
            .with_context(|| format!("failed to count distinct {table} dates"))
    }
}
