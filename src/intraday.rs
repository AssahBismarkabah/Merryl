use std::collections::{BTreeMap, HashMap};

use anyhow::{Result, bail};
use serde_json::json;

use crate::config::intraday;
use crate::domain::models::{
    DailyPrice, IntradayPrice, IntradaySetup, IntradayTrigger, SectorMap, Symbol, VolumeProfile,
};
use crate::scoring::{PriceHistories, effective_index, histories_by_symbol};

#[derive(Debug, Clone)]
pub struct IntradayReadinessInput {
    pub date: String,
    pub symbols: Vec<Symbol>,
    pub sector_maps: Vec<SectorMap>,
    pub daily_prices: Vec<DailyPrice>,
    pub profile_prices: Vec<IntradayPrice>,
    pub trigger_prices: Vec<IntradayPrice>,
    pub profile_timeframe: String,
    pub trigger_timeframe: String,
    pub candidate_limit: usize,
    pub opening_range_minutes: usize,
}

#[derive(Debug, Clone)]
pub struct IntradayReadinessResult {
    pub date: String,
    pub evaluated_count: usize,
    pub stage1_count: usize,
    pub stage2_count: usize,
    pub stage3_trigger_count: usize,
    pub setups: Vec<IntradaySetup>,
    pub profiles: Vec<VolumeProfile>,
    pub triggers: Vec<IntradayTrigger>,
}

#[derive(Debug, Clone)]
struct Stage1Candidate {
    symbol: Symbol,
    sector_etf: String,
    adr_pct: f64,
    rvol_ratio: f64,
    mansfield_rs_spy: f64,
    mansfield_rs_sector: f64,
    ema_10: f64,
    ema_20: f64,
    latest_price: f64,
    profile_bin_size: f64,
}

