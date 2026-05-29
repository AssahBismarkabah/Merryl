use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::{Duration as ChronoDuration, NaiveDate};

use crate::actionability::refresh_stock_components;
use crate::config::market_data;
use crate::data::{
    AlpacaProvider, AlphaVantageProvider, CatalystEventProvider, DailyOhlcvProvider,
    EarningsCalendarProvider, FilingEventProvider, FredProvider, MacroSeriesProvider,
    SecEdgarProvider, default_end_date,
};
use crate::domain::models::{MarketEvent, StockScore};
use crate::output::{DailyReportInput, write_daily_outputs};
use crate::scoring::{
    apply_catalyst_status, latest_date, preserve_existing_catalyst_statuses,
    previous_watchlist_symbols_for_date, score_market_history,
};
use crate::storage::{Database, default_db_path};
use crate::validation::macro_context_overlay;

use super::date_args::parse_date_arg;

#[derive(Debug)]
pub struct RunDailyResult {
    pub date: String,
    pub database: PathBuf,
    pub report: PathBuf,
    pub sector_export: PathBuf,
    pub watchlist_export: PathBuf,
    pub historical_score_dates: usize,
    pub macro_observations: usize,
    pub news_events: usize,
    pub earnings_events: usize,
    pub filing_events: usize,
}

pub fn run_daily(date_arg: &str) -> Result<RunDailyResult> {
    let requested_date = parse_date_arg(date_arg)?;
    let fetch_end_date = requested_date
        .map(|date| date + ChronoDuration::days(1))
        .unwrap_or_else(default_end_date);

    let provider = AlpacaProvider::from_env()?;
    let macro_provider = FredProvider::from_env()?;
    let earnings_provider = AlphaVantageProvider::from_env()?;
    let filing_provider = SecEdgarProvider::from_env()?;
    let symbols = provider.symbols()?;
    let sector_maps = provider.sector_maps();
    let industry_maps = provider.industry_maps(&symbols);
    let prices = provider.daily_prices(&symbols, fetch_end_date)?;
    let macro_observations = macro_provider.macro_observations(fetch_end_date)?;

    if prices.is_empty() {
        bail!("Alpaca returned no daily prices for the configured universe");
    }
    if macro_observations.is_empty() {
        bail!("FRED returned no macro observations for the configured macro series");
    }

    let score_date = match requested_date {
        Some(date) => date.format("%Y-%m-%d").to_string(),
        None => latest_date(&prices).context("could not determine latest date from price data")?,
    };

    let db_path = default_db_path();
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    let mut score_history = score_market_history(&score_date, &symbols, &prices, &sector_maps);
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

    let report_date_value = NaiveDate::parse_from_str(&report_date, "%Y-%m-%d")
        .with_context(|| format!("invalid report date: {report_date}"))?;
    let news_start_date = (report_date_value
        - ChronoDuration::days(market_data::NEWS_LOOKBACK_DAYS))
    .format("%Y-%m-%d")
    .to_string();
    let news_end_date = report_date.clone();
    let watchlist_symbols = scores
        .stocks
        .iter()
        .map(|stock| stock.symbol.clone())
        .collect::<Vec<_>>();
    let mut recent_events = provider.recent_news_events(&watchlist_symbols, report_date_value)?;
    let mut earnings_events = earnings_provider.upcoming_earnings_events(&watchlist_symbols)?;
    let mut filing_events =
        filing_provider.recent_filing_events(&watchlist_symbols, report_date_value)?;
    let mut structured_events = Vec::new();
    structured_events.extend(earnings_events.clone());
    structured_events.extend(filing_events.clone());
    let mut all_events = Vec::new();
    all_events.extend(recent_events.clone());
    all_events.extend(earnings_events.clone());
    all_events.extend(filing_events.clone());

    if let Some(scores) = score_history.last_mut() {
        attach_event_sectors(&mut recent_events, &scores.stocks);
        attach_event_sectors(&mut earnings_events, &scores.stocks);
        attach_event_sectors(&mut filing_events, &scores.stocks);
        attach_event_sectors(&mut structured_events, &scores.stocks);
        attach_event_sectors(&mut all_events, &scores.stocks);
        apply_catalyst_status(&mut scores.stocks, &all_events);
    }
    let existing_catalyst_statuses = db.non_pending_stock_catalyst_statuses_before(&report_date)?;
    for scores in &mut score_history {
        preserve_existing_catalyst_statuses(
            &mut scores.stocks,
            &existing_catalyst_statuses,
            &report_date,
        );
        for stock in &mut scores.stocks {
            refresh_stock_components(stock);
        }
    }

    db.upsert_symbols(&symbols)?;
    db.upsert_sector_maps(&sector_maps)?;
    db.upsert_industry_maps(&industry_maps)?;
    db.upsert_prices(&prices)?;
    db.upsert_macro_observations(&macro_observations)?;
    for scores in &score_history {
        db.replace_market_regime(&scores.regime)?;
        db.replace_sector_scores(&scores.date, &scores.sectors)?;
        db.replace_industry_scores(&scores.date, &scores.industries)?;
        db.replace_stock_scores(&scores.date, &scores.stocks)?;
        db.replace_watchlist(&scores.date, &scores.stocks)?;
    }
    db.replace_recent_news_events(&news_start_date, &news_end_date, &recent_events)?;
    db.upsert_structured_events(&structured_events)?;

    let scores = score_history
        .last()
        .context("score history unexpectedly became empty")?;
    let macro_context =
        macro_context_overlay(&report_date, &scores.regime.label, &macro_observations)?;

    let outputs = write_daily_outputs(DailyReportInput {
        date: &report_date,
        regime: &scores.regime,
        sector_scores: &scores.sectors,
        industry_scores: &scores.industries,
        stock_scores: &scores.stocks,
        events: &all_events,
        macro_observations: &macro_observations,
        macro_context: Some(&macro_context),
        previous_watchlist_symbols: &previous_watchlist_symbols,
    })?;

    Ok(RunDailyResult {
        date: report_date,
        database: db_path,
        report: outputs.report,
        sector_export: outputs.sector_export,
        watchlist_export: outputs.watchlist_export,
        historical_score_dates: score_history.len(),
        macro_observations: macro_observations.len(),
        news_events: recent_events.len(),
        earnings_events: earnings_events.len(),
        filing_events: filing_events.len(),
    })
}

fn attach_event_sectors(events: &mut [MarketEvent], stock_scores: &[StockScore]) {
    for event in events {
        if let Some(stock) = stock_scores
            .iter()
            .find(|stock| stock.symbol == event.symbol)
        {
            event.sector = Some(stock.sector.clone());
        }
    }
}
