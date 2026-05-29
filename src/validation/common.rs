use std::collections::{HashMap, HashSet};

use crate::config::scoring;
use crate::domain::models::{DailyPrice, SectorMap, StockScore, WatchlistRow};
use crate::scoring::forward_return;

#[derive(Debug, Clone, Copy)]
pub(super) struct ForwardReturns {
    pub stock: f64,
    pub spy: f64,
    pub sector: f64,
}

pub(super) fn scored_watchlist_rows<'a>(
    stock_scores: &'a [StockScore],
    watchlist_rows: &[WatchlistRow],
) -> Vec<&'a StockScore> {
    let watchlist_keys: HashSet<(&str, &str)> = watchlist_rows
        .iter()
        .map(|row| (row.date.as_str(), row.symbol.as_str()))
        .collect();

    stock_scores
        .iter()
        .filter(|score| watchlist_keys.contains(&(score.date.as_str(), score.symbol.as_str())))
        .collect()
}

pub(super) fn sector_etfs_by_sector(sector_maps: &[SectorMap]) -> HashMap<&str, &str> {
    sector_maps
        .iter()
        .map(|sector_map| (sector_map.sector.as_str(), sector_map.sector_etf.as_str()))
        .collect()
}

pub(super) fn forward_returns_for_horizon(
    histories: &HashMap<String, Vec<DailyPrice>>,
    symbol: &str,
    date: &str,
    sector_etf: &str,
    horizon: usize,
) -> Option<ForwardReturns> {
    let stock = forward_return(histories, symbol, date, horizon)?;
    let spy = forward_return(histories, scoring::BENCHMARK_SYMBOL, date, horizon)?;
    let sector = forward_return(histories, sector_etf, date, horizon)?;
    Some(ForwardReturns { stock, spy, sector })
}

pub(super) fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

pub(super) fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}