pub fn run_intraday_readiness(input: IntradayReadinessInput) -> Result<IntradayReadinessResult> {
    let histories = histories_by_symbol(&input.daily_prices);
    let stage1_metrics =
        stage1_metrics(&input.symbols, &input.sector_maps, &histories, &input.date);
    let evaluated_count = stage1_metrics.len();
    let stage1_candidates = stage1_candidates(stage1_metrics, input.candidate_limit.max(1));
    let profile_prices = intraday_by_symbol(&input.profile_prices);
    let trigger_prices = intraday_by_symbol(&input.trigger_prices);
    let mut profiles = Vec::new();
    let mut setups = Vec::new();
    let mut triggers = Vec::new();

    for candidate in stage1_candidates {
        let bars = profile_prices
            .get(candidate.symbol.symbol.as_str())
            .cloned()
            .unwrap_or_default();
        let profile = build_volume_profile(
            &candidate.symbol.symbol,
            &input.date,
            &input.profile_timeframe,
            &bars,
            candidate.profile_bin_size,
        );
        let mut latest_price = candidate.latest_price;
        let mut confluence = Vec::new();
        let mut stage2_passed = false;

        if let Some(profile) = profile.as_ref() {
            latest_price = bars
                .last()
                .map(|bar| bar.close)
                .unwrap_or(candidate.latest_price);
            confluence = confluence_labels(
                latest_price,
                &[
                    ("poc", profile.poc),
                    ("val", profile.val),
                    ("vwap", profile.vwap),
                    ("ema_10", candidate.ema_10),
                    ("ema_20", candidate.ema_20),
                ],
                intraday::CONFLUENCE_WINDOW,
            );
            stage2_passed = candidate.ema_10 > candidate.ema_20
                && confluence.len() >= intraday::CONFLUENCE_MIN
                && pct_distance(latest_price, candidate.ema_20) >= intraday::MATERIAL_EMA20_BREAK;
            profiles.push(profile.clone());
        }

        let mut candidate_triggers = Vec::new();
        if stage2_passed && let Some(profile) = profile.as_ref() {
            let bars = trigger_prices
                .get(candidate.symbol.symbol.as_str())
                .cloned()
                .unwrap_or_default();
            candidate_triggers = detect_intraday_triggers(
                &input.date,
                &candidate.symbol.symbol,
                &input.trigger_timeframe,
                &bars,
                &[
                    profile.poc,
                    profile.val,
                    profile.vwap,
                    candidate.ema_10,
                    candidate.ema_20,
                ],
                intraday::CONFLUENCE_WINDOW,
                input.opening_range_minutes,
            );
            triggers.extend(candidate_triggers.clone());
        }

        let stage3_passed = stage2_passed && !candidate_triggers.is_empty();
        let primary_label = if stage3_passed {
            intraday::LABEL_STAGE3
        } else if stage2_passed {
            intraday::LABEL_STAGE2
        } else {
            intraday::LABEL_STAGE1
        };
        setups.push(IntradaySetup {
            date: input.date.clone(),
            symbol: candidate.symbol.symbol.clone(),
            name: candidate.symbol.name.clone(),
            sector: candidate.symbol.sector.clone().unwrap_or_default(),
            industry: candidate.symbol.industry.clone().unwrap_or_default(),
            direction: intraday::DIRECTION_LONG.to_string(),
            stage1_passed: true,
            stage2_passed,
            stage3_passed,
            primary_label: primary_label.to_string(),
            adr_pct: candidate.adr_pct,
            rvol_ratio: candidate.rvol_ratio,
            mansfield_rs_spy: candidate.mansfield_rs_spy,
            mansfield_rs_sector: candidate.mansfield_rs_sector,
            ema_10: candidate.ema_10,
            ema_20: candidate.ema_20,
            latest_price,
            confluence_count: confluence.len(),
            confluence_json: json!(confluence).to_string(),
            trigger_count: candidate_triggers.len(),
            components_json: json!({
                "sector_etf": candidate.sector_etf,
                "adr_threshold": intraday::ADR_MIN,
                "rvol_threshold": intraday::RVOL_MIN,
                "confluence_window": intraday::CONFLUENCE_WINDOW,
                "confluence_min": intraday::CONFLUENCE_MIN,
                "profile_bin_size": candidate.profile_bin_size,
                "signal_only": true
            })
            .to_string(),
        });
    }

    Ok(IntradayReadinessResult {
        date: input.date,
        evaluated_count,
        stage1_count: setups.len(),
        stage2_count: setups.iter().filter(|setup| setup.stage2_passed).count(),
        stage3_trigger_count: triggers.len(),
        setups,
        profiles,
        triggers,
    })
}

pub fn high_momentum_candidate_symbols(
    symbols: &[Symbol],
    sector_maps: &[SectorMap],
    daily_prices: &[DailyPrice],
    date: &str,
    candidate_limit: usize,
) -> (usize, Vec<String>) {
    let histories = histories_by_symbol(daily_prices);
    let metrics = stage1_metrics(symbols, sector_maps, &histories, date);
    let evaluated_count = metrics.len();
    let symbols = stage1_candidates(metrics, candidate_limit.max(1))
        .into_iter()
        .map(|candidate| candidate.symbol.symbol)
        .collect();
    (evaluated_count, symbols)
}

