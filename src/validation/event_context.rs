use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::Serialize;

use crate::config::{event_validation, scoring};
use crate::domain::models::{DailyPrice, SectorMap, StockScore, WatchlistRow};
use crate::scoring::histories_by_symbol;

use super::common::{
    average, forward_returns_for_horizon, median, scored_watchlist_rows, sector_etfs_by_sector,
};

#[derive(Debug, Clone)]
pub struct EventContextValidationInput {
    pub from_date: String,
    pub to_date: String,
    pub stock_scores: Vec<StockScore>,
    pub watchlist_rows: Vec<WatchlistRow>,
    pub sector_maps: Vec<SectorMap>,
    pub prices: Vec<DailyPrice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventContextValidationMetrics {
    pub from_date: String,
    pub to_date: String,
    pub validation_scope: EventContextValidationScope,
    pub watchlist_row_count: usize,
    pub scored_watchlist_row_count: usize,
    pub event_context_row_count: usize,
    pub pending_source_row_count: usize,
    pub forward_observation_count: usize,
    pub event_context_forward_observation_count: usize,
    pub grouped_forward_observation_count: usize,
    pub skipped_missing_future_bars: usize,
    pub summaries: Vec<EventContextSummaryRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventContextValidationScope {
    pub purpose: String,
    pub proves: Vec<String>,
    pub does_not_prove: Vec<String>,
    pub date_alignment_rule: String,
    pub group_policy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventContextSummaryRow {
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
struct EventContextObservation {
    group: String,
    horizon: usize,
    forward_return: f64,
    relative_return_vs_spy: f64,
    relative_return_vs_sector: f64,
}

pub fn run_event_context_validation(
    input: EventContextValidationInput,
) -> Result<EventContextValidationMetrics> {
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

    let event_context_row_count = scored_watchlist_rows
        .iter()
        .filter(|score| has_event_context(&score.catalyst_status))
        .count();
    let pending_source_row_count = scored_watchlist_rows.len() - event_context_row_count;
    let histories = histories_by_symbol(&input.prices);
    let sector_etfs = sector_etfs_by_sector(&input.sector_maps);
    let mut observations = Vec::new();
    let mut forward_observation_count = 0;
    let mut event_context_forward_observation_count = 0;
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
            if has_event_context(&score.catalyst_status) {
                event_context_forward_observation_count += 1;
            }
            observations.extend(
                event_groups(&score.catalyst_status)
                    .into_iter()
                    .map(|group| EventContextObservation {
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

    Ok(EventContextValidationMetrics {
        from_date: input.from_date,
        to_date: input.to_date,
        validation_scope: validation_scope(),
        watchlist_row_count: input.watchlist_rows.len(),
        scored_watchlist_row_count: scored_watchlist_rows.len(),
        event_context_row_count,
        pending_source_row_count,
        forward_observation_count,
        event_context_forward_observation_count,
        grouped_forward_observation_count,
        skipped_missing_future_bars,
        summaries,
    })
}

fn validation_scope() -> EventContextValidationScope {
    EventContextValidationScope {
        purpose: "event_context_watchlist_validation".to_string(),
        proves: vec![
            "Whether stored watchlist rows with event context behaved differently from pending-source rows over forward trading-bar horizons.".to_string(),
            "Whether recent-news, earnings, filing, event-risk, and multi-event labels had different forward behavior in stored daily data.".to_string(),
            "Whether event context is useful as a watchlist classification and review layer before any score-weight change.".to_string(),
        ],
        does_not_prove: vec![
            "Trade profitability.".to_string(),
            "That event context is bullish or bearish.".to_string(),
            "That catalyst labels should change Merryl score weights.".to_string(),
            "Entry timing, exits, transaction costs, slippage, or portfolio behavior.".to_string(),
        ],
        date_alignment_rule: "Use the stored catalyst_status on the same score date; forward returns use only future trading bars.".to_string(),
        group_policy: "Groups are inclusive: one watchlist row can appear in all_watchlist, event_context, earnings, filing, event_risk, and multiple_event_types.".to_string(),
    }
}

fn summarize_observations(observations: &[EventContextObservation]) -> Vec<EventContextSummaryRow> {
    let mut groups: HashMap<(&str, usize), Vec<&EventContextObservation>> = HashMap::new();
    for observation in observations {
        groups
            .entry((observation.group.as_str(), observation.horizon))
            .or_default()
            .push(observation);
    }

    let mut summaries: Vec<EventContextSummaryRow> = groups
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

            EventContextSummaryRow {
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

fn event_groups(catalyst_status: &str) -> Vec<String> {
    let mut groups = vec![event_validation::GROUP_ALL_WATCHLIST.to_string()];
    if !has_event_context(catalyst_status) {
        groups.push(event_validation::GROUP_PENDING_SOURCE.to_string());
        return groups;
    }

    groups.push(event_validation::GROUP_EVENT_CONTEXT.to_string());
    let mut event_type_count = 0;
    if has_label(catalyst_status, scoring::CATALYST_RECENT_NEWS_PREFIX) {
        event_type_count += 1;
        groups.push(event_validation::GROUP_RECENT_NEWS.to_string());
    }
    if has_label(catalyst_status, scoring::CATALYST_EARNINGS_PREFIX) {
        event_type_count += 1;
        groups.push(event_validation::GROUP_EARNINGS.to_string());
    }
    if has_label(catalyst_status, scoring::CATALYST_FILING_PREFIX) {
        event_type_count += 1;
        groups.push(event_validation::GROUP_FILING.to_string());
    }
    if has_label(catalyst_status, scoring::CATALYST_EARNINGS_PREFIX)
        || has_label(catalyst_status, scoring::CATALYST_FILING_PREFIX)
    {
        groups.push(event_validation::GROUP_EVENT_RISK.to_string());
    }
    if event_type_count >= 2 {
        groups.push(event_validation::GROUP_MULTIPLE_EVENT_TYPES.to_string());
    }

    groups
}

fn has_event_context(catalyst_status: &str) -> bool {
    catalyst_status != scoring::CATALYST_PENDING_SOURCE
}

fn has_label(catalyst_status: &str, prefix: &str) -> bool {
    catalyst_status.contains(&format!("{prefix}:"))
}
