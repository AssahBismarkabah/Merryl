//! Daily total-volume structural candidates for the dashboard review tab.
//!
//! This is deliberately a candidate classifier, not an execution strategy. It
//! uses daily OHLCV and documents the approximation that daily-bar volume is
//! spread across each bar's high-low range.

use std::collections::HashMap;

use serde::Serialize;

use crate::domain::models::{DailyPrice, Symbol};

const MIN_BARS: usize = 60;
const PROFILE_BARS: usize = 30;
const ATR_BARS: usize = 20;
const PROFILE_ROWS: usize = 32;

#[derive(Debug, Clone, Serialize)]
pub struct VolumeProfileClassificationResponse {
    pub date: String,
    pub timeframe: String,
    pub volume_source: String,
    pub approximation_note: String,
    pub evaluated_count: usize,
    pub candidate_count: usize,
    pub skipped_count: usize,
    pub results: Vec<VolumeProfileClassification>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VolumeProfileClassification {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub labels: Vec<String>,
    pub direction: String,
    pub status: String,
    pub anchor_start: String,
    pub anchor_end: String,
    pub poc: Option<f64>,
    pub node_low: Option<f64>,
    pub node_high: Option<f64>,
    pub latest_price: f64,
    pub distance_to_node_pct: Option<f64>,
    pub review_note: String,
}

#[derive(Debug, Clone, Copy)]
struct Profile {
    poc: f64,
    node_low: f64,
    node_high: f64,
}

pub fn classify_active_stocks(
    symbols: &[Symbol],
    daily_prices: &[DailyPrice],
) -> VolumeProfileClassificationResponse {
    let mut histories: HashMap<&str, Vec<&DailyPrice>> = HashMap::new();
    for price in daily_prices {
        histories
            .entry(price.symbol.as_str())
            .or_default()
            .push(price);
    }

    let date = daily_prices
        .iter()
        .map(|price| price.date.as_str())
        .max()
        .unwrap_or("")
        .to_string();
    let mut results = Vec::new();
    let mut evaluated_count = 0;
    let mut skipped_count = 0;

    for symbol in symbols
        .iter()
        .filter(|symbol| symbol.is_active && symbol.asset_type == "stock")
    {
        let Some(history) = histories.get(symbol.symbol.as_str()) else {
            skipped_count += 1;
            continue;
        };
        if history.len() < MIN_BARS {
            skipped_count += 1;
            continue;
        }
        evaluated_count += 1;
        results.push(classify_symbol(symbol, history));
    }

    results.sort_by(|left, right| {
        right
            .labels
            .len()
            .cmp(&left.labels.len())
            .then_with(|| left.symbol.cmp(&right.symbol))
    });
    let candidate_count = results
        .iter()
        .filter(|result| !result.labels.is_empty())
        .count();

    VolumeProfileClassificationResponse {
        date,
        timeframe: "1D".to_string(),
        volume_source: "total daily exchange volume".to_string(),
        approximation_note: "Daily-bar approximation: each bar's volume is distributed across its high-low range. Review the candidate on a lower-timeframe chart before acting.".to_string(),
        evaluated_count,
        candidate_count,
        skipped_count,
        results,
    }
}

fn classify_symbol(symbol: &Symbol, history: &[&DailyPrice]) -> VolumeProfileClassification {
    let start = history.len() - PROFILE_BARS;
    let window = &history[start..];
    let latest = history[history.len() - 1];
    let atr = average_true_range(history, history.len() - 1, ATR_BARS);
    let profile = build_profile(window);
    let mut labels = Vec::new();
    let mut direction = "neutral".to_string();
    let mut review_note =
        "No current candidate; continue monitoring the daily structure.".to_string();

    if let Some(atr) = atr {
        if is_box_candidate(window, atr) {
            labels.push("box_candidate".to_string());
            review_note = "Review whether the selected 30-session window is a genuine box with repeated boundaries; the anchor is provisional.".to_string();
        }

        let net_move = latest.adjusted_close - window[0].adjusted_close;
        if net_move.abs() >= 2.5 * atr && trend_efficiency(window) >= 0.35 {
            labels.push("trend_candidate".to_string());
            direction = if net_move > 0.0 { "long" } else { "short" }.to_string();
            review_note = "Review the true swing-leg start and end; this daily candidate is not a confirmed visual trend anchor.".to_string();
        }

        if is_rejection_candidate(window, atr) {
            labels.push("rejection_candidate".to_string());
            direction = rejection_direction(window).to_string();
            review_note = "Review the approach, rejection displacement, and follow-through on the chart; wick size alone is not sufficient.".to_string();
        }

        if let Some(profile) = profile {
            if let Some(failure_direction) = crossed_and_retested(profile, history, atr) {
                labels.push("level_failure_candidate".to_string());
                direction = failure_direction.to_string();
                review_note = "Review whether price clearly accepted through the level and returned from the opposite side; one intraday touch is not enough.".to_string();
            }
        }
    }

    let distance_to_node_pct =
        profile.map(|profile| ((latest.adjusted_close - profile.poc).abs() / profile.poc) * 100.0);

    VolumeProfileClassification {
        symbol: symbol.symbol.clone(),
        name: symbol.name.clone(),
        sector: symbol.sector.clone().unwrap_or_default(),
        industry: symbol.industry.clone().unwrap_or_default(),
        labels,
        direction,
        status: "candidate_review".to_string(),
        anchor_start: window[0].date.clone(),
        anchor_end: window[window.len() - 1].date.clone(),
        poc: profile.map(|profile| profile.poc),
        node_low: profile.map(|profile| profile.node_low),
        node_high: profile.map(|profile| profile.node_high),
        latest_price: latest.adjusted_close,
        distance_to_node_pct,
        review_note,
    }
}

fn build_profile(bars: &[&DailyPrice]) -> Option<Profile> {
    let low = bars.iter().map(|bar| bar.low).fold(f64::INFINITY, f64::min);
    let high = bars
        .iter()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);
    if !low.is_finite() || !high.is_finite() || high <= low {
        return None;
    }

