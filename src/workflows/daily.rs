use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::Duration as ChronoDuration;

use crate::data::{AlpacaProvider, DailyOhlcvProvider, default_end_date};
use crate::output::write_daily_outputs;
use crate::scoring::{latest_date, previous_watchlist_symbols_for_date, score_market_history};
use crate::storage::{Database, default_db_path};

use super::date_args::parse_date_arg;

#[derive(Debug)]
pub struct RunDailyResult {
    pub date: String,
    pub database: PathBuf,
    pub report: PathBuf,
    pub sector_export: PathBuf,
    pub watchlist_export: PathBuf,
    pub historical_score_dates: usize,
}

pub fn run_daily(date_arg: &str) -> Result<RunDailyResult> {
    let requested_date = parse_date_arg(date_arg)?;
    let fetch_end_date = requested_date
        .map(|date| date + ChronoDuration::days(1))
        .unwrap_or_else(default_end_date);

    let provider = AlpacaProvider::from_env()?;
    let symbols = provider.symbols()?;
    let sector_maps = provider.sector_maps();
    let industry_maps = provider.industry_maps(&symbols);
    let prices = provider.daily_prices(&symbols, fetch_end_date)?;

    if prices.is_empty() {
        bail!("Alpaca returned no daily prices for the configured universe");
    }

    let score_date = match requested_date {
        Some(date) => date.format("%Y-%m-%d").to_string(),
        None => latest_date(&prices).context("could not determine latest date from price data")?,
    };

    let db_path = default_db_path();
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    let score_history = score_market_history(&score_date, &symbols, &prices, &sector_maps);
    let scores = score_history.last().context(
        "no valid historical score dates were produced; need at least 60 benchmark bars",
    )?;
    let report_date = scores.date.clone();
    let previous_watchlist_symbols =
        previous_watchlist_symbols_for_date(&score_history, &report_date);

    if scores.sectors.is_empty() {
        bail!("no sector scores were produced for {report_date}");
    }
    if scores.stocks.is_empty() {
        bail!("no stock watchlist rows were produced for {report_date}");
    }

    db.upsert_symbols(&symbols)?;
    db.upsert_sector_maps(&sector_maps)?;
    db.upsert_industry_maps(&industry_maps)?;
    db.upsert_prices(&prices)?;
    for scores in &score_history {
        db.replace_market_regime(&scores.regime)?;
        db.replace_sector_scores(&scores.date, &scores.sectors)?;
        db.replace_industry_scores(&scores.date, &scores.industries)?;
        db.replace_stock_scores(&scores.date, &scores.stocks)?;
        db.replace_watchlist(&scores.date, &scores.stocks)?;
    }

    let outputs = write_daily_outputs(
        &report_date,
        &scores.regime,
        &scores.sectors,
        &scores.industries,
        &scores.stocks,
        &previous_watchlist_symbols,
    )?;

    Ok(RunDailyResult {
        date: report_date,
        database: db_path,
        report: outputs.report,
        sector_export: outputs.sector_export,
        watchlist_export: outputs.watchlist_export,
        historical_score_dates: score_history.len(),
    })
}
