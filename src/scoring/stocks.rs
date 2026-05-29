use std::collections::HashMap;

use serde_json::json;

use crate::actionability::{
    ActionabilityInput, ActionabilityMetrics, components_with_actionability,
};
use crate::config::actionability as actionability_config;
use crate::config::scoring as scoring_config;
use crate::config::universe::ASSET_STOCK;
use crate::domain::models::{SectorMap, SectorScore, StockScore, Symbol};

use super::explanations::stock_explanation;
use super::indicators::{
    PriceHistories, average_true_range, avg_dollar_volume, clamp_score, distance_pct,
    effective_index, gap_pct, highest_close, lowest_close, moving_average, pct_return, range_pct,
    relative_volume, trend_state, true_range,
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
        let actionability_metrics =
            stock_actionability_metrics(histories, &symbol.symbol, date).unwrap_or_default();
        let actionability_input = ActionabilityInput {
            score,
            sector_score,
            return_1d: r1,
            return_5d: r5,
            relative_return_vs_sector,
            relative_return_vs_spy,
            relative_volume: rel_volume,
            trend_state: &trend_state,
            catalyst_status: scoring_config::CATALYST_PENDING_SOURCE,
        };
        let components_json = components_with_actionability(
            json!({
                "sector_score": sector_score,
                "relative_strength_component": relative_strength_component,
                "relative_volume_component": relative_volume_component,
                "trend_component": trend_component,
                "liquidity_component": liquidity_component,
                "relative_return_vs_sector": relative_return_vs_sector,
                "relative_return_vs_spy": relative_return_vs_spy,
                "avg_dollar_volume": avg_dollar_volume
            }),
            &actionability_input,
            &actionability_metrics,
        )
        .to_string();

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
            components_json,
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

fn stock_actionability_metrics(
    histories: &PriceHistories,
    symbol: &str,
    date: &str,
) -> Option<ActionabilityMetrics> {
    let history = histories.get(symbol)?;
    let idx = effective_index(history, date)?;
    let price = history.get(idx)?;
    let close = price.adjusted_close;
    let raw_close = price.close;
    let ma_20d = moving_average(history, idx, scoring_config::RETURN_20D)?;
    let ma_50d = moving_average(history, idx, scoring_config::RETURN_50D)?;
    let atr_14d = average_true_range(history, idx, actionability_config::ATR_LOOKBACK)?;
    let high_20d = highest_close(history, idx, actionability_config::HIGH_20D_LOOKBACK)?;
    let high_60d = highest_close(history, idx, actionability_config::HIGH_60D_LOOKBACK)?;
    let high_10d = highest_close(history, idx, actionability_config::RANGE_10D_LOOKBACK)?;
    let low_10d = lowest_close(history, idx, actionability_config::RANGE_10D_LOOKBACK)?;
    let true_range = true_range(history, idx)?;

    Some(ActionabilityMetrics {
        ma_20d,
        ma_50d,
        distance_from_20d_ma_pct: distance_pct(close, ma_20d)?,
        distance_from_50d_ma_pct: distance_pct(close, ma_50d)?,
        atr_14d,
        atr_14d_pct: distance_pct(raw_close + atr_14d, raw_close)?,
        atr_extension_from_20d_ma: if atr_14d == 0.0 {
            0.0
        } else {
            (close - ma_20d) / atr_14d
        },
        atr_extension_from_50d_ma: if atr_14d == 0.0 {
            0.0
        } else {
            (close - ma_50d) / atr_14d
        },
        high_20d,
        high_60d,
        distance_from_20d_high_pct: distance_pct(close, high_20d)?,
        distance_from_60d_high_pct: distance_pct(close, high_60d)?,
        range_10d_pct: range_pct(high_10d, low_10d, close)?,
        gap_pct: gap_pct(history, idx).unwrap_or_default(),
        true_range_pct: distance_pct(raw_close + true_range, raw_close)?,
        complete: true,
    })
}