    let width = (high - low) / PROFILE_ROWS as f64;
    let mut volumes = vec![0.0; PROFILE_ROWS];
    for bar in bars {
        let first = (((bar.low - low) / width).floor() as isize).clamp(0, PROFILE_ROWS as isize - 1)
            as usize;
        let last = (((bar.high - low) / width).floor() as isize).clamp(0, PROFILE_ROWS as isize - 1)
            as usize;
        let count = (last - first + 1) as f64;
        for volume in &mut volumes[first..=last] {
            *volume += bar.volume / count;
        }
    }

    let poc_index = volumes
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .map(|(index, _)| index)?;
    let poc = low + (poc_index as f64 + 0.5) * width;
    Some(Profile {
        poc,
        node_low: low + poc_index as f64 * width,
        node_high: low + (poc_index as f64 + 1.0) * width,
    })
}

fn average_true_range(history: &[&DailyPrice], index: usize, length: usize) -> Option<f64> {
    if index + 1 < length || index >= history.len() {
        return None;
    }
    let start = index + 1 - length;
    let mut ranges = Vec::with_capacity(length);
    for position in start..=index {
        let previous_close = if position == 0 {
            history[position].open
        } else {
            history[position - 1].adjusted_close
        };
        ranges.push(
            (history[position].high - history[position].low)
                .max((history[position].high - previous_close).abs())
                .max((history[position].low - previous_close).abs()),
        );
    }
    Some(ranges.iter().sum::<f64>() / ranges.len() as f64)
}

fn is_box_candidate(window: &[&DailyPrice], atr: f64) -> bool {
    let high = window
        .iter()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = window
        .iter()
        .map(|bar| bar.low)
        .fold(f64::INFINITY, f64::min);
    let width = high - low;
    if width <= 0.0 || width > 6.0 * atr {
        return false;
    }
    let tolerance = width * 0.15;
    let upper_touches = window
        .iter()
        .filter(|bar| bar.high >= high - tolerance)
        .count();
    let lower_touches = window
        .iter()
        .filter(|bar| bar.low <= low + tolerance)
        .count();
    upper_touches >= 2 && lower_touches >= 2
}

