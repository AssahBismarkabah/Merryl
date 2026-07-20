use anyhow::Result;
use rusqlite::params;
use serde_json::json;

use crate::config::{market_data, scoring::REPORT_WATCHLIST_LIMIT};
use crate::domain::models::{
    DailyPrice, IndustryMap, IndustryScore, IntradayPrice, IntradaySetup, IntradayTrigger,
    MacroObservation, MarketEvent, MarketRegimeScore, ScreenerResultRow, SectorMap, SectorScore,
    StockScore, Symbol, VolumeProfile,
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

    pub fn upsert_intraday_prices(&mut self, prices: &[IntradayPrice]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO prices_intraday (
                    symbol, ts, timeframe, open, high, low, close, volume, vwap, source, inserted_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)
                ON CONFLICT(symbol, ts, timeframe) DO UPDATE SET
                    open = excluded.open,
                    high = excluded.high,
                    low = excluded.low,
                    close = excluded.close,
                    volume = excluded.volume,
                    vwap = excluded.vwap,
                    source = excluded.source,
                    inserted_at = CURRENT_TIMESTAMP
                "#,
            )?;

            for price in prices {
                stmt.execute(params![
                    &price.symbol,
                    &price.ts,
                    &price.timeframe,
                    price.open,
                    price.high,
                    price.low,
                    price.close,
                    price.volume,
                    price.vwap,
                    &price.source,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_macro_observations(&mut self, observations: &[MacroObservation]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO macro_series (
                    series, date, value, source, series_name, frequency, units,
                    realtime_start, realtime_end, raw_json, quality_status, inserted_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)
                ON CONFLICT(series, date) DO UPDATE SET
                    value = excluded.value,
                    source = excluded.source,
                    series_name = excluded.series_name,
                    frequency = excluded.frequency,
                    units = excluded.units,
                    realtime_start = excluded.realtime_start,
                    realtime_end = excluded.realtime_end,
                    raw_json = excluded.raw_json,
                    quality_status = excluded.quality_status,
                    inserted_at = CURRENT_TIMESTAMP
                "#,
            )?;

            for observation in observations {
                stmt.execute(params![
                    &observation.series,
                    &observation.date,
                    observation.value,
                    &observation.source,
                    &observation.series_name,
                    &observation.frequency,
                    &observation.units,
                    &observation.realtime_start,
                    &observation.realtime_end,
                    &observation.raw_json,
                    &observation.quality_status,
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

    /// Replace all screener results for a given sector.
    pub fn replace_screener_results(
        &mut self,
        sector: &str,
        results: &[ScreenerResultRow],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM screener_cache WHERE sector = ?1",
            params![sector],
        )?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO screener_cache (
                    sector, ticker, company, industry, market_cap, pe_ratio, price,
                    change, volume, dividend, roa, roe, debt_equity, net_profit_margin
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                "#,
            )?;

            for row in results {
                stmt.execute(params![
                    &row.sector,
                    &row.ticker,
                    &row.company,
                    &row.industry,
                    &row.market_cap,
                    &row.pe_ratio,
                    &row.price,
                    &row.change,
                    &row.volume,
                    &row.dividend,
                    &row.roa,
                    &row.roe,
                    &row.debt_equity,
                    &row.net_profit_margin,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_recent_news_events(
        &mut self,
        from_date: &str,
        to_date: &str,
        events: &[MarketEvent],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            r#"
            DELETE FROM events
            WHERE event_date BETWEEN ?1 AND ?2
              AND event_type = ?3
              AND source LIKE ?4
            "#,
            params![
                from_date,
                to_date,
                market_data::NEWS_EVENT_TYPE,
                format!("{}:%", market_data::NEWS_SOURCE_PREFIX),
            ],
        )?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO events (
                    symbol, sector, event_date, event_time, event_type, headline, source, url,
                    source_event_id, effective_date, processed_at, fetched_at, actual, estimate,
                    surprise, fiscal_period, raw_json, quality_status
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
                "#,
            )?;

            for event in events {
                stmt.execute(params![
                    &event.symbol,
                    event.sector.as_deref(),
                    &event.event_date,
                    event.metadata.event_time.as_deref(),
                    &event.event_type,
                    &event.headline,
                    &event.source,
                    event.url.as_deref(),
                    event.metadata.source_event_id.as_deref(),
                    event.metadata.effective_date.as_deref(),
                    event.metadata.processed_at.as_deref(),
                    event.metadata.fetched_at.as_deref(),
                    event.metadata.actual,
                    event.metadata.estimate,
                    event.metadata.surprise,
                    event.metadata.fiscal_period.as_deref(),
                    event.metadata.raw_json.as_deref().unwrap_or("{}"),
                    &event.metadata.quality_status,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_structured_events(&mut self, events: &[MarketEvent]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO events (
                    symbol, sector, event_date, event_time, event_type, headline, source, url,
                    source_event_id, effective_date, processed_at, fetched_at, actual, estimate,
                    surprise, fiscal_period, raw_json, quality_status
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
                ON CONFLICT(source, source_event_id)
                WHERE source_event_id IS NOT NULL AND source_event_id != ''
                DO UPDATE SET
                    symbol = excluded.symbol,
                    sector = excluded.sector,
                    event_date = excluded.event_date,
                    event_time = excluded.event_time,
                    event_type = excluded.event_type,
                    headline = excluded.headline,
                    url = excluded.url,
                    effective_date = excluded.effective_date,
                    processed_at = excluded.processed_at,
                    fetched_at = excluded.fetched_at,
                    actual = excluded.actual,
                    estimate = excluded.estimate,
                    surprise = excluded.surprise,
                    fiscal_period = excluded.fiscal_period,
                    raw_json = excluded.raw_json,
                    quality_status = excluded.quality_status,
                    inserted_at = CURRENT_TIMESTAMP
                "#,
            )?;

            for event in events {
                if event
                    .metadata
                    .source_event_id
                    .as_deref()
                    .is_none_or(str::is_empty)
                {
                    continue;
                }
                stmt.execute(params![
                    &event.symbol,
                    event.sector.as_deref(),
                    &event.event_date,
                    event.metadata.event_time.as_deref(),
                    &event.event_type,
                    &event.headline,
                    &event.source,
                    event.url.as_deref(),
                    event.metadata.source_event_id.as_deref(),
                    event.metadata.effective_date.as_deref(),
                    event.metadata.processed_at.as_deref(),
                    event.metadata.fetched_at.as_deref(),
                    event.metadata.actual,
                    event.metadata.estimate,
                    event.metadata.surprise,
                    event.metadata.fiscal_period.as_deref(),
                    event.metadata.raw_json.as_deref().unwrap_or("{}"),
                    &event.metadata.quality_status,
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

    pub fn replace_intraday_readiness(
        &mut self,
        date: &str,
        profiles: &[VolumeProfile],
        setups: &[IntradaySetup],
        triggers: &[IntradayTrigger],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM volume_profiles WHERE date = ?1", params![date])?;
        tx.execute("DELETE FROM intraday_setups WHERE date = ?1", params![date])?;
        tx.execute(
            "DELETE FROM intraday_triggers WHERE date = ?1",
            params![date],
        )?;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO volume_profiles (
                    symbol, date, timeframe, poc, vah, val, vwap, high, low,
                    total_volume, source, components_json, inserted_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP)
                "#,
            )?;
            for profile in profiles {
                stmt.execute(params![
                    &profile.symbol,
                    &profile.date,
                    &profile.timeframe,
                    profile.poc,
                    profile.vah,
                    profile.val,
                    profile.vwap,
                    profile.high,
                    profile.low,
                    profile.total_volume,
                    &profile.source,
                    &profile.components_json,
                ])?;
            }
        }

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO intraday_setups (
                    date, symbol, name, sector, industry, direction, stage1_passed,
                    stage2_passed, stage3_passed, primary_label, adr_pct, rvol_ratio,
                    mansfield_rs_spy, mansfield_rs_sector, ema_10, ema_20, latest_price,
                    confluence_count, confluence_json, trigger_count, components_json,
                    inserted_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                        ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, CURRENT_TIMESTAMP)
                "#,
            )?;
            for setup in setups {
                stmt.execute(params![
                    &setup.date,
                    &setup.symbol,
                    &setup.name,
                    &setup.sector,
                    &setup.industry,
                    &setup.direction,
                    bool_int(setup.stage1_passed),
                    bool_int(setup.stage2_passed),
                    bool_int(setup.stage3_passed),
                    &setup.primary_label,
                    setup.adr_pct,
                    setup.rvol_ratio,
                    setup.mansfield_rs_spy,
                    setup.mansfield_rs_sector,
                    setup.ema_10,
                    setup.ema_20,
                    setup.latest_price,
                    setup.confluence_count as i64,
                    &setup.confluence_json,
                    setup.trigger_count as i64,
                    &setup.components_json,
                ])?;
            }
        }

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO intraday_triggers (
                    date, symbol, ts, timeframe, trigger_type, direction, trigger_price,
                    reference_level, volume_spike, price_action, components_json, source,
                    inserted_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP)
                "#,
            )?;
            for trigger in triggers {
                stmt.execute(params![
                    &trigger.date,
                    &trigger.symbol,
                    &trigger.ts,
                    &trigger.timeframe,
                    &trigger.trigger_type,
                    &trigger.direction,
                    trigger.trigger_price,
                    trigger.reference_level,
                    trigger.volume_spike,
                    &trigger.price_action,
                    &trigger.components_json,
                    &trigger.source,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
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

fn bool_int(value: bool) -> i64 {
    if value { 1 } else { 0 }
}
