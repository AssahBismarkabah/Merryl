use crate::config::scoring as scoring_config;
use crate::config::universe::ASSET_STOCK;
use crate::domain::models::{SectorMap, SectorScore, Symbol};

use super::explanations::sector_explanation;
use super::indicators::{
    PriceHistories, clamp_score, effective_index, moving_average, pct_return, relative_volume,
};

pub fn score_sectors(
    date: &str,
    symbols: &[Symbol],
    histories: &PriceHistories,
    sector_maps: &[SectorMap],
) -> Vec<SectorScore> {
    let spy_20d = pct_return(
        histories,
        scoring_config::BENCHMARK_SYMBOL,
        date,
        scoring_config::RETURN_20D,
    )
    .unwrap_or_default();
    let mut scores = Vec::new();

    for sector_map in sector_maps {
        let sector = &sector_map.sector;
        let etf = &sector_map.sector_etf;
        let r1 = pct_return(histories, etf, date, scoring_config::RETURN_1D).unwrap_or_default();
        let r5 = pct_return(histories, etf, date, scoring_config::RETURN_5D).unwrap_or_default();
        let r20 = pct_return(histories, etf, date, scoring_config::RETURN_20D).unwrap_or_default();
        let r60 = pct_return(histories, etf, date, scoring_config::RETURN_60D).unwrap_or_default();
        let relative_return = r20 - spy_20d;
        let relative_volume = relative_volume(histories, etf, date, scoring_config::RETURN_20D)
            .unwrap_or(scoring_config::DEFAULT_RELATIVE_VOLUME);
        let (breadth_20d, breadth_50d) = sector_breadth(symbols, histories, sector, date);

        let relative_return_component = clamp_score(
            scoring_config::NEUTRAL_SCORE
                + relative_return * scoring_config::RELATIVE_RETURN_SCORE_MULTIPLIER,
        );
        let trend_component = clamp_score(
            scoring_config::NEUTRAL_SCORE + r20 * scoring_config::TREND_RETURN_SCORE_MULTIPLIER,
        );
        let relative_volume_component = clamp_score(
            (relative_volume - scoring_config::RELATIVE_VOLUME_BASELINE)
                * scoring_config::RELATIVE_VOLUME_SCORE_MULTIPLIER,
        );
        let breadth_component =
            (breadth_20d + breadth_50d) / scoring_config::BREADTH_COMPONENT_DIVISOR;
        let rank_improvement_component = scoring_config::NEUTRAL_SCORE;

        let score = scoring_config::SECTOR_RELATIVE_RETURN_WEIGHT * relative_return_component
            + scoring_config::SECTOR_TREND_WEIGHT * trend_component
            + scoring_config::SECTOR_RELATIVE_VOLUME_WEIGHT * relative_volume_component
            + scoring_config::SECTOR_BREADTH_WEIGHT * breadth_component
            + scoring_config::SECTOR_RANK_CHANGE_WEIGHT * rank_improvement_component;

        scores.push(SectorScore {
            date: date.to_string(),
            sector: sector.clone(),
            sector_etf: etf.clone(),
            score,
            rank: scoring_config::INITIAL_RANK,
            return_1d: r1,
            return_5d: r5,
            return_20d: r20,
            return_60d: r60,
            relative_return_vs_spy: relative_return,
            relative_volume,
            breadth_20d,
            breadth_50d,
            rank_change: scoring_config::ZERO_RANK_CHANGE,
            explanation: sector_explanation(
                sector,
                score,
                r20,
                relative_return,
                relative_volume,
                breadth_component,
            ),
        });
    }

    scores.sort_by(|a, b| b.score.total_cmp(&a.score));
    for (idx, score) in scores.iter_mut().enumerate() {
        score.rank = idx + scoring_config::FIRST_RANK;
    }
    scores
}

fn sector_breadth(
    symbols: &[Symbol],
    histories: &PriceHistories,
    sector: &str,
    date: &str,
) -> (f64, f64) {
    let members: Vec<&Symbol> = symbols
        .iter()
        .filter(|symbol| symbol.asset_type == ASSET_STOCK)
        .filter(|symbol| symbol.sector.as_deref() == Some(sector))
        .collect();
    if members.is_empty() {
        return (scoring_config::ZERO_PERCENT, scoring_config::ZERO_PERCENT);
    }

    let mut above_20 = 0usize;
    let mut above_50 = 0usize;
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
    }

    if valid == 0 {
        return (scoring_config::ZERO_PERCENT, scoring_config::ZERO_PERCENT);
    }

    (
        above_20 as f64 / valid as f64 * scoring_config::PERCENT_SCALE,
        above_50 as f64 / valid as f64 * scoring_config::PERCENT_SCALE,
    )
}