fn stage1_metrics(
    symbols: &[Symbol],
    sector_maps: &[SectorMap],
    histories: &PriceHistories,
    date: &str,
) -> Vec<Stage1Candidate> {
    let sector_etfs: HashMap<&str, &str> = sector_maps
        .iter()
        .map(|map| (map.sector.as_str(), map.sector_etf.as_str()))
        .collect();

    symbols
        .iter()
        .filter(|symbol| symbol.is_active && symbol.asset_type == "stock")
        .filter_map(|symbol| {
            let sector = symbol.sector.as_deref()?;
            let sector_etf = sector_etfs.get(sector)?;
            let history = histories.get(&symbol.symbol)?;
            let idx = effective_index(history, date)?;
            let adr_pct = adr_pct(history, idx, intraday::ADR_LOOKBACK)?;
            let rvol_ratio = rvol_ratio(history, idx, intraday::RVOL_LOOKBACK)?;
            let ema_10 = ema_close(history, idx, intraday::EMA_FAST)?;
            let ema_20 = ema_close(history, idx, intraday::EMA_SLOW)?;
            let atr = atr_amount(history, idx, intraday::VOLUME_PROFILE_ATR_LOOKBACK)?;
            let profile_bin_size = volume_profile_bin_size(history[idx].adjusted_close, Some(atr));
            let mansfield_rs_spy = mansfield_rs(
                histories,
                &symbol.symbol,
                "SPY",
                date,
                intraday::MANSFIELD_LOOKBACK,
            )?;
            let mansfield_rs_sector = mansfield_rs(
                histories,
                &symbol.symbol,
                sector_etf,
                date,
                intraday::MANSFIELD_LOOKBACK,
            )?;
            Some(Stage1Candidate {
                symbol: symbol.clone(),
                sector_etf: (*sector_etf).to_string(),
                adr_pct,
                rvol_ratio,
                mansfield_rs_spy,
                mansfield_rs_sector,
                ema_10,
                ema_20,
                latest_price: history[idx].adjusted_close,
                profile_bin_size,
            })
        })
        .collect()
}

fn stage1_candidates(
    mut metrics: Vec<Stage1Candidate>,
    candidate_limit: usize,
) -> Vec<Stage1Candidate> {
    metrics.sort_by(|left, right| {
        right
            .mansfield_rs_spy
            .total_cmp(&left.mansfield_rs_spy)
            .then_with(|| right.rvol_ratio.total_cmp(&left.rvol_ratio))
    });
    let top_count = ((metrics.len() as f64) * intraday::RS_TOP_PERCENTILE)
        .ceil()
        .max(1.0) as usize;
    metrics
        .into_iter()
        .enumerate()
        .filter(|(idx, candidate)| {
            *idx < top_count
                && candidate.adr_pct > intraday::ADR_MIN
                && candidate.rvol_ratio > intraday::RVOL_MIN
        })
        .map(|(_, candidate)| candidate)
        .take(candidate_limit)
        .collect()
}

fn intraday_by_symbol(prices: &[IntradayPrice]) -> HashMap<&str, Vec<IntradayPrice>> {
    let mut grouped: HashMap<&str, Vec<IntradayPrice>> = HashMap::new();
    for price in prices {
        grouped
            .entry(price.symbol.as_str())
            .or_default()
            .push(price.clone());
    }
    for bars in grouped.values_mut() {
        bars.sort_by(|left, right| left.ts.cmp(&right.ts));
    }
    grouped
}

pub fn adr_pct(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx + 1 < lookback {
        return None;
    }
    let values = history[idx + 1 - lookback..=idx]
        .iter()
        .filter_map(|price| {
            if price.close == 0.0 {
                None
            } else {
                Some((price.high - price.low) / price.close)
            }
        })
        .collect::<Vec<_>>();
    average(&values)
}

pub fn rvol_ratio(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx < lookback {
        return None;
    }
    let average_volume = history[idx - lookback..idx]
        .iter()
        .map(|price| price.volume)
        .sum::<f64>()
        / lookback as f64;
    if average_volume == 0.0 {
        None
    } else {
        Some(history[idx].volume / average_volume)
    }
}

pub fn ema_close(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx + 1 < lookback {
        return None;
    }
    let mut ema = history[..lookback]
        .iter()
        .map(|price| price.adjusted_close)
        .sum::<f64>()
        / lookback as f64;
    let multiplier = 2.0 / (lookback as f64 + 1.0);

    for price in history.iter().take(idx + 1).skip(lookback) {
        ema = (price.adjusted_close - ema) * multiplier + ema;
    }

    Some(ema)
}

