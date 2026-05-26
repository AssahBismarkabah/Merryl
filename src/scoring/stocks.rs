use std::collections::HashMap;

use crate::config::scoring as scoring_config;
use crate::config::universe::ASSET_STOCK;
use crate::domain::models::{SectorMap, SectorScore, StockScore, Symbol};

use super::explanations::stock_explanation;
use super::indicators::{
    PriceHistories, avg_dollar_volume, clamp_score, pct_return, relative_volume, trend_state,
};

pub fn score_stocks(
    date: &str,
    symbols: &[Symbol],
    histories: &PriceHistories,
    sector_maps: &[SectorMap],
    sector_scores: &[SectorScore],
) -> Vec<StockScore> {
    let sector_etfs: HashMap<&str, &str> = sector_maps
        .iter()
        .map(|sector_map| (sector_map.sector.as_str(), sector_map.sector_etf.as_str()))
        .collect();
    let sector_score_map: HashMap<&str, f64> = sector_scores
        .iter()
        .map(|score| (score.sector.as_str(), score.score))
        .collect();
    let spy_20d = pct_return(
        histories,
        scoring_config::BENCHMARK_SYMBOL,
        date,
        scoring_config::RETURN_20D,
    )
    .unwrap_or_default();
    let mut scores = Vec::new();

    for symbol in symbols
        .iter()
        .filter(|symbol| symbol.asset_type == ASSET_STOCK)
    {
        let Some(sector) = symbol.sector.as_deref() else {
            continue;
        };
        let Some(industry) = symbol.industry.as_deref() else {
            continue;
        };
        let Some(sector_etf) = sector_etfs.get(sector) else {
            continue;
        };

        let r1 = pct_return(histories, &symbol.symbol, date, scoring_config::RETURN_1D)
            .unwrap_or_default();
        let r5 = pct_return(histories, &symbol.symbol, date, scoring_config::RETURN_5D)
            .unwrap_or_default();
        let r20 = pct_return(histories, &symbol.symbol, date, scoring_config::RETURN_20D)
            .unwrap_or_default();
        let r60 = pct_return(histories, &symbol.symbol, date, scoring_config::RETURN_60D)
            .unwrap_or_default();
        let sector_r20 =
            pct_return(histories, sector_etf, date, scoring_config::RETURN_20D).unwrap_or_default();
        let relative_return_vs_sector = r20 - sector_r20;
        let relative_return_vs_spy = r20 - spy_20d;
        let rel_volume =
            relative_volume(histories, &symbol.symbol, date, scoring_config::RETURN_20D)
                .unwrap_or(scoring_config::DEFAULT_RELATIVE_VOLUME);
        let avg_dollar_volume =
            avg_dollar_volume(histories, &symbol.symbol, date, scoring_config::RETURN_20D)
                .unwrap_or_default();
        let trend_state = trend_state(histories, &symbol.symbol, date);
        let sector_score = sector_score_map
            .get(sector)
            .copied()
            .unwrap_or(scoring_config::NEUTRAL_SCORE);

        if avg_dollar_volume < scoring_config::MIN_AVG_DOLLAR_VOLUME {
            continue;
        }

        let relative_strength_component = clamp_score(
            scoring_config::NEUTRAL_SCORE
                + relative_return_vs_sector * scoring_config::RELATIVE_RETURN_SCORE_MULTIPLIER,
        );
        let relative_volume_component = clamp_score(
            (rel_volume - scoring_config::RELATIVE_VOLUME_BASELINE)
                * scoring_config::RELATIVE_VOLUME_SCORE_MULTIPLIER,
        );
        let trend_component = trend_component(&trend_state);
        let liquidity_component = clamp_score(
            avg_dollar_volume / scoring_config::MIN_AVG_DOLLAR_VOLUME
                * scoring_config::LIQUIDITY_SCORE_SCALE,
        );

        let score = scoring_config::STOCK_SECTOR_WEIGHT * sector_score
            + scoring_config::STOCK_RELATIVE_STRENGTH_WEIGHT * relative_strength_component
            + scoring_config::STOCK_RELATIVE_VOLUME_WEIGHT * relative_volume_component
            + scoring_config::STOCK_TREND_WEIGHT * trend_component
            + scoring_config::STOCK_LIQUIDITY_WEIGHT * liquidity_component;

        scores.push(StockScore {
            date: date.to_string(),
            rank: scoring_config::INITIAL_RANK,
            symbol: symbol.symbol.clone(),
            name: symbol.name.clone(),
            sector: sector.to_string(),
            industry: industry.to_string(),
            score,
            sector_score,
            return_1d: r1,
            return_5d: r5,
            return_20d: r20,
            return_60d: r60,
            relative_return_vs_sector,
            relative_return_vs_spy,
            relative_volume: rel_volume,
            avg_dollar_volume,
            trend_state,
            catalyst_status: scoring_config::CATALYST_PENDING_SOURCE.to_string(),
            explanation: stock_explanation(
                &symbol.symbol,
                score,
                sector,
                relative_return_vs_sector,
                rel_volume,
            ),
        });
    }

    scores.sort_by(|a, b| b.score.total_cmp(&a.score));
    for (idx, score) in scores.iter_mut().enumerate() {
        score.rank = idx + scoring_config::FIRST_RANK;
    }
    scores.truncate(scoring_config::STOCK_WATCHLIST_LIMIT);
    scores
}

fn trend_component(trend_state: &str) -> f64 {
    match trend_state {
        "above_20d_50d" => scoring_config::TREND_ABOVE_20D_50D_SCORE,
        "above_20d" => scoring_config::TREND_ABOVE_20D_SCORE,
        "below_trend" => scoring_config::TREND_BELOW_SCORE,
        _ => scoring_config::NEUTRAL_SCORE,
    }
}