fn trend_efficiency(window: &[&DailyPrice]) -> f64 {
    let net = (window.last().unwrap().adjusted_close - window[0].adjusted_close).abs();
    let total = window
        .windows(2)
        .map(|bars| (bars[1].adjusted_close - bars[0].adjusted_close).abs())
        .sum::<f64>();
    if total <= 0.0 { 0.0 } else { net / total }
}

fn is_rejection_candidate(window: &[&DailyPrice], atr: f64) -> bool {
    let Some((index, bar)) = window.iter().enumerate().max_by(|left, right| {
        let left_range = left.1.high - left.1.low;
        let right_range = right.1.high - right.1.low;
        left_range.total_cmp(&right_range)
    }) else {
        return false;
    };
    let range = bar.high - bar.low;
    let close_location = (bar.close - bar.low) / range.max(f64::EPSILON);
    range >= 1.8 * atr
        && ((close_location <= 0.25 && index + 2 < window.len())
            || (close_location >= 0.75 && index + 2 < window.len()))
}

fn rejection_direction(window: &[&DailyPrice]) -> &'static str {
    let bar = window
        .iter()
        .max_by(|left, right| (left.high - left.low).total_cmp(&(right.high - right.low)))
        .unwrap();
    let close_location = (bar.close - bar.low) / (bar.high - bar.low).max(f64::EPSILON);
    if close_location <= 0.25 {
        "short"
    } else {
        "long"
    }
}

fn crossed_and_retested(
    profile: Profile,
    history: &[&DailyPrice],
    atr: f64,
) -> Option<&'static str> {
    let departure = 0.8 * atr;
    let split = history.len().saturating_sub(10);
    let latest = history.last()?.adjusted_close;
    let was_above = history[..history.len() - 1]
        .iter()
        .any(|bar| bar.adjusted_close > profile.node_high + departure);
    let was_below = history[..history.len() - 1]
        .iter()
        .any(|bar| bar.adjusted_close < profile.node_low - departure);
    let recent_prior = history.get(split..history.len().saturating_sub(1))?;
    if was_above
        && recent_prior
            .iter()
            .any(|bar| bar.adjusted_close < profile.node_low)
        && latest >= profile.node_low
        && latest <= profile.node_high
    {
        return Some("short");
    }
    if was_below
        && recent_prior
            .iter()
            .any(|bar| bar.adjusted_close > profile.node_high)
        && latest >= profile.node_low
        && latest <= profile.node_high
    {
        return Some("long");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol() -> Symbol {
        Symbol {
            symbol: "TEST".to_string(),
            name: "Test Company".to_string(),
            asset_type: "stock".to_string(),
            sector: Some("Technology".to_string()),
            industry: Some("Software".to_string()),
            exchange: "NASDAQ".to_string(),
            market_cap: Some(1_000_000_000.0),
            is_active: true,
        }
    }

    fn bars() -> Vec<DailyPrice> {
        (0..80)
            .map(|index| {
                let close = 100.0 + (index % 10) as f64 * 0.2;
                DailyPrice {
                    symbol: "TEST".to_string(),
                    date: format!("2026-01-{:02}", (index % 28) + 1),
                    open: close - 0.2,
                    high: close + 0.4,
                    low: close - 0.4,
                    close,
                    adjusted_close: close,
                    volume: 1_000_000.0 + (index % 5) as f64 * 100_000.0,
                    source: "test".to_string(),
                }
            })
            .collect()
    }

    #[test]
    fn classifier_evaluates_active_stock_and_reports_profile_contract() {
        let prices = bars();
        let response = classify_active_stocks(&[symbol()], &prices);

        assert_eq!(response.timeframe, "1D");
        assert_eq!(response.volume_source, "total daily exchange volume");
        assert_eq!(response.evaluated_count, 1);
        assert_eq!(response.skipped_count, 0);
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].status, "candidate_review");
        assert!(response.results[0].poc.is_some());
        assert!(response.results[0].node_low.is_some());
        assert!(response.results[0].node_high.is_some());
    }

    #[test]
    fn classifier_skips_short_history_without_failing() {
        let prices = bars().into_iter().take(MIN_BARS - 1).collect::<Vec<_>>();
        let response = classify_active_stocks(&[symbol()], &prices);

        assert_eq!(response.evaluated_count, 0);
        assert_eq!(response.skipped_count, 1);
        assert!(response.results.is_empty());
    }
}
