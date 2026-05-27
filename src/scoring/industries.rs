use std::collections::HashMap;

use serde_json::json;

use crate::config::scoring as scoring_config;
use crate::config::universe::ASSET_STOCK;
use crate::domain::models::{DailyPrice, IndustryScore, SectorMap, Symbol};

use super::indicators::{
    PriceHistories, average, clamp_score, effective_index, moving_average, pct_return,
    relative_volume,
};

pub fn score_industries(
    date: &str,
    symbols: &[Symbol],
    histories: &PriceHistories,
    sector_maps: &[SectorMap],
) -> Vec<IndustryScore> {
    let mut groups: HashMap<(String, String), Vec<&Symbol>> = HashMap::new();
    let sector_etfs: HashMap<&str, &str> = sector_maps
        .iter()
        .map(|sector_map| (sector_map.sector.as_str(), sector_map.sector_etf.as_str()))
        .collect();
    let spy_20d = pct_return(
        histories,
        scoring_config::BENCHMARK_SYMBOL,
        date,
        scoring_config::RETURN_20D,
    )
    .unwrap_or_default();

    for symbol in symbols
        .iter()
        .filter(|symbol| symbol.asset_type == ASSET_STOCK)
    {
        if let (Some(sector), Some(industry)) = (&symbol.sector, &symbol.industry) {
            groups
                .entry((sector.clone(), industry.clone()))
                .or_default()
                .push(symbol);
        }
    }

    let mut scores = Vec::new();
    for ((sector, industry), members) in groups {
        let Some(sector_etf) = sector_etfs.get(sector.as_str()) else {
            continue;
        };

        let return_5d =
            average_industry_return(&members, histories, date, scoring_config::RETURN_5D);
        let return_20d =
            average_industry_return(&members, histories, date, scoring_config::RETURN_20D);
        let return_60d =
            average_industry_return(&members, histories, date, scoring_config::RETURN_60D);
        let sector_return_20d =
            pct_return(histories, sector_etf, date, scoring_config::RETURN_20D).unwrap_or_default();
        let relative_return_vs_sector = return_20d - sector_return_20d;
        let relative_return_vs_spy = return_20d - spy_20d;
        let relative_volume = average_relative_volume(&members, histories, date);
        let (breadth_20d, breadth_50d, high_20d_rate, valid_member_count) =
            industry_participation(&members, histories, date);

        let relative_sector_component = clamp_score(
            scoring_config::NEUTRAL_SCORE
                + relative_return_vs_sector * scoring_config::RELATIVE_RETURN_SCORE_MULTIPLIER,
        );
        let relative_spy_component = clamp_score(
            scoring_config::NEUTRAL_SCORE
                + relative_return_vs_spy * scoring_config::RELATIVE_RETURN_SCORE_MULTIPLIER,
        );
        let breadth_component =
            (breadth_20d + breadth_50d) / scoring_config::BREADTH_COMPONENT_DIVISOR;
        let relative_volume_component = clamp_score(
            (relative_volume - scoring_config::RELATIVE_VOLUME_BASELINE)
                * scoring_config::RELATIVE_VOLUME_SCORE_MULTIPLIER,
        );
        let high_rate_component = high_20d_rate;

        let score = scoring_config::INDUSTRY_RELATIVE_SECTOR_WEIGHT * relative_sector_component
            + scoring_config::INDUSTRY_RELATIVE_SPY_WEIGHT * relative_spy_component
            + scoring_config::INDUSTRY_BREADTH_WEIGHT * breadth_component
            + scoring_config::INDUSTRY_RELATIVE_VOLUME_WEIGHT * relative_volume_component
            + scoring_config::INDUSTRY_HIGH_RATE_WEIGHT * high_rate_component;

        scores.push(IndustryScore {
            date: date.to_string(),
            industry: industry.clone(),
            sector: sector.clone(),
            score,
            rank: scoring_config::INITIAL_RANK,
            return_5d,
            return_20d,
            return_60d,
            relative_return_vs_sector,
            relative_return_vs_spy,
            relative_volume,
            breadth_20d,
            breadth_50d,
            high_20d_rate,
            member_count: members.len(),
            components_json: json!({
                "return_5d": return_5d,
                "return_20d": return_20d,
                "return_60d": return_60d,
                "relative_return_vs_sector": relative_return_vs_sector,
                "relative_return_vs_spy": relative_return_vs_spy,
                "relative_volume": relative_volume,
                "breadth_20d": breadth_20d,
                "breadth_50d": breadth_50d,
                "high_20d_rate": high_20d_rate,
                "member_count": members.len(),
                "valid_member_count": valid_member_count,
                "relative_sector_component": relative_sector_component,
                "relative_spy_component": relative_spy_component,
                "breadth_component": breadth_component,
                "relative_volume_component": relative_volume_component,
                "high_rate_component": high_rate_component
            })
            .to_string(),
        });
    }

    scores.sort_by(|a, b| b.score.total_cmp(&a.score));
    for (idx, score) in scores.iter_mut().enumerate() {
        score.rank = idx + scoring_config::FIRST_RANK;
    }
    scores
}

fn average_industry_return(
    members: &[&Symbol],
    histories: &PriceHistories,
    date: &str,
    lookback: usize,
) -> f64 {
    let returns: Vec<f64> = members
        .iter()
        .filter_map(|symbol| pct_return(histories, &symbol.symbol, date, lookback))
        .collect();
    average(&returns).unwrap_or_default()
}

fn average_relative_volume(members: &[&Symbol], histories: &PriceHistories, date: &str) -> f64 {
    let volumes: Vec<f64> = members
        .iter()
        .filter_map(|symbol| {
            relative_volume(histories, &symbol.symbol, date, scoring_config::RETURN_20D)
        })
        .collect();
    average(&volumes).unwrap_or(scoring_config::DEFAULT_RELATIVE_VOLUME)
}

fn industry_participation(
    members: &[&Symbol],
    histories: &PriceHistories,
    date: &str,
) -> (f64, f64, f64, usize) {
    let mut above_20 = 0usize;
    let mut above_50 = 0usize;
    let mut highs_20 = 0usize;
    let mut valid = 0usize;

    for member in members {
        let Some(history) = histories.get(&member.symbol) else {
            continue;
        };
        let Some(idx) = effective_index(history, date) else {
            continue;
        };
        valid += 1;
        let close = history[idx].adjusted_close;
        if moving_average(history, idx, scoring_config::RETURN_20D).is_some_and(|ma| close > ma) {
            above_20 += 1;
        }
        if moving_average(history, idx, scoring_config::RETURN_50D).is_some_and(|ma| close > ma) {
            above_50 += 1;
        }
        if is_20d_high(history, idx) {
            highs_20 += 1;
        }
    }

    if valid == 0 {
        return (
            scoring_config::ZERO_PERCENT,
            scoring_config::ZERO_PERCENT,
            scoring_config::ZERO_PERCENT,
            valid,
        );
    }

    (
        above_20 as f64 / valid as f64 * scoring_config::PERCENT_SCALE,
        above_50 as f64 / valid as f64 * scoring_config::PERCENT_SCALE,
        highs_20 as f64 / valid as f64 * scoring_config::PERCENT_SCALE,
        valid,
    )
}

fn is_20d_high(history: &[DailyPrice], idx: usize) -> bool {
    if idx + 1 < scoring_config::RETURN_20D {
        return false;
    }

    let start = idx + 1 - scoring_config::RETURN_20D;
    let high = history[start..=idx]
        .iter()
        .map(|price| price.adjusted_close)
        .fold(f64::NEG_INFINITY, f64::max);
    history[idx].adjusted_close >= high
}