pub fn atr_amount(history: &[DailyPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx + 1 < lookback {
        return None;
    }
    let start = idx + 1 - lookback;
    let values = (start..=idx)
        .map(|current_idx| {
            let price = &history[current_idx];
            let previous_close = if current_idx > 0 {
                history[current_idx - 1].adjusted_close
            } else {
                price.adjusted_close
            };
            (price.high - price.low)
                .max((price.high - previous_close).abs())
                .max((price.low - previous_close).abs())
        })
        .collect::<Vec<_>>();
    average(&values)
}

pub fn mansfield_rs(
    histories: &PriceHistories,
    symbol: &str,
    benchmark: &str,
    date: &str,
    lookback: usize,
) -> Option<f64> {
    let history = histories.get(symbol)?;
    let idx = effective_index(history, date)?;
    if idx + 1 < lookback {
        return None;
    }

    let ratio_values = history[idx + 1 - lookback..=idx]
        .iter()
        .map(|price| {
            let benchmark_history = histories.get(benchmark)?;
            let benchmark_idx = effective_index(benchmark_history, &price.date)?;
            let benchmark_close = benchmark_history[benchmark_idx].adjusted_close;
            if benchmark_close == 0.0 {
                return None;
            }
            Some(price.adjusted_close / benchmark_close)
        })
        .collect::<Option<Vec<_>>>()?;
    let current_ratio = *ratio_values.last()?;
    let average_ratio = average(&ratio_values)?;
    if average_ratio == 0.0 {
        None
    } else {
        Some(current_ratio / average_ratio)
    }
}

pub fn build_volume_profile(
    symbol: &str,
    date: &str,
    timeframe: &str,
    bars: &[IntradayPrice],
    bin_size: f64,
) -> Option<VolumeProfile> {
    if bars.is_empty() {
        return None;
    }

    let mut bins: BTreeMap<i64, f64> = BTreeMap::new();
    let mut total_volume = 0.0;
    let mut weighted_price_volume = 0.0;
    let mut high = f64::NEG_INFINITY;
    let mut low = f64::INFINITY;
    let bin_size = bin_size.max(intraday::VOLUME_PROFILE_MIN_BIN_SIZE);

    for bar in bars {
        let hlc3 = hlc3(bar);
        let price_bin = (hlc3 / bin_size).round() as i64;
        *bins.entry(price_bin).or_default() += bar.volume;
        total_volume += bar.volume;
        weighted_price_volume += hlc3 * bar.volume;
        high = high.max(bar.high);
        low = low.min(bar.low);
    }

    if total_volume == 0.0 {
        return None;
    }

    let ordered_bins = bins.into_iter().collect::<Vec<_>>();
    let poc_idx = ordered_bins
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.1.total_cmp(&right.1))
        .map(|(idx, _)| idx)?;
    let (val_idx, vah_idx, captured_volume) = value_area_bounds(
        &ordered_bins,
        poc_idx,
        total_volume * intraday::VALUE_AREA_SHARE,
    );
    let poc = ordered_bins[poc_idx].0 as f64 * bin_size;
    let val = ordered_bins[val_idx].0 as f64 * bin_size;
    let vah = ordered_bins[vah_idx].0 as f64 * bin_size;
    let vwap = weighted_price_volume / total_volume;

    Some(VolumeProfile {
        symbol: symbol.to_string(),
        date: date.to_string(),
        timeframe: timeframe.to_string(),
        poc,
        vah,
        val,
        vwap,
        high,
        low,
        total_volume,
        source: bars
            .first()
            .map(|bar| bar.source.clone())
            .unwrap_or_default(),
        components_json: json!({
            "bar_count": bars.len(),
            "captured_value_area_volume": captured_volume,
            "value_area_share": intraday::VALUE_AREA_SHARE,
            "bin_size": bin_size,
            "bin_size_policy": "max(price_pct, atr_pct)_clamped",
            "price_bin": "rounded_hlc3_dynamic",
            "value_area_tie_break": intraday::VALUE_AREA_TIE_BREAK_POLICY
        })
        .to_string(),
    })
}

