use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::config::{macro_data, scoring, universe};
use crate::domain::models::{
    BacktestResultRow, IndustryScore, MarketRegimeScore, SectorScore, StockScore, WatchlistRow,
};
use crate::storage::{DataQualitySnapshot, Database};

use super::models::{
    BacktestDto, DashboardSnapshot, DataHealthDto, HealthDto, IndustryDto, LatestScoreCoverageDto,
    MacroCoverageDto, PriceCoverageDto, RegimeDto, SectorDto, StockDto, WatchlistDto,
};

const RUN_DAILY_MESSAGE: &str =
    "missing dashboard data; run `merryl run daily --date latest` first";

pub fn load_health(db_path: &Path) -> Result<HealthDto> {
    if !db_path.exists() {
        bail!(RUN_DAILY_MESSAGE);
    }

    let db = Database::open(db_path)?;
    db.migrate()?;
    let snapshot = db.data_quality_snapshot(
        &universe::required_market_symbols(),
        universe::SECTOR_ETFS,
        macro_data::MACRO_SERIES,
    )?;

    Ok(HealthDto {
        status: "ok".to_string(),
        database_path: db_path.display().to_string(),
        latest_score_date: snapshot.latest_score_date,
        score_dates: snapshot.score_dates,
    })
}

pub fn load_scored_dates(db_path: &Path) -> Result<Vec<String>> {
    if !db_path.exists() {
        bail!(RUN_DAILY_MESSAGE);
    }

    let db = Database::open(db_path)?;
    db.migrate()?;
    db.scored_dates()
}

pub fn load_latest_dashboard(db_path: &Path) -> Result<DashboardSnapshot> {
    if !db_path.exists() {
        bail!(RUN_DAILY_MESSAGE);
    }

    let db = Database::open(db_path)?;
    db.migrate()?;
    let Some(date) = db.latest_scored_date()? else {
        bail!(RUN_DAILY_MESSAGE);
    };

    dashboard_for_date(db_path, &db, &date)
}

pub fn load_dashboard_for_date(db_path: &Path, date: &str) -> Result<DashboardSnapshot> {
    if !db_path.exists() {
        bail!(RUN_DAILY_MESSAGE);
    }

    let db = Database::open(db_path)?;
    db.migrate()?;
    dashboard_for_date(db_path, &db, date)
}

pub fn is_missing_dashboard_data_error(message: &str) -> bool {
    message.contains("run `merryl run daily --date latest` first")
}

fn dashboard_for_date(db_path: &Path, db: &Database, date: &str) -> Result<DashboardSnapshot> {
    let sectors = db.sector_scores_for_date(date)?;
    let industries = db.industry_scores_for_date(date, scoring::TOP_INDUSTRY_REPORT_LIMIT)?;
    let stocks = db.stock_scores_for_date(date, scoring::STOCK_WATCHLIST_LIMIT)?;
    if sectors.is_empty() || industries.is_empty() || stocks.is_empty() {
        bail!(RUN_DAILY_MESSAGE);
    }

    let regime = db.market_regime_for_date(date)?;
    let watchlist = db.watchlist_for_date(date)?;
    let latest_backtest = db.latest_backtest_result()?;
    let data_quality = db.data_quality_snapshot(
        &universe::required_market_symbols(),
        universe::SECTOR_ETFS,
        macro_data::MACRO_SERIES,
    )?;

    Ok(DashboardSnapshot {
        score_date: date.to_string(),
        limitations: limitations(),
        regime: regime.map(regime_dto),
        sectors: sectors.into_iter().map(sector_dto).collect(),
        industries: industries.into_iter().map(industry_dto).collect(),
        watchlist: watchlist_dtos(&watchlist, &stocks),
        stocks: stocks.into_iter().map(stock_dto).collect(),
        latest_backtest: latest_backtest.map(backtest_dto).transpose()?,
        data_health: data_health_dto(db_path, data_quality),
    })
}

fn limitations() -> Vec<String> {
    vec![
        "Dashboard is read-only and uses stored SQLite scores.".to_string(),
        "Watchlist rows are not automatic trade signals.".to_string(),
        "Sector ranking is a market-map and attention layer, not a proven standalone forward-return signal.".to_string(),
        "Market regime scoring still uses ETF proxies SPY, QQQ, IWM, DIA, TLT, GLD, and USO; FRED macro context is stored separately and is not part of scoring yet.".to_string(),
        "Structured earnings calendar data is not connected yet.".to_string(),
        "Backtests validate score behavior, not trade profitability.".to_string(),
    ]
}

fn regime_dto(regime: MarketRegimeScore) -> RegimeDto {
    let components = json_value(&regime.components_json);
    RegimeDto {
        date: regime.date,
        label: regime.label,
        score: regime.score,
        spy_return_20d: regime.spy_return_20d,
        spy_return_60d: regime.spy_return_60d,
        qqq_relative_return_vs_spy: regime.qqq_relative_return_vs_spy,
        iwm_relative_return_vs_spy: regime.iwm_relative_return_vs_spy,
        dia_relative_return_vs_spy: regime.dia_relative_return_vs_spy,
        tlt_return_20d: json_f64(&components, "tlt_return_20d"),
        gld_return_20d: json_f64(&components, "gld_return_20d"),
        uso_return_20d: json_f64(&components, "uso_return_20d"),
        components,
        explanation: regime.explanation,
    }
}

