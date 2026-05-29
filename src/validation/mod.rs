use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::Serialize;

use crate::config::{macro_data, macro_validation};
use crate::domain::models::{MacroObservation, MarketRegimeScore, SectorScore};

mod actionability;
mod event_context;

pub use actionability::{
    ActionabilitySummaryRow, ActionabilityValidationInput, ActionabilityValidationMetrics,
    ActionabilityValidationScope, run_actionability_validation,
};
pub use event_context::{
    EventContextSummaryRow, EventContextValidationInput, EventContextValidationMetrics,
    EventContextValidationScope, run_event_context_validation,
};

const SERIES_VIX: &str = "VIXCLS";
const SERIES_DGS10: &str = "DGS10";
const SERIES_T10Y2Y: &str = "T10Y2Y";
const SERIES_CPI: &str = "CPIAUCSL";
const SERIES_UNRATE: &str = "UNRATE";
const SERIES_PAYEMS: &str = "PAYEMS";
const SERIES_CREDIT_SPREAD: &str = "BAMLC0A0CM";
const SERIES_DOLLAR: &str = "DTWEXBGS";
const SERIES_LIQUIDITY: &str = "WALCL";

#[derive(Debug, Clone)]
pub struct MacroRegimeValidationInput {
    pub from_date: String,
    pub to_date: String,
    pub regime_scores: Vec<MarketRegimeScore>,
    pub sector_scores: Vec<SectorScore>,
    pub macro_observations: Vec<MacroObservation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroRegimeValidationMetrics {
    pub from_date: String,
    pub to_date: String,
    pub validation_scope: MacroRegimeValidationScope,
    pub score_date_count: usize,
    pub macro_snapshot_count: usize,
    pub complete_macro_snapshot_count: usize,
    pub missing_macro_snapshot_count: usize,
    pub stale_macro_snapshot_count: usize,
    pub risk_on_with_stress_count: usize,
    pub defensive_or_mixed_with_improving_count: usize,
    pub series_freshness: Vec<MacroSeriesFreshness>,
    pub flag_summaries: Vec<MacroFlagSummary>,
    pub sector_leadership: Vec<MacroSectorLeadershipSummary>,
    pub disagreement_examples: Vec<MacroRegimeDisagreement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroRegimeValidationScope {
    pub purpose: String,
    pub proves: Vec<String>,
    pub does_not_prove: Vec<String>,
    pub date_alignment_rule: String,
    pub revision_limitation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroSeriesFreshness {
    pub series: String,
    pub frequency: String,
    pub score_dates_covered: usize,
    pub missing_score_dates: usize,
    pub stale_score_dates: usize,
    pub average_age_days: Option<f64>,
    pub max_age_days: Option<i64>,
    pub latest_observation_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroFlagSummary {
    pub flag: String,
    pub active_dates: usize,
    pub active_share: f64,
    pub average_regime_score_when_active: Option<f64>,
    pub risk_on_dates_when_active: usize,
    pub defensive_or_mixed_dates_when_active: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroSectorLeadershipSummary {
    pub flag: String,
    pub sector: String,
    pub observations: usize,
    pub top_rank_count: usize,
    pub average_rank: f64,
    pub average_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroRegimeDisagreement {
    pub date: String,
    pub regime_label: String,
    pub regime_score: f64,
    pub active_flags: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroContextOverlay {
    pub date: String,
    pub active_flags: Vec<String>,
    pub stale_series: Vec<String>,
    pub covered_series_count: usize,
    pub required_series_count: usize,
    pub interpretation: String,
}

#[derive(Debug, Clone)]
struct MacroDateSnapshot {
    date: String,
    regime: MarketRegimeScore,
    series: HashMap<String, MacroSeriesSnapshot>,
    active_flags: Vec<String>,
}

#[derive(Debug, Clone)]
struct MacroSeriesSnapshot {
    value: f64,
    previous_value: Option<f64>,
    year_ago_value: Option<f64>,
    observation_date: String,
    age_days: i64,
    stale: bool,
}

pub fn run_macro_regime_validation(
    input: MacroRegimeValidationInput,
) -> Result<MacroRegimeValidationMetrics> {
    if input.regime_scores.is_empty() || input.macro_observations.is_empty() {
        bail!(
            "missing historical scores or macro context; run `merryl run daily --date latest` first"
        );
    }

    let snapshots = macro_snapshots(&input.regime_scores, &input.macro_observations)?;
    if snapshots.is_empty() {
        bail!(
            "missing historical scores or macro context; run `merryl run daily --date latest` first"
        );
    }

    let series_freshness = series_freshness(&snapshots);
    let flag_summaries = flag_summaries(&snapshots);
    let sector_leadership = sector_leadership(&snapshots, &input.sector_scores);
    let disagreement_examples = disagreement_examples(&snapshots);
    let complete_macro_snapshot_count = snapshots
        .iter()
        .filter(|snapshot| is_complete(snapshot))
        .count();
    let stale_macro_snapshot_count = snapshots
        .iter()
        .filter(|snapshot| snapshot.series.values().any(|series| series.stale))
        .count();
    let risk_on_with_stress_count = snapshots
        .iter()
        .filter(|snapshot| is_risk_on(&snapshot.regime.label) && !snapshot.active_flags.is_empty())
        .count();
    let defensive_or_mixed_with_improving_count = snapshots
        .iter()
        .filter(|snapshot| {
            is_defensive_or_mixed(&snapshot.regime.label)
                && snapshot.active_flags.is_empty()
                && is_complete(snapshot)
        })
        .count();

    Ok(MacroRegimeValidationMetrics {
        from_date: input.from_date,
        to_date: input.to_date,
        validation_scope: validation_scope(),
        score_date_count: snapshots.len(),
        macro_snapshot_count: snapshots.len(),
        complete_macro_snapshot_count,
        missing_macro_snapshot_count: snapshots.len() - complete_macro_snapshot_count,
        stale_macro_snapshot_count,
        risk_on_with_stress_count,
        defensive_or_mixed_with_improving_count,
        series_freshness,
        flag_summaries,
        sector_leadership,
        disagreement_examples,
    })
}

pub fn macro_context_overlay(
    date: &str,
    regime_label: &str,
    macro_observations: &[MacroObservation],
) -> Result<MacroContextOverlay> {
    let observations_by_series = observations_by_series(macro_observations);
    let score_date = parse_date(date)?;
    let mut series = HashMap::new();
    for (series_id, _, frequency, _) in macro_data::MACRO_SERIES {
        if let Some(snapshot) =
            as_of_series_snapshot(series_id, frequency, score_date, &observations_by_series)?
        {
            series.insert((*series_id).to_string(), snapshot);
        }
    }

    let active_flags = active_flags(&series);
    let stale_series = stale_series(&series);
    let interpretation = overlay_interpretation(regime_label, &active_flags);

    Ok(MacroContextOverlay {
        date: date.to_string(),
        active_flags,
        stale_series,
        covered_series_count: series.len(),
        required_series_count: macro_data::MACRO_SERIES.len(),
        interpretation,
    })
}

fn validation_scope() -> MacroRegimeValidationScope {
    MacroRegimeValidationScope {
        purpose: "macro_regime_context_validation".to_string(),
        proves: vec![
            "Whether stored FRED macro observations are available as-of historical score dates.".to_string(),
            "Whether ETF-proxy regime labels agree or disagree with simple macro stress flags.".to_string(),
            "Which sectors led during macro stress flag dates in stored historical scores.".to_string(),
        ],
        does_not_prove: vec![
            "Trade profitability.".to_string(),
            "That macro data should already change Merryl scoring weights.".to_string(),
            "Point-in-time vintage correctness for revised macro series.".to_string(),
            "That any macro flag is a buy or sell signal.".to_string(),
        ],
        date_alignment_rule:
            "For every score date, use only macro observations with observation_date <= score_date."
                .to_string(),
        revision_limitation:
            "Stored FRED rows currently use the latest available vintage, not true point-in-time historical vintage."
                .to_string(),
    }
}

fn macro_snapshots(
    regimes: &[MarketRegimeScore],
    observations: &[MacroObservation],
) -> Result<Vec<MacroDateSnapshot>> {
    let observations_by_series = observations_by_series(observations);
    let mut regimes = regimes.to_vec();
    regimes.sort_by(|a, b| a.date.cmp(&b.date));

    regimes
        .into_iter()
        .map(|regime| {
            let date = parse_date(&regime.date)?;
            let mut series = HashMap::new();
            for (series_id, _, frequency, _) in macro_data::MACRO_SERIES {
                if let Some(snapshot) =
                    as_of_series_snapshot(series_id, frequency, date, &observations_by_series)?
                {
                    series.insert((*series_id).to_string(), snapshot);
                }
            }
            let active_flags = active_flags(&series);
            Ok(MacroDateSnapshot {
                date: regime.date.clone(),
                regime,
                series,
                active_flags,
            })
        })
        .collect()
}

fn observations_by_series(
    observations: &[MacroObservation],
) -> HashMap<String, Vec<MacroObservation>> {
    let mut by_series: HashMap<String, Vec<MacroObservation>> = HashMap::new();
    for observation in observations {
        by_series
            .entry(observation.series.clone())
            .or_default()
            .push(observation.clone());
    }
    for rows in by_series.values_mut() {
        rows.sort_by(|a, b| a.date.cmp(&b.date));
    }
    by_series
}

fn as_of_series_snapshot(
    series: &str,
    frequency: &str,
    score_date: NaiveDate,
    observations_by_series: &HashMap<String, Vec<MacroObservation>>,
) -> Result<Option<MacroSeriesSnapshot>> {
    let Some(observations) = observations_by_series.get(series) else {
        return Ok(None);
    };

    let mut latest_idx = None;
    for (idx, observation) in observations.iter().enumerate() {
        if parse_date(&observation.date)? <= score_date {
            latest_idx = Some(idx);
        } else {
            break;
        }
    }

    let Some(idx) = latest_idx else {
        return Ok(None);
    };
    let latest = &observations[idx];
    let observation_date = parse_date(&latest.date)?;
    let age_days = score_date
        .signed_duration_since(observation_date)
        .num_days();
    let previous_value = idx
        .checked_sub(1)
        .and_then(|previous_idx| observations.get(previous_idx))
        .map(|observation| observation.value);
    let year_ago_value = idx
        .checked_sub(12)
        .and_then(|previous_idx| observations.get(previous_idx))
        .map(|observation| observation.value);

    Ok(Some(MacroSeriesSnapshot {
        value: latest.value,
        previous_value,
        year_ago_value,
        observation_date: latest.date.clone(),
        age_days,
        stale: age_days > max_age_days(frequency),
    }))
}

fn active_flags(series: &HashMap<String, MacroSeriesSnapshot>) -> Vec<String> {
    let mut flags = Vec::new();
    if latest_value(series, SERIES_VIX)
        .is_some_and(|value| value >= macro_validation::VIX_STRESS_THRESHOLD)
    {
        flags.push(macro_validation::FLAG_VOLATILITY_STRESS.to_string());
    }
    if rising(series, SERIES_DGS10) {
        flags.push(macro_validation::FLAG_RATE_PRESSURE.to_string());
    }
    if latest_value(series, SERIES_T10Y2Y)
        .is_some_and(|value| value < macro_validation::YIELD_CURVE_INVERSION_THRESHOLD)
    {
        flags.push(macro_validation::FLAG_YIELD_CURVE_INVERSION.to_string());
    }
    if rising(series, SERIES_CREDIT_SPREAD) {
        flags.push(macro_validation::FLAG_CREDIT_STRESS.to_string());
    }
    if rising(series, SERIES_DOLLAR) {
        flags.push(macro_validation::FLAG_DOLLAR_PRESSURE.to_string());
    }
    if falling(series, SERIES_LIQUIDITY) {
        flags.push(macro_validation::FLAG_LIQUIDITY_TIGHTENING.to_string());
    }
    if year_over_year_change(series, SERIES_CPI)
        .is_some_and(|change| change >= macro_validation::CPI_YOY_PRESSURE_THRESHOLD)
    {
        flags.push(macro_validation::FLAG_INFLATION_PRESSURE.to_string());
    }
    if rising(series, SERIES_UNRATE) || falling(series, SERIES_PAYEMS) {
        flags.push(macro_validation::FLAG_LABOR_COOLING.to_string());
    }
    flags
}

fn rising(series: &HashMap<String, MacroSeriesSnapshot>, series_id: &str) -> bool {
    series.get(series_id).is_some_and(|snapshot| {
        snapshot
            .previous_value
            .is_some_and(|previous| snapshot.value - previous > macro_validation::TREND_MIN_DELTA)
    })
}

fn falling(series: &HashMap<String, MacroSeriesSnapshot>, series_id: &str) -> bool {
    series.get(series_id).is_some_and(|snapshot| {
        snapshot
            .previous_value
            .is_some_and(|previous| previous - snapshot.value > macro_validation::TREND_MIN_DELTA)
    })
}

fn latest_value(series: &HashMap<String, MacroSeriesSnapshot>, series_id: &str) -> Option<f64> {
    series.get(series_id).map(|snapshot| snapshot.value)
}

fn year_over_year_change(
    series: &HashMap<String, MacroSeriesSnapshot>,
    series_id: &str,
) -> Option<f64> {
    let snapshot = series.get(series_id)?;
    let year_ago = snapshot.year_ago_value?;
    (year_ago != 0.0).then(|| (snapshot.value / year_ago) - 1.0)
}

fn series_freshness(snapshots: &[MacroDateSnapshot]) -> Vec<MacroSeriesFreshness> {
    macro_data::MACRO_SERIES
        .iter()
        .map(|(series, _, frequency, _)| {
            let rows: Vec<&MacroSeriesSnapshot> = snapshots
                .iter()
                .filter_map(|snapshot| snapshot.series.get(*series))
                .collect();
            let age_days: Vec<i64> = rows.iter().map(|snapshot| snapshot.age_days).collect();
            MacroSeriesFreshness {
                series: (*series).to_string(),
                frequency: (*frequency).to_string(),
                score_dates_covered: rows.len(),
                missing_score_dates: snapshots.len() - rows.len(),
                stale_score_dates: rows.iter().filter(|snapshot| snapshot.stale).count(),
                average_age_days: optional_average_i64(&age_days),
                max_age_days: age_days.iter().copied().max(),
                latest_observation_date: rows
                    .iter()
                    .map(|snapshot| snapshot.observation_date.as_str())
                    .max()
                    .map(str::to_string),
            }
        })
        .collect()
}

fn flag_summaries(snapshots: &[MacroDateSnapshot]) -> Vec<MacroFlagSummary> {
    all_flags()
        .iter()
        .map(|flag| {
            let active: Vec<&MacroDateSnapshot> = snapshots
                .iter()
                .filter(|snapshot| snapshot.active_flags.iter().any(|active| active == flag))
                .collect();
            let active_scores: Vec<f64> = active
                .iter()
                .map(|snapshot| snapshot.regime.score)
                .collect();
            MacroFlagSummary {
                flag: (*flag).to_string(),
                active_dates: active.len(),
                active_share: if snapshots.is_empty() {
                    0.0
                } else {
                    active.len() as f64 / snapshots.len() as f64
                },
                average_regime_score_when_active: optional_average(&active_scores),
                risk_on_dates_when_active: active
                    .iter()
                    .filter(|snapshot| is_risk_on(&snapshot.regime.label))
                    .count(),
                defensive_or_mixed_dates_when_active: active
                    .iter()
                    .filter(|snapshot| is_defensive_or_mixed(&snapshot.regime.label))
                    .count(),
            }
        })
        .collect()
}

fn sector_leadership(
    snapshots: &[MacroDateSnapshot],
    sector_scores: &[SectorScore],
) -> Vec<MacroSectorLeadershipSummary> {
    let active_flags_by_date: HashMap<&str, &Vec<String>> = snapshots
        .iter()
        .filter(|snapshot| !snapshot.active_flags.is_empty())
        .map(|snapshot| (snapshot.date.as_str(), &snapshot.active_flags))
        .collect();
    let mut groups: HashMap<(String, String), Vec<&SectorScore>> = HashMap::new();

    for score in sector_scores {
        let Some(flags) = active_flags_by_date.get(score.date.as_str()) else {
            continue;
        };
        for flag in *flags {
            groups
                .entry((flag.clone(), score.sector.clone()))
                .or_default()
                .push(score);
        }
    }

    let mut rows: Vec<MacroSectorLeadershipSummary> = groups
        .into_iter()
        .map(|((flag, sector), scores)| {
            let ranks: Vec<f64> = scores.iter().map(|score| score.rank as f64).collect();
            let sector_scores: Vec<f64> = scores.iter().map(|score| score.score).collect();
            MacroSectorLeadershipSummary {
                flag,
                sector,
                observations: scores.len(),
                top_rank_count: scores.iter().filter(|score| score.rank == 1).count(),
                average_rank: average(&ranks),
                average_score: average(&sector_scores),
            }
        })
        .collect();

    rows.sort_by(|a, b| {
        a.flag
            .cmp(&b.flag)
            .then(a.average_rank.total_cmp(&b.average_rank))
            .then(b.top_rank_count.cmp(&a.top_rank_count))
            .then(a.sector.cmp(&b.sector))
    });
    rows
}

fn disagreement_examples(snapshots: &[MacroDateSnapshot]) -> Vec<MacroRegimeDisagreement> {
    snapshots
        .iter()
        .filter_map(|snapshot| {
            if is_risk_on(&snapshot.regime.label) && !snapshot.active_flags.is_empty() {
                Some(MacroRegimeDisagreement {
                    date: snapshot.date.clone(),
                    regime_label: snapshot.regime.label.clone(),
                    regime_score: snapshot.regime.score,
                    active_flags: snapshot.active_flags.clone(),
                    reason: "ETF-proxy regime is risk-on while macro stress flags are active."
                        .to_string(),
                })
            } else if is_defensive_or_mixed(&snapshot.regime.label)
                && snapshot.active_flags.is_empty()
                && is_complete(snapshot)
            {
                Some(MacroRegimeDisagreement {
                    date: snapshot.date.clone(),
                    regime_label: snapshot.regime.label.clone(),
                    regime_score: snapshot.regime.score,
                    active_flags: Vec::new(),
                    reason: "ETF-proxy regime is defensive or mixed while no macro stress flags are active."
                        .to_string(),
                })
            } else {
                None
            }
        })
        .take(10)
        .collect()
}

fn stale_series(series: &HashMap<String, MacroSeriesSnapshot>) -> Vec<String> {
    let mut stale: Vec<String> = series
        .iter()
        .filter(|(_, snapshot)| snapshot.stale)
        .map(|(series, _)| series.clone())
        .collect();
    stale.sort();
    stale
}

fn overlay_interpretation(regime_label: &str, active_flags: &[String]) -> String {
    if is_risk_on(regime_label) && !active_flags.is_empty() {
        "ETF-proxy regime is risk-on while macro stress flags are active.".to_string()
    } else if is_defensive_or_mixed(regime_label) && active_flags.is_empty() {
        "ETF-proxy regime is defensive or mixed while no macro stress flags are active.".to_string()
    } else if active_flags.is_empty() {
        "No active macro stress flags for this report date.".to_string()
    } else {
        "Macro stress flags are active; treat them as context, not score inputs.".to_string()
    }
}

fn all_flags() -> [&'static str; 8] {
    [
        macro_validation::FLAG_VOLATILITY_STRESS,
        macro_validation::FLAG_RATE_PRESSURE,
        macro_validation::FLAG_YIELD_CURVE_INVERSION,
        macro_validation::FLAG_CREDIT_STRESS,
        macro_validation::FLAG_DOLLAR_PRESSURE,
        macro_validation::FLAG_LIQUIDITY_TIGHTENING,
        macro_validation::FLAG_INFLATION_PRESSURE,
        macro_validation::FLAG_LABOR_COOLING,
    ]
}

fn is_complete(snapshot: &MacroDateSnapshot) -> bool {
    snapshot.series.len() == macro_data::MACRO_SERIES.len()
}

fn is_risk_on(label: &str) -> bool {
    let normalized = label.to_ascii_lowercase();
    normalized.starts_with("risk-on") || normalized.starts_with("risk_on")
}

fn is_defensive_or_mixed(label: &str) -> bool {
    let normalized = label.to_ascii_lowercase();
    normalized.starts_with("defensive") || normalized.starts_with("mixed")
}

fn max_age_days(frequency: &str) -> i64 {
    match frequency.to_ascii_lowercase().as_str() {
        "monthly" => macro_validation::MONTHLY_MAX_AGE_DAYS,
        "weekly" => macro_validation::WEEKLY_MAX_AGE_DAYS,
        _ => macro_validation::DAILY_MAX_AGE_DAYS,
    }
}

fn parse_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .with_context(|| format!("invalid macro validation date: {date}"))
}

fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn optional_average(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| average(values))
}

fn optional_average_i64(values: &[i64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<i64>() as f64 / values.len() as f64)
}