pub fn volume_profile_bin_size(close_price: f64, atr: Option<f64>) -> f64 {
    let close = close_price.abs();
    let price_bin = close * intraday::VOLUME_PROFILE_BIN_WIDTH_PCT;
    let atr_bin = atr
        .filter(|value| value.is_finite() && *value > 0.0)
        .map(|value| value * intraday::VOLUME_PROFILE_ATR_BIN_WIDTH_PCT)
        .unwrap_or(price_bin);
    let max_bin = intraday::VOLUME_PROFILE_MIN_BIN_SIZE
        .max(close * intraday::VOLUME_PROFILE_MAX_BIN_SIZE_PCT);
    price_bin
        .max(atr_bin)
        .max(intraday::VOLUME_PROFILE_MIN_BIN_SIZE)
        .min(max_bin)
}

fn value_area_bounds(
    bins: &[(i64, f64)],
    poc_idx: usize,
    target_volume: f64,
) -> (usize, usize, f64) {
    let mut low_idx = poc_idx;
    let mut high_idx = poc_idx;
    let mut captured_volume = bins[poc_idx].1;

    while captured_volume < target_volume && (low_idx > 0 || high_idx + 1 < bins.len()) {
        let lower_volume = if low_idx > 0 {
            bins[low_idx - 1].1
        } else {
            f64::NEG_INFINITY
        };
        let upper_volume = if high_idx + 1 < bins.len() {
            bins[high_idx + 1].1
        } else {
            f64::NEG_INFINITY
        };

        if should_expand_upper_value_area_bin(upper_volume, lower_volume)
            && high_idx + 1 < bins.len()
        {
            high_idx += 1;
            captured_volume += bins[high_idx].1;
        } else if low_idx > 0 {
            low_idx -= 1;
            captured_volume += bins[low_idx].1;
        } else {
            break;
        }
    }

    (low_idx, high_idx, captured_volume)
}

fn should_expand_upper_value_area_bin(upper_volume: f64, lower_volume: f64) -> bool {
    // Equal-volume expansion intentionally moves upward first.
    matches!(
        upper_volume.total_cmp(&lower_volume),
        std::cmp::Ordering::Greater | std::cmp::Ordering::Equal
    )
}

pub fn session_vwap(bars: &[IntradayPrice]) -> Option<f64> {
    let total_volume = bars.iter().map(|bar| bar.volume).sum::<f64>();
    if total_volume == 0.0 {
        return None;
    }
    Some(bars.iter().map(|bar| hlc3(bar) * bar.volume).sum::<f64>() / total_volume)
}

pub fn confluence_labels(latest_price: f64, levels: &[(&str, f64)], window: f64) -> Vec<String> {
    levels
        .iter()
        .filter(|(_, level)| pct_distance(*level, latest_price).abs() <= window)
        .map(|(label, _)| (*label).to_string())
        .collect()
}

pub fn detect_intraday_triggers(
    date: &str,
    symbol: &str,
    timeframe: &str,
    bars: &[IntradayPrice],
    confluence_levels: &[f64],
    confluence_window: f64,
    opening_range_minutes: usize,
) -> Vec<IntradayTrigger> {
    let mut triggers = Vec::new();
    if let Some(trigger) = detect_orb_breakout(date, symbol, timeframe, bars, opening_range_minutes)
    {
        triggers.push(trigger);
    }
    if let Some(trigger) = detect_hod_break(date, symbol, timeframe, bars) {
        triggers.push(trigger);
    }
    if let Some(trigger) = detect_volume_dryup_confirmation(
        date,
        symbol,
        timeframe,
        bars,
        confluence_levels,
        confluence_window,
    ) {
        triggers.push(trigger);
    }
    if let Some(trigger) = detect_micro_cluster_break(
        date,
        symbol,
        timeframe,
        bars,
        confluence_levels,
        confluence_window,
    ) {
        triggers.push(trigger);
    }
    triggers
}

pub fn validate_intraday_timeframe(timeframe: &str) -> Result<()> {
    if timeframe.ends_with("Min") || timeframe.ends_with("Hour") {
        return Ok(());
    }
    bail!("intraday timeframe `{timeframe}` must use Alpaca minute/hour syntax, for example 5Min")
}