fn sector_dto(sector: SectorScore) -> SectorDto {
    SectorDto {
        date: sector.date,
        sector: sector.sector,
        sector_etf: sector.sector_etf,
        score: sector.score,
        rank: sector.rank,
        return_1d: sector.return_1d,
        return_5d: sector.return_5d,
        return_20d: sector.return_20d,
        return_60d: sector.return_60d,
        relative_return_vs_spy: sector.relative_return_vs_spy,
        relative_volume: sector.relative_volume,
        breadth_20d: sector.breadth_20d,
        breadth_50d: sector.breadth_50d,
        rank_change: sector.rank_change,
        explanation: sector.explanation,
    }
}

fn industry_dto(industry: IndustryScore) -> IndustryDto {
    IndustryDto {
        date: industry.date,
        industry: industry.industry,
        sector: industry.sector,
        score: industry.score,
        rank: industry.rank,
        return_5d: industry.return_5d,
        return_20d: industry.return_20d,
        return_60d: industry.return_60d,
        relative_return_vs_sector: industry.relative_return_vs_sector,
        relative_return_vs_spy: industry.relative_return_vs_spy,
        relative_volume: industry.relative_volume,
        breadth_20d: industry.breadth_20d,
        breadth_50d: industry.breadth_50d,
        high_20d_rate: industry.high_20d_rate,
        member_count: industry.member_count,
        components: json_value(&industry.components_json),
    }
}

fn stock_dto(stock: StockScore) -> StockDto {
    StockDto {
        date: stock.date,
        rank: stock.rank,
        symbol: stock.symbol,
        name: stock.name,
        sector: stock.sector,
        industry: stock.industry,
        score: stock.score,
        sector_score: stock.sector_score,
        return_1d: stock.return_1d,
        return_5d: stock.return_5d,
        return_20d: stock.return_20d,
        return_60d: stock.return_60d,
        relative_return_vs_sector: stock.relative_return_vs_sector,
        relative_return_vs_spy: stock.relative_return_vs_spy,
        relative_volume: stock.relative_volume,
        avg_dollar_volume: stock.avg_dollar_volume,
        trend_state: stock.trend_state,
        catalyst_status: stock.catalyst_status,
        components: json_value(&stock.components_json),
        explanation: stock.explanation,
    }
}

fn watchlist_dtos(watchlist: &[WatchlistRow], stocks: &[StockScore]) -> Vec<WatchlistDto> {
    let stock_lookup: HashMap<&str, &StockScore> = stocks
        .iter()
        .map(|stock| (stock.symbol.as_str(), stock))
        .collect();

    watchlist
        .iter()
        .map(|row| {
            let stock = stock_lookup.get(row.symbol.as_str());
            WatchlistDto {
                date: row.date.clone(),
                rank: row.rank,
                symbol: row.symbol.clone(),
                name: stock.map(|stock| stock.name.clone()).unwrap_or_default(),
                sector: stock.map(|stock| stock.sector.clone()).unwrap_or_default(),
                industry: stock
                    .map(|stock| stock.industry.clone())
                    .unwrap_or_default(),
                score: row.score,
                catalyst_status: stock
                    .map(|stock| stock.catalyst_status.clone())
                    .unwrap_or_else(|| scoring::CATALYST_PENDING_SOURCE.to_string()),
                reason: row.reason.clone(),
            }
        })
        .collect()
}

fn backtest_dto(row: BacktestResultRow) -> Result<BacktestDto> {
    Ok(BacktestDto {
        id: row.id,
        run_name: row.run_name,
        from_date: row.from_date,
        to_date: row.to_date,
        created_at: row.created_at,
        metrics: serde_json::from_str(&row.metrics_json)
            .with_context(|| format!("failed to parse backtest metrics_json for id {}", row.id))?,
    })
}

fn data_health_dto(db_path: &Path, snapshot: DataQualitySnapshot) -> DataHealthDto {
    DataHealthDto {
        database_path: db_path.display().to_string(),
        latest_benchmark_price_date: snapshot.latest_benchmark_price_date,
        latest_score_date: snapshot.latest_score_date,
        score_dates: snapshot.score_dates,
        required_symbol_count: snapshot.symbol_coverage.required_count,
        missing_symbols: snapshot.symbol_coverage.missing,
        missing_sector_maps: snapshot.missing_sector_maps,
        required_price_coverage: snapshot
            .price_coverage
            .into_iter()
            .map(|coverage| PriceCoverageDto {
                symbol: coverage.symbol,
                bar_count: coverage.bar_count,
                first_date: coverage.first_date,
                latest_date: coverage.latest_date,
            })
            .collect(),
        required_macro_coverage: snapshot
            .macro_coverage
            .into_iter()
            .map(|coverage| MacroCoverageDto {
                series: coverage.series,
                observation_count: coverage.observation_count,
                first_date: coverage.first_date,
                latest_date: coverage.latest_date,
            })
            .collect(),
        latest_score_coverage: LatestScoreCoverageDto {
            market_regime_rows: snapshot.latest_score_coverage.market_regime_rows,
            sector_rows: snapshot.latest_score_coverage.sector_rows,
            industry_rows: snapshot.latest_score_coverage.industry_rows,
            stock_rows: snapshot.latest_score_coverage.stock_rows,
            watchlist_rows: snapshot.latest_score_coverage.watchlist_rows,
        },
    }
}

fn json_value(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or(Value::Null)
}

fn json_f64(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or_default()
}
