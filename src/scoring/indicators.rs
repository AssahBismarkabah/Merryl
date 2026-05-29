use std::collections::HashMap;

use crate::config::scoring as scoring_config;
use crate::domain::models::DailyPrice;

pub type PriceHistories = HashMap<String, Vec<DailyPrice>>;

pub fn latest_date(prices: &[DailyPrice]) -> Option<String> {
    prices.iter().map(|price| price.date.clone()).max()
}

pub fn histories_by_symbol(prices: &[DailyPrice]) -> PriceHistories {
    let mut histories: PriceHistories = HashMap::new();
    for price in prices {
        histories
            .entry(price.symbol.clone())
            .or_default()
            .push(price.clone());
    }
    for history in histories.values_mut() {
        history.sort_by(|a, b| a.date.cmp(&b.date));
    }
    histories
}

pub fn pct_return(
    histories: &PriceHistories,
    symbol: &str,
    date: &str,
    lookback: usize,
) -> Option<f64> {
    let history = histories.get(symbol)?;
    let idx = effective_index(history, date)?;
    if idx < lookback {
        return None;
    }
    let current = history[idx].adjusted_close;
    let previous = history[idx - lookback].adjusted_close;
    Some((current / previous) - 1.0)
}

pub fn forward_return(
    histories: &PriceHistories,
    symbol: &str,
    date: &str,
    horizon: usize,
) -> Option<f64> {
    let history = histories.get(symbol)?;
    let idx = history.iter().position(|price| price.date == date)?;
    let future_idx = idx + horizon;
    if future_idx >= history.len() {
        return None;
    }
    let current = history[idx].adjusted_close;
    let future = history[future_idx].adjusted_close;
    Some((future / current) - 1.0)
}

pub fn relative_volume(
    histories: &PriceHistories,
    symbol: &str,
    date: &str,
    lookback: usize,
) -> Option<f64> {
    let history = histories.get(symbol)?;
    let idx = effective_index(history, date)?;
    if idx < lookback {
        return None;
    }
    let current = history[idx].volume;
    let avg = history[idx - lookback..idx]
        .iter()
        .map(|price| price.volume)
        .sum::<f64>()
        / lookback as f64;
    Some(current / avg)
}

pub fn avg_dollar_volume(
    histories: &PriceHistories,
    symbol: &str,
    date: &str,
    lookback: usize,
) -> Option<f64> {
    let history = histories.get(symbol)?;
    let idx = effective_index(history, date)?;
    if idx < lookback {
        return None;
    }
    Some(
        history[idx - lookback + 1..=idx]
            .iter()
            .map(|price| price.adjusted_close * price.volume)
            .sum::<f64>()
            / lookback as f64,
    )
}

pub fn trend_state(histories: &PriceHistories, symbol: &str, date: &str) -> String {
    let Some(history) = histories.get(symbol) else {
        return "unknown".to_string();
    };
    let Some(idx) = effective_index(history, date) else {
        return "unknown".to_string();
    };
    let close = history[idx].adjusted_close;
    let above_20 =
        moving_average(history, idx, scoring_config::RETURN_20D).is_some_and(|ma| close > ma);
    let above_50 =
        moving_average(history, idx, scoring_config::RETURN_50D).is_some_and(|ma| close > ma);

    match (above_20, above_50) {
        (true, true) => "above_20d_50d".to_string(),
        (true, false) => "above_20d".to_string(),
        _ => "below_trend".to_string(),
    }
}

pub fn moving_average(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx + 1 < lookback {
        return None;
    }
    Some(
        history[idx + 1 - lookback..=idx]
            .iter()
            .map(|price| price.adjusted_close)
            .sum::<f64>()
            / lookback as f64,
    )
}

pub fn highest_close(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx + 1 < lookback {
        return None;
    }
    history[idx + 1 - lookback..=idx]
        .iter()
        .map(|price| price.adjusted_close)
        .max_by(|left, right| left.total_cmp(right))
}

pub fn lowest_close(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx + 1 < lookback {
        return None;
    }
    history[idx + 1 - lookback..=idx]
        .iter()
        .map(|price| price.adjusted_close)
        .min_by(|left, right| left.total_cmp(right))
}

pub fn true_range(history: &[DailyPrice], idx: usize) -> Option<f64> {
    let price = history.get(idx)?;
    if idx == 0 {
        return Some(price.high - price.low);
    }
    let previous_close = history.get(idx - 1)?.close;
    Some(
        (price.high - price.low)
            .max((price.high - previous_close).abs())
            .max((price.low - previous_close).abs()),
    )
}

pub fn average_true_range(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx + 1 < lookback {
        return None;
    }
    let ranges = (idx + 1 - lookback..=idx)
        .map(|range_idx| true_range(history, range_idx))
        .collect::<Option<Vec<_>>>()?;
    average(&ranges)
}

pub fn distance_pct(value: f64, reference: f64) -> Option<f64> {
    if reference == 0.0 {
        None
    } else {
        Some((value / reference) - 1.0)
    }
}

pub fn range_pct(high: f64, low: f64, close: f64) -> Option<f64> {
    if close == 0.0 {
        None
    } else {
        Some((high - low) / close)
    }
}

pub fn gap_pct(history: &[DailyPrice], idx: usize) -> Option<f64> {
    if idx == 0 {
        return None;
    }
    let current_open = history.get(idx)?.open;
    let previous_close = history.get(idx - 1)?.close;
    distance_pct(current_open, previous_close)
}

pub fn average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

pub fn clamp_score(value: f64) -> f64 {
    value.clamp(scoring_config::SCORE_MIN, scoring_config::SCORE_MAX)
}

pub fn effective_index(history: &[DailyPrice], date: &str) -> Option<usize> {
    history
        .iter()
        .enumerate()
        .filter(|(_, price)| price.date.as_str() <= date)
        .map(|(idx, _)| idx)
        .next_back()
}