fn detect_orb_breakout(
    date: &str,
    symbol: &str,
    timeframe: &str,
    bars: &[IntradayPrice],
    opening_range_minutes: usize,
) -> Option<IntradayTrigger> {
    if bars.is_empty() {
        return None;
    }
    let timeframe_minutes = timeframe_minutes(timeframe)?;
    let opening_range_bars =
        opening_range_bars(opening_range_minutes, timeframe_minutes).min(bars.len() - 1);
    if bars.len() <= opening_range_bars || opening_range_bars == 0 {
        return None;
    }
    let opening_high = bars[..opening_range_bars]
        .iter()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);

    for idx in opening_range_bars..bars.len() {
        let bar = &bars[idx];
        let avg_volume = recent_average_volume(bars, idx, opening_range_bars)?;
        let volume_spike = volume_spike(bar.volume, avg_volume)?;
        if bar.high > opening_high
            && bar.close > opening_high
            && volume_spike > intraday::VOLUME_SPIKE_MIN
        {
            return Some(trigger(
                date,
                symbol,
                timeframe,
                bar,
                TriggerEvent {
                    trigger_type: intraday::TRIGGER_ORB_BREAKOUT,
                    reference_level: opening_high,
                    volume_spike,
                    price_action: "broke above the 30-minute opening range high",
                },
            ));
        }
    }
    None
}

fn detect_hod_break(
    date: &str,
    symbol: &str,
    timeframe: &str,
    bars: &[IntradayPrice],
) -> Option<IntradayTrigger> {
    if bars.len() < 4 {
        return None;
    }
    let mut previous_hod = bars[0].high;
    for idx in 1..bars.len() {
        let bar = &bars[idx];
        let Some(avg_volume) = recent_average_volume(bars, idx, 3) else {
            previous_hod = previous_hod.max(bar.high);
            continue;
        };
        let spike = volume_spike(bar.volume, avg_volume)?;
        if bar.high > previous_hod && bar.close > previous_hod && spike > intraday::VOLUME_SPIKE_MIN
        {
            return Some(trigger(
                date,
                symbol,
                timeframe,
                bar,
                TriggerEvent {
                    trigger_type: intraday::TRIGGER_HOD_BREAK,
                    reference_level: previous_hod,
                    volume_spike: spike,
                    price_action: "broke the prior intraday high with volume expansion",
                },
            ));
        }
        previous_hod = previous_hod.max(bar.high);
    }
    None
}

fn detect_volume_dryup_confirmation(
    date: &str,
    symbol: &str,
    timeframe: &str,
    bars: &[IntradayPrice],
    levels: &[f64],
    window: f64,
) -> Option<IntradayTrigger> {
    if bars.len() < 5 {
        return None;
    }

    for idx in 3..bars.len() - 1 {
        let pullback = &bars[idx];
        let avg_volume = recent_average_volume(bars, idx, 3)?;
        if !near_any_level(pullback.close, levels, window)
            || pullback.volume > avg_volume * intraday::DRYUP_RATIO_MAX
        {
            continue;
        }

        let confirmation = &bars[idx + 1];
        let confirmation_avg = recent_average_volume(bars, idx + 1, 3)?;
        let spike = volume_spike(confirmation.volume, confirmation_avg)?;
        if confirmation.close > pullback.high && spike > intraday::VOLUME_SPIKE_MIN {
            return Some(trigger(
                date,
                symbol,
                timeframe,
                confirmation,
                TriggerEvent {
                    trigger_type: intraday::TRIGGER_VOLUME_DRYUP_CONFIRMATION,
                    reference_level: pullback.close,
                    volume_spike: spike,
                    price_action: "volume dried up near confluence and expanded on confirmation",
                },
            ));
        }
    }

    None
}

