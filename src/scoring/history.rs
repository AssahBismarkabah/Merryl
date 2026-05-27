use std::collections::{HashMap, HashSet};

use crate::config::scoring as scoring_config;
use crate::domain::models::{DailyPrice, SectorMap, SectorScore, StockScore, Symbol};

use super::indicators::{PriceHistories, histories_by_symbol};
use super::market::{MarketScores, score_market_from_histories};
use super::sectors::apply_sector_rank_changes;

pub fn score_market_history(
    through_date: &str,
    symbols: &[Symbol],
    prices: &[DailyPrice],
    sector_maps: &[SectorMap],
) -> Vec<MarketScores> {
    let histories = histories_by_symbol(prices);
    let dates = valid_score_dates(&histories, through_date);
    let mut previous_sector_ranks = HashMap::new();
    let mut history = Vec::new();

    for date in dates {
        let mut scores = score_market_from_histories(&date, symbols, &histories, sector_maps);
        apply_sector_rank_changes(&mut scores.sectors, &previous_sector_ranks);
        previous_sector_ranks = sector_rank_map(&scores.sectors);
        history.push(scores);
    }

    history
}

pub fn previous_watchlist_symbols_for_date(
    history: &[MarketScores],
    date: &str,
) -> HashSet<String> {
    let Some(idx) = history.iter().position(|scores| scores.date == date) else {
        return HashSet::new();
    };
    if idx == 0 {
        return HashSet::new();
    }

    watchlist_symbols(&history[idx - 1].stocks)
}

fn valid_score_dates(histories: &PriceHistories, through_date: &str) -> Vec<String> {
    histories
        .get(scoring_config::BENCHMARK_SYMBOL)
        .map(|history| {
            history
                .iter()
                .enumerate()
                .filter(|(idx, price)| {
                    *idx >= scoring_config::RETURN_60D && price.date.as_str() <= through_date
                })
                .map(|(_, price)| price.date.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn sector_rank_map(scores: &[SectorScore]) -> HashMap<String, usize> {
    scores
        .iter()
        .map(|score| (score.sector.clone(), score.rank))
        .collect()
}

fn watchlist_symbols(scores: &[StockScore]) -> HashSet<String> {
    scores
        .iter()
        .take(scoring_config::REPORT_WATCHLIST_LIMIT)
        .map(|score| score.symbol.clone())
        .collect()
}
