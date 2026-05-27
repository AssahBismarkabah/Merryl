use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::Duration as ChronoDuration;

use crate::data::{AlpacaProvider, DailyOhlcvProvider, default_end_date};
use crate::output::write_daily_outputs;
use crate::scoring::{apply_sector_rank_changes, latest_date, score_market};
use crate::storage::{Database, default_db_path};

use super::date_args::parse_date_arg;

#[derive(Debug)]
pub struct RunDailyResult {
    pub date: String,
    pub database: PathBuf,
    pub report: PathBuf,
    pub sector_export: PathBuf,
    pub watchlist_export: PathBuf,
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

    let previous_sector_ranks = db.latest_sector_ranks_before(&score_date)?;
    let previous_watchlist_symbols = db.latest_watchlist_symbols_before(&score_date)?;

    let mut scores = score_market(&score_date, &symbols, &prices, &sector_maps);
    apply_sector_rank_changes(&mut scores.sectors, &previous_sector_ranks);

    if scores.sectors.is_empty() {
        bail!("no sector scores were produced for {score_date}");
    }
    if scores.stocks.is_empty() {
        bail!("no stock watchlist rows were produced for {score_date}");
    }

    db.upsert_symbols(&symbols)?;
    db.upsert_sector_maps(&sector_maps)?;
    db.upsert_industry_maps(&industry_maps)?;
    db.upsert_prices(&prices)?;
    db.replace_market_regime(&scores.regime)?;
    db.replace_sector_scores(&score_date, &scores.sectors)?;
    db.replace_industry_scores(&score_date, &scores.industries)?;
    db.replace_stock_scores(&score_date, &scores.stocks)?;
    db.replace_watchlist(&score_date, &scores.stocks)?;

    let outputs = write_daily_outputs(
        &score_date,
        &scores.regime,
        &scores.sectors,
        &scores.industries,
        &scores.stocks,
        &previous_watchlist_symbols,
    )?;

    Ok(RunDailyResult {
        date: score_date,
        database: db_path,
        report: outputs.report,
        sector_export: outputs.sector_export,
        watchlist_export: outputs.watchlist_export,
    })
}