fn detect_micro_cluster_break(
    date: &str,
    symbol: &str,
    timeframe: &str,
    bars: &[IntradayPrice],
    levels: &[f64],
    window: f64,
) -> Option<IntradayTrigger> {
    let cluster_len = intraday::MICRO_CLUSTER_BAR_COUNT;
    if bars.len() <= cluster_len {
        return None;
    }

    for start in 0..bars.len() - cluster_len {
        let end = start + cluster_len;
        let cluster = &bars[start..end];
        let cluster_high = cluster
            .iter()
            .map(|bar| bar.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let cluster_low = cluster
            .iter()
            .map(|bar| bar.low)
            .fold(f64::INFINITY, f64::min);
        let cluster_mid = (cluster_high + cluster_low) / 2.0;
        let range_pct = if cluster_mid == 0.0 {
            continue;
        } else {
            (cluster_high - cluster_low) / cluster_mid
        };
        if range_pct > intraday::MICRO_CLUSTER_MAX_RANGE
            || !near_any_level(cluster_mid, levels, window)
        {
            continue;
        }

        let breakout = &bars[end];
        let avg_volume = recent_average_volume(bars, end, cluster_len)?;
        let spike = volume_spike(breakout.volume, avg_volume)?;
        if breakout.close > cluster_high && spike > intraday::VOLUME_SPIKE_MIN {
            return Some(trigger(
                date,
                symbol,
                timeframe,
                breakout,
                TriggerEvent {
                    trigger_type: intraday::TRIGGER_MICRO_CLUSTER_BREAK,
                    reference_level: cluster_high,
                    volume_spike: spike,
                    price_action: "tight volume cluster broke upward with volume expansion",
                },
            ));
        }
    }

    None
}

struct TriggerEvent<'a> {
    trigger_type: &'a str,
    reference_level: f64,
    volume_spike: f64,
    price_action: &'a str,
}

fn trigger(
    date: &str,
    symbol: &str,
    timeframe: &str,
    bar: &IntradayPrice,
    event: TriggerEvent<'_>,
) -> IntradayTrigger {
    IntradayTrigger {
        date: date.to_string(),
        symbol: symbol.to_string(),
        ts: bar.ts.clone(),
        timeframe: timeframe.to_string(),
        trigger_type: event.trigger_type.to_string(),
        direction: intraday::DIRECTION_LONG.to_string(),
        trigger_price: bar.close,
        reference_level: event.reference_level,
        volume_spike: event.volume_spike,
        price_action: event.price_action.to_string(),
        components_json: json!({
            "bar_high": bar.high,
            "bar_low": bar.low,
            "bar_volume": bar.volume
        })
        .to_string(),
        source: bar.source.clone(),
    }
}

fn recent_average_volume(bars: &[IntradayPrice], idx: usize, lookback: usize) -> Option<f64> {
    if idx < lookback {
        return None;
    }
    average(
        &bars[idx - lookback..idx]
            .iter()
            .map(|bar| bar.volume)
            .collect::<Vec<_>>(),
    )
}

pub fn opening_range_bars(opening_range_minutes: usize, timeframe_minutes: usize) -> usize {
    if timeframe_minutes == 0 {
        return 0;
    }
    opening_range_minutes.div_ceil(timeframe_minutes).max(1)
}

pub fn timeframe_minutes(timeframe: &str) -> Option<usize> {
    if let Some(value) = timeframe.strip_suffix("Min") {
        return value.parse::<usize>().ok();
    }
    if let Some(value) = timeframe.strip_suffix("Hour") {
        return value.parse::<usize>().ok().map(|hours| hours * 60);
    }
    None
}

fn volume_spike(volume: f64, average_volume: f64) -> Option<f64> {
    if average_volume == 0.0 {
        None
    } else {
        Some(volume / average_volume)
    }
}

fn near_any_level(price: f64, levels: &[f64], window: f64) -> bool {
    levels
        .iter()
        .any(|level| pct_distance(price, *level).abs() <= window)
}

fn hlc3(bar: &IntradayPrice) -> f64 {
    (bar.high + bar.low + bar.close) / 3.0
}

fn pct_distance(value: f64, reference: f64) -> f64 {
    if reference == 0.0 {
        0.0
    } else {
        (value / reference) - 1.0
    }
}

fn average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}
