use std::collections::{HashMap, HashSet};

use anyhow::Result;
use rusqlite::{OptionalExtension, params};

use crate::config::scoring;
use crate::domain::models::{
    BacktestResultRow, DailyPrice, IndustryScore, IndustryScoreSnapshot, IntradaySetup,
    IntradayTrigger, MacroObservation, MarketRegimeScore, ScreenerResultRow, SectorMap,
    SectorScore, StockScore, Symbol, VolumeProfile, WatchlistRow,
};

use super::sqlite::Database;

impl Database {
    pub fn active_symbols(&self) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT symbol, name, asset_type, sector, industry, exchange, market_cap, is_active
            FROM symbols
            WHERE is_active = 1
            ORDER BY symbol
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Symbol {
                symbol: row.get(0)?,
                name: row.get(1)?,
                asset_type: row.get(2)?,
                sector: row.get(3)?,
                industry: row.get(4)?,
                exchange: row.get(5)?,
                market_cap: row.get(6)?,
                is_active: row.get::<_, i64>(7)? == 1,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn scored_dates(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT date FROM sector_scores ORDER BY date DESC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn latest_scored_date(&self) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT MAX(date) FROM sector_scores", [], |row| row.get(0))
            .optional()?
            .flatten())
    }

    pub fn market_regime_for_date(&self, date: &str) -> Result<Option<MarketRegimeScore>> {
        self.conn
            .query_row(
                r#"
                SELECT date, label, score, spy_return_20d, spy_return_60d,
                       qqq_relative_return_vs_spy, iwm_relative_return_vs_spy,
                       dia_relative_return_vs_spy, components_json, explanation
                FROM market_regime_scores
                WHERE date = ?1
                "#,
                params![date],
                |row| {
                    Ok(MarketRegimeScore {
                        date: row.get(0)?,
                        label: row.get(1)?,
                        score: row.get(2)?,
                        spy_return_20d: row.get(3)?,
                        spy_return_60d: row.get(4)?,
                        qqq_relative_return_vs_spy: row.get(5)?,
                        iwm_relative_return_vs_spy: row.get(6)?,
                        dia_relative_return_vs_spy: row.get(7)?,
                        components_json: row.get(8)?,
                        explanation: row.get(9)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn market_regimes_between(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<MarketRegimeScore>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, label, score, spy_return_20d, spy_return_60d,
                   qqq_relative_return_vs_spy, iwm_relative_return_vs_spy,
                   dia_relative_return_vs_spy, components_json, explanation
            FROM market_regime_scores
            WHERE date BETWEEN ?1 AND ?2
            ORDER BY date
            "#,
        )?;
        let rows = stmt.query_map(params![from_date, to_date], |row| {
            Ok(MarketRegimeScore {
                date: row.get(0)?,
                label: row.get(1)?,
                score: row.get(2)?,
                spy_return_20d: row.get(3)?,
                spy_return_60d: row.get(4)?,
                qqq_relative_return_vs_spy: row.get(5)?,
                iwm_relative_return_vs_spy: row.get(6)?,
                dia_relative_return_vs_spy: row.get(7)?,
                components_json: row.get(8)?,
                explanation: row.get(9)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn sector_scores_for_date(&self, date: &str) -> Result<Vec<SectorScore>> {
        self.sector_scores_between(date, date)
    }

    pub fn industry_scores_for_date(&self, date: &str, limit: usize) -> Result<Vec<IndustryScore>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, industry, sector, score, rank, components_json
            FROM industry_scores
            WHERE date = ?1
            ORDER BY rank
            LIMIT ?2
            "#,
        )?;
        let rows = stmt.query_map(params![date, limit as i64], |row| {
            let components_json: String = row.get(5)?;
            let components = serde_json::from_str::<serde_json::Value>(&components_json).ok();
            Ok(IndustryScore {
                date: row.get(0)?,
                industry: row.get(1)?,
                sector: row.get(2)?,
                score: row.get(3)?,
                rank: row.get::<_, i64>(4)? as usize,
                return_5d: component_f64(components.as_ref(), "return_5d"),
                return_20d: component_f64(components.as_ref(), "return_20d"),
                return_60d: component_f64(components.as_ref(), "return_60d"),
                relative_return_vs_sector: component_f64(
                    components.as_ref(),
                    "relative_return_vs_sector",
                ),
                relative_return_vs_spy: component_f64(
                    components.as_ref(),
                    "relative_return_vs_spy",
                ),
                relative_volume: component_f64(components.as_ref(), "relative_volume"),
                breadth_20d: component_f64(components.as_ref(), "breadth_20d"),
                breadth_50d: component_f64(components.as_ref(), "breadth_50d"),
                high_20d_rate: component_f64(components.as_ref(), "high_20d_rate"),
                member_count: component_usize(components.as_ref(), "member_count"),
                components_json,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn stock_scores_for_date(&self, date: &str, limit: usize) -> Result<Vec<StockScore>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, rank, symbol, name, sector, industry, score, sector_score, return_1d,
                   return_5d, return_20d, return_60d, relative_return_vs_sector,
                   relative_return_vs_spy, relative_volume, avg_dollar_volume, trend_state,
                   catalyst_status, components_json, explanation
            FROM stock_scores
            WHERE date = ?1
            ORDER BY rank
            LIMIT ?2
            "#,
        )?;
        let rows = stmt.query_map(params![date, limit as i64], |row| {
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

    pub fn watchlist_for_date(&self, date: &str) -> Result<Vec<WatchlistRow>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, rank, symbol, score, reason
            FROM watchlists
            WHERE date = ?1
            ORDER BY rank
            "#,
        )?;
        let rows = stmt.query_map(params![date], |row| {
            Ok(WatchlistRow {
                date: row.get(0)?,
                rank: row.get::<_, i64>(1)? as usize,
                symbol: row.get(2)?,
                score: row.get(3)?,
                reason: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn watchlists_between(&self, from_date: &str, to_date: &str) -> Result<Vec<WatchlistRow>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, rank, symbol, score, reason
            FROM watchlists
            WHERE date BETWEEN ?1 AND ?2
            ORDER BY date, rank
            "#,
        )?;
        let rows = stmt.query_map(params![from_date, to_date], |row| {
            Ok(WatchlistRow {
                date: row.get(0)?,
                rank: row.get::<_, i64>(1)? as usize,
                symbol: row.get(2)?,
                score: row.get(3)?,
                reason: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn intraday_setups_for_date(&self, date: &str) -> Result<Vec<IntradaySetup>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, symbol, name, sector, industry, direction, stage1_passed,
                   stage2_passed, stage3_passed, primary_label, adr_pct, rvol_ratio,
                   mansfield_rs_spy, mansfield_rs_sector, ema_10, ema_20, latest_price,
                   confluence_count, confluence_json, trigger_count, components_json
            FROM intraday_setups
            WHERE date = ?1
            ORDER BY stage3_passed DESC, stage2_passed DESC, mansfield_rs_spy DESC, symbol
            "#,
        )?;
        let rows = stmt.query_map(params![date], |row| {
            Ok(IntradaySetup {
                date: row.get(0)?,
                symbol: row.get(1)?,
                name: row.get(2)?,
                sector: row.get(3)?,
                industry: row.get(4)?,
                direction: row.get(5)?,
                stage1_passed: row.get::<_, i64>(6)? == 1,
                stage2_passed: row.get::<_, i64>(7)? == 1,
                stage3_passed: row.get::<_, i64>(8)? == 1,
                primary_label: row.get(9)?,
                adr_pct: row.get(10)?,
                rvol_ratio: row.get(11)?,
                mansfield_rs_spy: row.get(12)?,
                mansfield_rs_sector: row.get(13)?,
                ema_10: row.get(14)?,
                ema_20: row.get(15)?,
                latest_price: row.get(16)?,
                confluence_count: row.get::<_, i64>(17)? as usize,
                confluence_json: row.get(18)?,
                trigger_count: row.get::<_, i64>(19)? as usize,
                components_json: row.get(20)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn intraday_triggers_for_date(&self, date: &str) -> Result<Vec<IntradayTrigger>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, symbol, ts, timeframe, trigger_type, direction, trigger_price,
                   reference_level, volume_spike, price_action, components_json, source
            FROM intraday_triggers
            WHERE date = ?1
            ORDER BY symbol, ts, trigger_type
            "#,
        )?;
        let rows = stmt.query_map(params![date], |row| {
            Ok(IntradayTrigger {
                date: row.get(0)?,
                symbol: row.get(1)?,
                ts: row.get(2)?,
                timeframe: row.get(3)?,
                trigger_type: row.get(4)?,
                direction: row.get(5)?,
                trigger_price: row.get(6)?,
                reference_level: row.get(7)?,
                volume_spike: row.get(8)?,
                price_action: row.get(9)?,
                components_json: row.get(10)?,
                source: row.get(11)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn volume_profiles_for_date(&self, date: &str) -> Result<Vec<VolumeProfile>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT symbol, date, timeframe, poc, vah, val, vwap, high, low,
                   total_volume, source, components_json
            FROM volume_profiles
            WHERE date = ?1
            ORDER BY symbol, timeframe
            "#,
        )?;
        let rows = stmt.query_map(params![date], |row| {
            Ok(VolumeProfile {
                symbol: row.get(0)?,
                date: row.get(1)?,
                timeframe: row.get(2)?,
                poc: row.get(3)?,
                vah: row.get(4)?,
                val: row.get(5)?,
                vwap: row.get(6)?,
                high: row.get(7)?,
                low: row.get(8)?,
                total_volume: row.get(9)?,
                source: row.get(10)?,
                components_json: row.get(11)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn non_pending_stock_catalyst_statuses_before(
        &self,
        date: &str,
    ) -> Result<HashMap<(String, String), String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, symbol, catalyst_status
            FROM stock_scores
            WHERE date < ?1
              AND catalyst_status != ?2
            "#,
        )?;
        let rows = stmt.query_map(params![date, scoring::CATALYST_PENDING_SOURCE], |row| {
            let score_date: String = row.get(0)?;
            let symbol: String = row.get(1)?;
            let catalyst_status: String = row.get(2)?;
            Ok(((score_date, symbol), catalyst_status))
        })?;

        let mut statuses = HashMap::new();
        for row in rows {
            let (key, catalyst_status) = row?;
            statuses.insert(key, catalyst_status);
        }
        Ok(statuses)
    }

    pub fn latest_backtest_result(&self) -> Result<Option<BacktestResultRow>> {
        self.conn
            .query_row(
                r#"
                SELECT id, run_name, from_date, to_date, config_json, metrics_json, created_at
                FROM backtest_results
                ORDER BY id DESC
                LIMIT 1
                "#,
                [],
                |row| {
                    Ok(BacktestResultRow {
                        id: row.get(0)?,
                        run_name: row.get(1)?,
                        from_date: row.get(2)?,
                        to_date: row.get(3)?,
                        config_json: row.get(4)?,
                        metrics_json: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

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

    pub fn industry_scores_between(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<IndustryScoreSnapshot>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, industry, sector, score, rank
            FROM industry_scores
            WHERE date BETWEEN ?1 AND ?2
            ORDER BY date, rank
            "#,
        )?;
        let rows = stmt.query_map(params![from_date, to_date], |row| {
            Ok(IndustryScoreSnapshot {
                date: row.get(0)?,
                industry: row.get(1)?,
                sector: row.get(2)?,
                score: row.get(3)?,
                rank: row.get::<_, i64>(4)? as usize,
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

    pub fn macro_observations_through(&self, to_date: &str) -> Result<Vec<MacroObservation>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT series, series_name, date, value, source, frequency, units,
                   realtime_start, realtime_end, raw_json, quality_status
            FROM macro_series
            WHERE date <= ?1
            ORDER BY series, date
            "#,
        )?;
        let rows = stmt.query_map(params![to_date], |row| {
            Ok(MacroObservation {
                series: row.get(0)?,
                series_name: row.get(1)?,
                date: row.get(2)?,
                value: row.get(3)?,
                source: row.get(4)?,
                frequency: row.get(5)?,
                units: row.get(6)?,
                realtime_start: row.get(7)?,
                realtime_end: row.get(8)?,
                raw_json: row.get(9)?,
                quality_status: row.get(10)?,
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

    /// Return all cached screener rows for a given sector key.
    /// `sector` is the string key used in `screener_cache.sector` ("" for "All Sectors").
    pub fn screener_results_for_sector(&self, sector: &str) -> Result<Vec<ScreenerResultRow>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT sector, ticker, company, industry, market_cap, pe_ratio, price,
                   change, volume, dividend, roa, roe, debt_equity, net_profit_margin
            FROM screener_cache
            WHERE sector = ?1
            ORDER BY ticker
            "#,
        )?;
        let rows = stmt.query_map(params![sector], |row| {
            Ok(ScreenerResultRow {
                sector: row.get(0)?,
                ticker: row.get(1)?,
                company: row.get(2)?,
                industry: row.get(3)?,
                market_cap: row.get(4)?,
                pe_ratio: row.get(5)?,
                price: row.get(6)?,
                change: row.get(7)?,
                volume: row.get(8)?,
                dividend: row.get(9)?,
                roa: row.get(10)?,
                roe: row.get(11)?,
                debt_equity: row.get(12)?,
                net_profit_margin: row.get(13)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Check if screener_cache has any rows for the given sector.
    pub fn screener_has_sector(&self, sector: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM screener_cache WHERE sector = ?1",
            params![sector],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get all screener results across all sectors (for "All Sectors" view).
    /// Merges all sector-specific results, deduplicating by ticker.
    pub fn screener_all_results(&self) -> Result<Vec<ScreenerResultRow>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT sector, ticker, company, industry, market_cap, pe_ratio, price,
                   change, volume, dividend, roa, roe, debt_equity, net_profit_margin
            FROM screener_cache
            WHERE sector != ''
            ORDER BY ticker
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ScreenerResultRow {
                sector: row.get(0)?,
                ticker: row.get(1)?,
                company: row.get(2)?,
                industry: row.get(3)?,
                market_cap: row.get(4)?,
                pe_ratio: row.get(5)?,
                price: row.get(6)?,
                change: row.get(7)?,
                volume: row.get(8)?,
                dividend: row.get(9)?,
                roa: row.get(10)?,
                roe: row.get(11)?,
                debt_equity: row.get(12)?,
                net_profit_margin: row.get(13)?,
            })
        })?;
        let mut seen = std::collections::HashSet::new();
        let results = rows
            .filter_map(|r| {
                r.ok().filter(|row| seen.insert(row.ticker.clone()))
            })
            .collect();
        Ok(results)
    }
}

fn component_f64(value: Option<&serde_json::Value>, key: &str) -> f64 {
    value
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_default()
}

fn component_usize(value: Option<&serde_json::Value>, key: &str) -> usize {
    value
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or_default()
}
