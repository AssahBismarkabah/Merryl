use std::collections::HashMap;

use serde_json::json;

use crate::config::scoring as scoring_config;
use crate::config::universe::ASSET_STOCK;
use crate::domain::models::{IndustryScore, Symbol};

use super::indicators::{PriceHistories, average, clamp_score, pct_return};

pub fn score_industries(
    date: &str,
    symbols: &[Symbol],
    histories: &PriceHistories,
) -> Vec<IndustryScore> {
    let mut groups: HashMap<(String, String), Vec<&Symbol>> = HashMap::new();
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
        let returns: Vec<f64> = members
            .iter()
            .filter_map(|symbol| {
                pct_return(histories, &symbol.symbol, date, scoring_config::RETURN_20D)
            })
            .collect();
        let avg_return = average(&returns).unwrap_or_default();
        let score = clamp_score(
            scoring_config::NEUTRAL_SCORE
                + avg_return * scoring_config::INDUSTRY_RETURN_SCORE_MULTIPLIER,
        );
        scores.push(IndustryScore {
            date: date.to_string(),
            industry: industry.clone(),
            sector: sector.clone(),
            score,
            rank: scoring_config::INITIAL_RANK,
            components_json: json!({
                "avg_20d_return": avg_return,
                "member_count": members.len()
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
