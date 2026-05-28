use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde_json::json;

use crate::backtest::{BacktestInput, run_backtest_analysis};
use crate::config::scoring;
use crate::output::{
    write_backtest_outputs, write_event_context_validation_outputs,
    write_macro_regime_validation_outputs,
};
use crate::storage::{Database, default_db_path};
use crate::validation::{
    EventContextValidationInput, MacroRegimeValidationInput, run_event_context_validation,
    run_macro_regime_validation,
};

#[derive(Debug)]
pub struct RunBacktestResult {
    pub from_date: String,
    pub to_date: String,
    pub database: PathBuf,
    pub report: PathBuf,
    pub summary_export: PathBuf,
    pub macro_regime_validation_report: PathBuf,
    pub macro_regime_validation_export: PathBuf,
    pub event_context_validation_report: PathBuf,
    pub event_context_validation_export: PathBuf,
    pub sector_observation_count: usize,
    pub sector_component_observation_count: usize,
    pub stock_observation_count: usize,
    pub industry_stock_observation_count: usize,
    pub macro_regime_snapshot_count: usize,
    pub event_context_observation_count: usize,
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
    let watchlist_rows = db.watchlists_between(&from_date, &to_date)?;
    let regime_scores = db.market_regimes_between(&from_date, &to_date)?;
    let macro_observations = db.macro_observations_through(&to_date)?;

    if sector_scores.is_empty()
        || industry_scores.is_empty()
        || stock_scores.is_empty()
        || sector_maps.is_empty()
        || prices.is_empty()
        || watchlist_rows.is_empty()
        || regime_scores.is_empty()
        || macro_observations.is_empty()
    {
        bail!("missing historical scores or prices; run `merryl run daily --date latest` first");
    }

    let metrics = run_backtest_analysis(BacktestInput {
        from_date: from_date.clone(),
        to_date: to_date.clone(),
        sector_scores: sector_scores.clone(),
        industry_scores,
        stock_scores: stock_scores.clone(),
        sector_maps: sector_maps.clone(),
        prices: prices.clone(),
    })?;
    let macro_regime_metrics = run_macro_regime_validation(MacroRegimeValidationInput {
        from_date: from_date.clone(),
        to_date: to_date.clone(),
        regime_scores,
        sector_scores,
        macro_observations,
    })?;
    let event_context_metrics = run_event_context_validation(EventContextValidationInput {
        from_date: from_date.clone(),
        to_date: to_date.clone(),
        stock_scores: stock_scores.clone(),
        watchlist_rows,
        sector_maps: sector_maps.clone(),
        prices: prices.clone(),
    })?;
    let outputs = write_backtest_outputs(&metrics)?;
    let macro_regime_outputs = write_macro_regime_validation_outputs(&macro_regime_metrics)?;
    let event_context_outputs = write_event_context_validation_outputs(&event_context_metrics)?;
    let run_name = format!("backtest_{}_{}", from_date, to_date);
    let config_json = json!({
        "horizons": scoring::BACKTEST_HORIZONS,
        "validation_scope": &metrics.validation_scope,
        "macro_regime_validation_scope": &macro_regime_metrics.validation_scope,
        "event_context_validation_scope": &event_context_metrics.validation_scope,
        "relative_return_policy": {
            "sector": "sector ETF forward return minus SPY forward return",
            "sector_components": "sector ETF forward return grouped by same-day sector component decile",
            "stock_primary": "stock forward return minus sector ETF forward return",
            "stock_vs_spy": "stock forward return minus SPY forward return",
            "stock_by_industry": "stock forward return grouped by same-day industry/theme score decile",
            "event_context": "watchlist stock forward return grouped by stored same-day catalyst_status labels"
        },
        "source": "SQLite historical scores and daily prices"
    })
    .to_string();
    let mut metrics_json_value =
        serde_json::to_value(&metrics).context("failed to serialize backtest metrics")?;
    metrics_json_value["macro_regime_validation"] = serde_json::to_value(&macro_regime_metrics)
        .context("failed to serialize macro regime validation metrics")?;
    metrics_json_value["event_context_validation"] =
        serde_json::to_value(&event_context_metrics)
            .context("failed to serialize event context validation metrics")?;
    let metrics_json = metrics_json_value.to_string();
    let backtest_result_id =
        db.insert_backtest_result(&run_name, &from_date, &to_date, &config_json, &metrics_json)?;

    Ok(RunBacktestResult {
        from_date,
        to_date,
        database: db_path,
        report: outputs.report,
        summary_export: outputs.summary_export,
        macro_regime_validation_report: macro_regime_outputs.report,
        macro_regime_validation_export: macro_regime_outputs.summary_export,
        event_context_validation_report: event_context_outputs.report,
        event_context_validation_export: event_context_outputs.summary_export,
        sector_observation_count: metrics.sector_observation_count,
        sector_component_observation_count: metrics.sector_component_observation_count,
        stock_observation_count: metrics.stock_observation_count,
        industry_stock_observation_count: metrics.industry_stock_observation_count,
        macro_regime_snapshot_count: macro_regime_metrics.macro_snapshot_count,
        event_context_observation_count: event_context_metrics.forward_observation_count,
        backtest_result_id,
    })
}

fn parse_required_date(value: &str, label: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("{label} must be an explicit YYYY-MM-DD date"))
}
