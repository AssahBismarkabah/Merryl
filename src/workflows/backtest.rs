use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde_json::json;

use crate::backtest::{BacktestInput, run_backtest_analysis};
use crate::config::scoring;
use crate::output::write_backtest_outputs;
use crate::storage::{Database, default_db_path};

#[derive(Debug)]
pub struct RunBacktestResult {
    pub from_date: String,
    pub to_date: String,
    pub database: PathBuf,
    pub report: PathBuf,
    pub summary_export: PathBuf,
    pub sector_observation_count: usize,
    pub sector_component_observation_count: usize,
    pub stock_observation_count: usize,
    pub industry_stock_observation_count: usize,
    pub backtest_result_id: i64,
}

pub fn run_backtest(from_arg: &str, to_arg: &str) -> Result<RunBacktestResult> {
    let from_date = parse_required_date(from_arg, "from")?;
    let to_date = parse_required_date(to_arg, "to")?;
    if from_date > to_date {
        bail!("backtest from date must be before or equal to to date");
    }

    let from_date = from_date.format("%Y-%m-%d").to_string();
    let to_date = to_date.format("%Y-%m-%d").to_string();
    let db_path = default_db_path();
    if !db_path.exists() {
        bail!("missing historical scores or prices; run `merryl run daily --date latest` first");
    }

    let db = Database::open(&db_path)?;
    db.migrate()?;

    let sector_scores = db.sector_scores_between(&from_date, &to_date)?;
    let industry_scores = db.industry_scores_between(&from_date, &to_date)?;
    let stock_scores = db.stock_scores_between(&from_date, &to_date)?;
    let sector_maps = db.sector_maps()?;
    let prices = db.daily_prices()?;

    if sector_scores.is_empty()
        || industry_scores.is_empty()
        || stock_scores.is_empty()
        || sector_maps.is_empty()
        || prices.is_empty()
    {
        bail!("missing historical scores or prices; run `merryl run daily --date latest` first");
    }

    let metrics = run_backtest_analysis(BacktestInput {
        from_date: from_date.clone(),
        to_date: to_date.clone(),
        sector_scores,
        industry_scores,
        stock_scores,
        sector_maps,
        prices,
    })?;
    let outputs = write_backtest_outputs(&metrics)?;
    let run_name = format!("backtest_{}_{}", from_date, to_date);
    let config_json = json!({
        "horizons": scoring::BACKTEST_HORIZONS,
        "validation_scope": &metrics.validation_scope,
        "relative_return_policy": {
            "sector": "sector ETF forward return minus SPY forward return",
            "sector_components": "sector ETF forward return grouped by same-day sector component decile",
            "stock_primary": "stock forward return minus sector ETF forward return",
            "stock_vs_spy": "stock forward return minus SPY forward return",
            "stock_by_industry": "stock forward return grouped by same-day industry/theme score decile"
        },
        "source": "SQLite historical scores and daily prices"
    })
    .to_string();
    let metrics_json =
        serde_json::to_string(&metrics).context("failed to serialize backtest metrics")?;
    let backtest_result_id =
        db.insert_backtest_result(&run_name, &from_date, &to_date, &config_json, &metrics_json)?;

    Ok(RunBacktestResult {
        from_date,
        to_date,
        database: db_path,
        report: outputs.report,
        summary_export: outputs.summary_export,
        sector_observation_count: metrics.sector_observation_count,
        sector_component_observation_count: metrics.sector_component_observation_count,
        stock_observation_count: metrics.stock_observation_count,
        industry_stock_observation_count: metrics.industry_stock_observation_count,
        backtest_result_id,
    })
}

fn parse_required_date(value: &str, label: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("{label} must be an explicit YYYY-MM-DD date"))
}
