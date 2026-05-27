use anyhow::Result;
use rusqlite::params;
use serde_json::json;

use crate::config::scoring::REPORT_WATCHLIST_LIMIT;
use crate::domain::models::{
    DailyPrice, IndustryMap, IndustryScore, MarketRegimeScore, SectorMap, SectorScore, StockScore,
    Symbol,
};

use super::sqlite::Database;

impl Database {
    pub fn upsert_symbols(&mut self, symbols: &[Symbol]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO symbols (
                    symbol, name, asset_type, sector, industry, exchange, market_cap, is_active, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)
                ON CONFLICT(symbol) DO UPDATE SET
                    name = excluded.name,
                    asset_type = excluded.asset_type,
                    sector = excluded.sector,
                    industry = excluded.industry,
                    exchange = excluded.exchange,
                    market_cap = excluded.market_cap,
                    is_active = excluded.is_active,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )?;

            for symbol in symbols {
                stmt.execute(params![
                    &symbol.symbol,
                    &symbol.name,
                    &symbol.asset_type,
                    symbol.sector.as_deref(),
                    symbol.industry.as_deref(),
                    &symbol.exchange,
                    symbol.market_cap,
                    if symbol.is_active { 1 } else { 0 },
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_prices(&mut self, prices: &[DailyPrice]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO prices_daily (
                    symbol, date, open, high, low, close, adjusted_close, volume, source, inserted_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)
                ON CONFLICT(symbol, date) DO UPDATE SET
                    open = excluded.open,
                    high = excluded.high,
                    low = excluded.low,
                    close = excluded.close,
                    adjusted_close = excluded.adjusted_close,
                    volume = excluded.volume,
                    source = excluded.source,
                    inserted_at = CURRENT_TIMESTAMP
                "#,
            )?;

            for price in prices {
                stmt.execute(params![
                    &price.symbol,
                    &price.date,
                    price.open,
                    price.high,
                    price.low,
                    price.close,
                    price.adjusted_close,
                    price.volume,
                    &price.source,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_sector_maps(&mut self, maps: &[SectorMap]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO sector_map (sector, sector_etf, description)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(sector) DO UPDATE SET
                    sector_etf = excluded.sector_etf,
                    description = excluded.description
                "#,
            )?;

            for map in maps {
                stmt.execute(params![&map.sector, &map.sector_etf, &map.description])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_industry_maps(&mut self, maps: &[IndustryMap]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO industry_map (industry, sector, description)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(industry, sector) DO UPDATE SET
                    description = excluded.description
                "#,
            )?;

            for map in maps {
                stmt.execute(params![&map.industry, &map.sector, &map.description])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_market_regime(&mut self, regime: &MarketRegimeScore) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO market_regime_scores (
                date, label, score, spy_return_20d, spy_return_60d,
                qqq_relative_return_vs_spy, iwm_relative_return_vs_spy,
                dia_relative_return_vs_spy, components_json, explanation
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(date) DO UPDATE SET
                label = excluded.label,
                score = excluded.score,
                spy_return_20d = excluded.spy_return_20d,
                spy_return_60d = excluded.spy_return_60d,
                qqq_relative_return_vs_spy = excluded.qqq_relative_return_vs_spy,
                iwm_relative_return_vs_spy = excluded.iwm_relative_return_vs_spy,
                dia_relative_return_vs_spy = excluded.dia_relative_return_vs_spy,
                components_json = excluded.components_json,
                explanation = excluded.explanation
            "#,
            params![
                &regime.date,
                &regime.label,
                regime.score,
                regime.spy_return_20d,
                regime.spy_return_60d,
                regime.qqq_relative_return_vs_spy,
                regime.iwm_relative_return_vs_spy,
                regime.dia_relative_return_vs_spy,
                &regime.components_json,
                &regime.explanation,
            ],
        )?;
        Ok(())
    }

    pub fn replace_sector_scores(&mut self, date: &str, scores: &[SectorScore]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM sector_scores WHERE date = ?1", params![date])?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO sector_scores (
                    date, sector, sector_etf, score, rank, return_1d, return_5d, return_20d,
                    return_60d, relative_return_vs_spy, relative_volume, breadth_20d,
                    breadth_50d, rank_change, components_json, explanation
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                "#,
            )?;

            for score in scores {
                stmt.execute(params![
                    &score.date,
                    &score.sector,
                    &score.sector_etf,
                    score.score,
                    score.rank as i64,
                    score.return_1d,
                    score.return_5d,
                    score.return_20d,
                    score.return_60d,
                    score.relative_return_vs_spy,
                    score.relative_volume,
                    score.breadth_20d,
                    score.breadth_50d,
                    score.rank_change,
                    sector_components_json(score),
                    &score.explanation,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_industry_scores(&mut self, date: &str, scores: &[IndustryScore]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM industry_scores WHERE date = ?1", params![date])?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO industry_scores (
                    date, industry, sector, score, rank, components_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
            )?;

            for score in scores {
                stmt.execute(params![
                    &score.date,
                    &score.industry,
                    &score.sector,
                    score.score,
                    score.rank as i64,
                    &score.components_json,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_stock_scores(&mut self, date: &str, scores: &[StockScore]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM stock_scores WHERE date = ?1", params![date])?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO stock_scores (
                    date, rank, symbol, name, sector, industry, score, sector_score, return_1d,
                    return_5d, return_20d, return_60d, relative_return_vs_sector,
                    relative_return_vs_spy, relative_volume, avg_dollar_volume, trend_state,
                    catalyst_status, components_json, explanation
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
                "#,
            )?;

            for score in scores {
                stmt.execute(params![
                    &score.date,
                    score.rank as i64,
                    &score.symbol,
                    &score.name,
                    &score.sector,
                    &score.industry,
                    score.score,
                    score.sector_score,
                    score.return_1d,
                    score.return_5d,
                    score.return_20d,
                    score.return_60d,
                    score.relative_return_vs_sector,
                    score.relative_return_vs_spy,
                    score.relative_volume,
                    score.avg_dollar_volume,
                    &score.trend_state,
                    &score.catalyst_status,
                    &score.components_json,
                    &score.explanation,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_watchlist(&mut self, date: &str, scores: &[StockScore]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM watchlists WHERE date = ?1", params![date])?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO watchlists (date, rank, symbol, score, reason)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )?;

            for score in scores.iter().take(REPORT_WATCHLIST_LIMIT) {
                stmt.execute(params![
                    &score.date,
                    score.rank as i64,
                    &score.symbol,
                    score.score,
                    &score.explanation,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_backtest_result(
        &self,
        run_name: &str,
        from_date: &str,
        to_date: &str,
        config_json: &str,
        metrics_json: &str,
    ) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO backtest_results (
                run_name, from_date, to_date, config_json, metrics_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![run_name, from_date, to_date, config_json, metrics_json],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
}

fn sector_components_json(score: &SectorScore) -> String {
    json!({
        "return_1d": score.return_1d,
        "return_5d": score.return_5d,
        "return_20d": score.return_20d,
        "return_60d": score.return_60d,
        "relative_return_vs_spy": score.relative_return_vs_spy,
        "relative_volume": score.relative_volume,
        "breadth_20d": score.breadth_20d,
        "breadth_50d": score.breadth_50d,
        "rank_change": score.rank_change
    })
    .to_string()
}
