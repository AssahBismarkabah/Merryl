use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;

use crate::config::intraday;
use crate::data::{AlpacaProvider, IntradayOhlcvProvider};
use crate::intraday::{
    IntradayReadinessInput, high_momentum_candidate_symbols, run_intraday_readiness,
    validate_intraday_timeframe,
};
use crate::output::write_intraday_outputs;
use crate::storage::{Database, default_db_path};

use super::date_args::parse_date_arg;

#[derive(Debug)]
pub struct RunIntradayResult {
    pub date: String,
    pub database: PathBuf,
    pub profile_timeframe: String,
    pub trigger_timeframe: String,
    pub candidate_count: usize,
    pub stage1_count: usize,
    pub stage2_count: usize,
    pub stage3_trigger_count: usize,
    pub report: PathBuf,
    pub export: PathBuf,
}

pub fn run_intraday(date_arg: &str) -> Result<RunIntradayResult> {
    let db_path = default_db_path();
    if !db_path.exists() {
        bail!("missing daily market map; run `merryl run daily --date latest` first");
    }

    let mut db = Database::open(&db_path)?;
    db.migrate()?;
    let date = resolve_score_date(&db, date_arg)?;
    let score_date = date.format("%Y-%m-%d").to_string();
    if db.sector_scores_for_date(&score_date)?.is_empty()
        || db.stock_scores_for_date(&score_date, 1)?.is_empty()
    {
        bail!(
            "missing daily market map for {score_date}; run `merryl run daily --date latest` first"
        );
    }

    let config = IntradayWorkflowConfig::from_env()?;
    let symbols = db.active_symbols()?;
    let sector_maps = db.sector_maps()?;
    let daily_prices = db.daily_prices()?;
    let (_, candidate_symbols) = high_momentum_candidate_symbols(
        &symbols,
        &sector_maps,
        &daily_prices,
        &score_date,
        config.candidate_limit,
    );

    let provider = AlpacaProvider::from_env()?;
    let profile_prices = if candidate_symbols.is_empty() {
        Vec::new()
    } else {
        provider.intraday_prices(&candidate_symbols, date, &config.profile_timeframe)?
    };
    db.upsert_intraday_prices(&profile_prices)?;

    let stage2_preview = run_intraday_readiness(IntradayReadinessInput {
        date: score_date.clone(),
        symbols: symbols.clone(),
        sector_maps: sector_maps.clone(),
        daily_prices: daily_prices.clone(),
        profile_prices: profile_prices.clone(),
        trigger_prices: Vec::new(),
        profile_timeframe: config.profile_timeframe.clone(),
        trigger_timeframe: config.trigger_timeframe.clone(),
        candidate_limit: config.candidate_limit,
        opening_range_minutes: config.opening_range_minutes,
    })?;
    let stage2_symbols = stage2_preview
        .setups
        .iter()
        .filter(|setup| setup.stage2_passed)
        .map(|setup| setup.symbol.clone())
        .collect::<Vec<_>>();
    let trigger_prices = if stage2_symbols.is_empty() {
        Vec::new()
    } else {
        provider.intraday_prices(&stage2_symbols, date, &config.trigger_timeframe)?
    };
    db.upsert_intraday_prices(&trigger_prices)?;

    let result = run_intraday_readiness(IntradayReadinessInput {
        date: score_date.clone(),
        symbols,
        sector_maps,
        daily_prices,
        profile_prices,
        trigger_prices,
        profile_timeframe: config.profile_timeframe.clone(),
        trigger_timeframe: config.trigger_timeframe.clone(),
        candidate_limit: config.candidate_limit,
        opening_range_minutes: config.opening_range_minutes,
    })?;
    db.replace_intraday_readiness(
        &score_date,
        &result.profiles,
        &result.setups,
        &result.triggers,
    )?;
    let outputs = write_intraday_outputs(&result)?;

    Ok(RunIntradayResult {
        date: score_date,
        database: db_path,
        profile_timeframe: config.profile_timeframe,
        trigger_timeframe: config.trigger_timeframe,
        candidate_count: candidate_symbols.len(),
        stage1_count: result.stage1_count,
        stage2_count: result.stage2_count,
        stage3_trigger_count: result.stage3_trigger_count,
        report: outputs.report,
        export: outputs.export,
    })
}

fn resolve_score_date(db: &Database, date_arg: &str) -> Result<NaiveDate> {
    match parse_date_arg(date_arg)? {
        Some(date) => Ok(date),
        None => {
            let Some(date) = db.latest_scored_date()? else {
                bail!("missing daily market map; run `merryl run daily --date latest` first");
            };
            NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .with_context(|| format!("stored score date `{date}` is invalid"))
        }
    }
}

struct IntradayWorkflowConfig {
    profile_timeframe: String,
    trigger_timeframe: String,
    candidate_limit: usize,
    opening_range_minutes: usize,
}

impl IntradayWorkflowConfig {
    fn from_env() -> Result<Self> {
        let profile_timeframe = env::var(intraday::PROFILE_TIMEFRAME_ENV)
            .unwrap_or_else(|_| intraday::DEFAULT_PROFILE_TIMEFRAME.to_string());
        let trigger_timeframe = env::var(intraday::TRIGGER_TIMEFRAME_ENV)
            .unwrap_or_else(|_| intraday::DEFAULT_TRIGGER_TIMEFRAME.to_string());
        validate_intraday_timeframe(&profile_timeframe)?;
        validate_intraday_timeframe(&trigger_timeframe)?;
        let candidate_limit = env::var(intraday::CANDIDATE_LIMIT_ENV)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(intraday::DEFAULT_CANDIDATE_LIMIT)
            .max(1);
        let opening_range_minutes = env::var(intraday::OPENING_RANGE_MINUTES_ENV)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(intraday::DEFAULT_OPENING_RANGE_MINUTES)
            .max(1);

        Ok(Self {
            profile_timeframe,
            trigger_timeframe,
            candidate_limit,
            opening_range_minutes,
        })
    }
}
