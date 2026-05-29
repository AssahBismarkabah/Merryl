use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::Serialize;

use crate::actionability::stored_classification_from_components;
use crate::config::{actionability as actionability_config, scoring};
use crate::domain::models::{DailyPrice, SectorMap, StockScore, WatchlistRow};
use crate::scoring::histories_by_symbol;

use super::common::{
    average, forward_returns_for_horizon, median, scored_watchlist_rows, sector_etfs_by_sector,
};

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

#[derive(Debug, Clone, Default)]
struct ActionabilityRowCounts {
    extended_leader: usize,
    useful_review: usize,
    unclassified_leader: usize,
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

    let scored_watchlist_rows = scored_watchlist_rows(&input.stock_scores, &input.watchlist_rows);

    if scored_watchlist_rows.is_empty() {
        bail!("missing historical scores or prices; run `merryl run daily --date latest` first");
    }

    let row_counts = actionability_row_counts(&scored_watchlist_rows);
    let histories = histories_by_symbol(&input.prices);
    let sector_etfs = sector_etfs_by_sector(&input.sector_maps);
    let mut observations = Vec::new();
    let mut forward_observation_count = 0;
    let mut skipped_missing_future_bars = 0;

    for score in scored_watchlist_rows.iter() {
        let Some(sector_etf) = sector_etfs.get(score.sector.as_str()) else {
            skipped_missing_future_bars += scoring::BACKTEST_HORIZONS.len();
            continue;
        };
        for horizon in scoring::BACKTEST_HORIZONS {
            let Some(forward_returns) = forward_returns_for_horizon(
                &histories,
                &score.symbol,
                &score.date,
                sector_etf,
                *horizon,
            ) else {
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
                        forward_return: forward_returns.stock,
                        relative_return_vs_spy: forward_returns.stock - forward_returns.spy,
                        relative_return_vs_sector: forward_returns.stock - forward_returns.sector,
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
        extended_leader_row_count: row_counts.extended_leader,
        useful_review_row_count: row_counts.useful_review,
        unclassified_leader_row_count: row_counts.unclassified_leader,
        forward_observation_count,
        grouped_forward_observation_count,
        skipped_missing_future_bars,
        summaries,
    })
}

fn actionability_row_counts(scored_watchlist_rows: &[&StockScore]) -> ActionabilityRowCounts {
    let mut counts = ActionabilityRowCounts::default();

    for score in scored_watchlist_rows {
        let classification = stored_classification_from_components(&score.components_json);
        if has_label(
            &classification.labels,
            actionability_config::LABEL_EXTENDED_LEADER,
        ) {
            counts.extended_leader += 1;
        }
        if has_useful_review_label(&classification.labels) {
            counts.useful_review += 1;
        }
        if has_label(
            &classification.labels,
            actionability_config::LABEL_UNCLASSIFIED_LEADER,
        ) {
            counts.unclassified_leader += 1;
        }
    }

    counts
}

fn has_useful_review_label(labels: &[String]) -> bool {
    [
        actionability_config::LABEL_PULLBACK_LEADER,
        actionability_config::LABEL_BASE_COMPRESSION_CANDIDATE,
        actionability_config::LABEL_EARLY_ROTATION_CANDIDATE,
        actionability_config::LABEL_ACTIONABLE_LEADER,
    ]
    .iter()
    .any(|label| has_label(labels, label))
}

fn has_label(labels: &[String], expected: &str) -> bool {
    labels.iter().any(|label| label == expected)
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
