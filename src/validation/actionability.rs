use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::actionability::stored_classification_from_components;
use crate::config::{actionability as actionability_config, scoring};
use crate::domain::models::{DailyPrice, SectorMap, StockScore, WatchlistRow};
use crate::scoring::{forward_return, histories_by_symbol};

#[derive(Debug, Clone)]
pub struct ActionabilityValidationInput {
    pub from_date: String,
    pub to_date: String,
    pub stock_scores: Vec<StockScore>,
    pub watchlist_rows: Vec<WatchlistRow>,
    pub sector_maps: Vec<SectorMap>,
    pub prices: Vec<DailyPrice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionabilityValidationMetrics {
    pub from_date: String,
    pub to_date: String,
    pub validation_scope: ActionabilityValidationScope,
    pub watchlist_row_count: usize,
    pub scored_watchlist_row_count: usize,
    pub extended_leader_row_count: usize,
    pub useful_review_row_count: usize,
    pub unclassified_leader_row_count: usize,
    pub forward_observation_count: usize,
    pub grouped_forward_observation_count: usize,
    pub skipped_missing_future_bars: usize,
    pub summaries: Vec<ActionabilitySummaryRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionabilityValidationScope {
    pub purpose: String,
    pub proves: Vec<String>,
    pub does_not_prove: Vec<String>,
    pub date_alignment_rule: String,
    pub group_policy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionabilitySummaryRow {
    pub group: String,
    pub horizon: usize,
    pub count: usize,
    pub hit_rate: f64,
    pub average_forward_return: f64,
    pub median_forward_return: f64,
    pub average_relative_return_vs_spy: f64,
    pub median_relative_return_vs_spy: f64,
    pub average_relative_return_vs_sector: f64,
    pub median_relative_return_vs_sector: f64,
}

#[derive(Debug, Clone)]
struct ActionabilityObservation {
    group: String,
    horizon: usize,
    forward_return: f64,
    relative_return_vs_spy: f64,
    relative_return_vs_sector: f64,
}

pub fn run_actionability_validation(
    input: ActionabilityValidationInput,
) -> Result<ActionabilityValidationMetrics> {
    if input.stock_scores.is_empty()
        || input.watchlist_rows.is_empty()
        || input.sector_maps.is_empty()
        || input.prices.is_empty()
    {
        bail!("missing historical scores or prices; run `merryl run daily --date latest` first");
    }

    let watchlist_keys: HashSet<(String, String)> = input
        .watchlist_rows
        .iter()
        .map(|row| (row.date.clone(), row.symbol.clone()))
        .collect();
    let scored_watchlist_rows: Vec<&StockScore> = input
        .stock_scores
        .iter()
        .filter(|score| watchlist_keys.contains(&(score.date.clone(), score.symbol.clone())))
        .collect();

    if scored_watchlist_rows.is_empty() {
        bail!("missing historical scores or prices; run `merryl run daily --date latest` first");
    }

    let extended_leader_row_count = scored_watchlist_rows
        .iter()
        .filter(|score| {
            let classification = stored_classification_from_components(&score.components_json);
            classification
                .labels
                .contains(&actionability_config::LABEL_EXTENDED_LEADER.to_string())
        })
        .count();
    let useful_review_row_count = scored_watchlist_rows
        .iter()
        .filter(|score| {
            let classification = stored_classification_from_components(&score.components_json);
            classification.labels.iter().any(|label| {
                label == actionability_config::LABEL_PULLBACK_LEADER
                    || label == actionability_config::LABEL_BASE_COMPRESSION_CANDIDATE
                    || label == actionability_config::LABEL_EARLY_ROTATION_CANDIDATE
                    || label == actionability_config::LABEL_ACTIONABLE_LEADER
            })
        })
        .count();
    let unclassified_leader_row_count = scored_watchlist_rows
        .iter()
        .filter(|score| {
            let classification = stored_classification_from_components(&score.components_json);
            classification
                .labels
                .contains(&actionability_config::LABEL_UNCLASSIFIED_LEADER.to_string())
        })
        .count();

    let histories = histories_by_symbol(&input.prices);
    let sector_etfs: HashMap<&str, &str> = input
        .sector_maps
        .iter()
        .map(|sector_map| (sector_map.sector.as_str(), sector_map.sector_etf.as_str()))
        .collect();
    let mut observations = Vec::new();
    let mut forward_observation_count = 0;
    let mut skipped_missing_future_bars = 0;

    for score in scored_watchlist_rows.iter() {
        let Some(sector_etf) = sector_etfs.get(score.sector.as_str()) else {
            skipped_missing_future_bars += scoring::BACKTEST_HORIZONS.len();
            continue;
        };
        for horizon in scoring::BACKTEST_HORIZONS {
            let Some(stock_return) =
                forward_return(&histories, &score.symbol, &score.date, *horizon)
            else {
                skipped_missing_future_bars += 1;
                continue;
            };
            let Some(spy_return) =
                forward_return(&histories, scoring::BENCHMARK_SYMBOL, &score.date, *horizon)
            else {
                skipped_missing_future_bars += 1;
                continue;
            };
            let Some(sector_return) = forward_return(&histories, sector_etf, &score.date, *horizon)
            else {
                skipped_missing_future_bars += 1;
                continue;
            };

            forward_observation_count += 1;
            observations.extend(
                actionability_groups(&score.components_json)
                    .into_iter()
                    .map(|group| ActionabilityObservation {
                        group,
                        horizon: *horizon,
                        forward_return: stock_return,
                        relative_return_vs_spy: stock_return - spy_return,
                        relative_return_vs_sector: stock_return - sector_return,
                    }),
            );
        }
    }

    let grouped_forward_observation_count = observations.len();
    let summaries = summarize_observations(&observations);

    Ok(ActionabilityValidationMetrics {
        from_date: input.from_date,
        to_date: input.to_date,
        validation_scope: validation_scope(),
        watchlist_row_count: input.watchlist_rows.len(),
        scored_watchlist_row_count: scored_watchlist_rows.len(),
        extended_leader_row_count,
        useful_review_row_count,
        unclassified_leader_row_count,
        forward_observation_count,
        grouped_forward_observation_count,
        skipped_missing_future_bars,
        summaries,
    })
}

fn validation_scope() -> ActionabilityValidationScope {
    ActionabilityValidationScope {
        purpose: "watchlist_actionability_validation".to_string(),
        proves: vec![
            "Whether stored actionability buckets behaved differently over forward trading-bar horizons.".to_string(),
            "Whether extended leaders underperformed, outperformed, or simply required different review treatment.".to_string(),
            "Whether early, base, pullback, and actionable buckets are useful as watchlist review groups before any score-weight change.".to_string(),
        ],
        does_not_prove: vec![
            "Trade profitability.".to_string(),
            "That actionability labels should change Merryl score weights.".to_string(),
            "Entry timing, exits, transaction costs, slippage, position sizing, or portfolio behavior.".to_string(),
            "That a bucket is automatically bullish or bearish.".to_string(),
        ],
        date_alignment_rule: "Use stored components_json actionability labels on the same score date; forward returns use only future trading bars.".to_string(),
        group_policy: "Groups are inclusive: one watchlist row appears in all_watchlist plus its stored primary bucket and stored actionability labels.".to_string(),
    }
}

fn actionability_groups(components_json: &str) -> Vec<String> {
    let classification = stored_classification_from_components(components_json);
    let mut groups = vec![actionability_config::LABEL_ALL_WATCHLIST.to_string()];
    groups.push(classification.primary);
    groups.extend(classification.labels);
    groups.sort();
    groups.dedup();
    groups
}

fn summarize_observations(
    observations: &[ActionabilityObservation],
) -> Vec<ActionabilitySummaryRow> {
    let mut groups: HashMap<(&str, usize), Vec<&ActionabilityObservation>> = HashMap::new();
    for observation in observations {
        groups
            .entry((observation.group.as_str(), observation.horizon))
            .or_default()
            .push(observation);
    }

    let mut summaries: Vec<ActionabilitySummaryRow> = groups
        .into_iter()
        .map(|((group, horizon), rows)| {
            let forward_returns: Vec<f64> = rows.iter().map(|row| row.forward_return).collect();
            let relative_returns_vs_spy: Vec<f64> =
                rows.iter().map(|row| row.relative_return_vs_spy).collect();
            let relative_returns_vs_sector: Vec<f64> = rows
                .iter()
                .map(|row| row.relative_return_vs_sector)
                .collect();
            let wins = relative_returns_vs_sector
                .iter()
                .filter(|relative_return| **relative_return > 0.0)
                .count();

            ActionabilitySummaryRow {
                group: group.to_string(),
                horizon,
                count: rows.len(),
                hit_rate: wins as f64 / rows.len() as f64,
                average_forward_return: average(&forward_returns),
                median_forward_return: median(forward_returns),
                average_relative_return_vs_spy: average(&relative_returns_vs_spy),
                median_relative_return_vs_spy: median(relative_returns_vs_spy),
                average_relative_return_vs_sector: average(&relative_returns_vs_sector),
                median_relative_return_vs_sector: median(relative_returns_vs_sector),
            }
        })
        .collect();

    summaries.sort_by(|a, b| a.group.cmp(&b.group).then(a.horizon.cmp(&b.horizon)));
    summaries
}

fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}
