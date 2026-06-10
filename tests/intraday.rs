use std::collections::HashSet;

use anyhow::Result;
use chrono::{Duration, NaiveDate};

use merryl::config::intraday as intraday_config;
use merryl::domain::models::{DailyPrice, IntradayPrice, SectorMap, Symbol};
use merryl::intraday::{
    IntradayReadinessInput, adr_pct, build_volume_profile, confluence_labels,
    detect_intraday_triggers, ema_close, high_momentum_candidate_symbols, mansfield_rs,
    opening_range_bars, run_intraday_readiness, rvol_ratio, session_vwap, volume_profile_bin_size,
};
use merryl::scoring::histories_by_symbol;

#[test]
fn daily_intraday_metrics_calculate_stage_one_inputs() {
    let prices = daily_series("LEAD", 100.0, 0.01, 60, 1_000.0, 3_000.0);
    let idx = prices.len() - 1;

    assert!((adr_pct(&prices, idx, 20).expect("adr") - 0.06).abs() < 0.000000001);
    assert!((rvol_ratio(&prices, idx, 20).expect("rvol") - 3.0).abs() < 0.000000001);
    assert!(
        ema_close(&prices, idx, 10).expect("ema10") > ema_close(&prices, idx, 20).expect("ema20")
    );

    let mut all_prices = prices.clone();
    all_prices.extend(daily_series("SPY", 100.0, 0.001, 60, 1_000.0, 1_000.0));
    let histories = histories_by_symbol(&all_prices);
    let rs = mansfield_rs(&histories, "LEAD", "SPY", "2026-03-01", 50).expect("mansfield rs");

    assert!(rs > 1.0);
}

#[test]
fn volume_profile_calculates_poc_value_area_and_vwap() {
    let bars = vec![
        intraday_bar(
            "LEAD",
            "2026-03-01T14:30:00Z",
            100.0,
            100.0,
            100.0,
            100.0,
            100.0,
        ),
        intraday_bar(
            "LEAD",
            "2026-03-01T15:00:00Z",
            101.0,
            101.0,
            101.0,
            101.0,
            500.0,
        ),
        intraday_bar(
            "LEAD",
            "2026-03-01T15:30:00Z",
            102.0,
            102.0,
            102.0,
            102.0,
            100.0,
        ),
        intraday_bar(
            "LEAD",
            "2026-03-01T16:00:00Z",
            103.0,
            103.0,
            103.0,
            103.0,
            100.0,
        ),
    ];

    let profile = build_volume_profile("LEAD", "2026-03-01", "30Min", &bars).expect("profile");
    let bin_size = volume_profile_bin_size(103.0);

    assert!((profile.poc - 101.0).abs() <= bin_size);
    assert!((profile.val - 101.0).abs() <= bin_size);
    assert!((profile.vah - 102.0).abs() <= bin_size);
    assert!((session_vwap(&bars).expect("vwap") - profile.vwap).abs() < 0.000000001);
}

#[test]
fn volume_profile_uses_dynamic_bins_for_high_priced_tickers() {
    let bars = vec![
        intraday_bar(
            "MSTR",
            "2026-03-01T14:30:00Z",
            1600.00,
            1600.40,
            1599.90,
            1600.10,
            100.0,
        ),
        intraday_bar(
            "MSTR",
            "2026-03-01T15:00:00Z",
            1600.35,
            1600.70,
            1600.05,
            1600.45,
            150.0,
        ),
        intraday_bar(
            "MSTR",
            "2026-03-01T15:30:00Z",
            1600.55,
            1600.95,
            1600.25,
            1600.65,
            200.0,
        ),
    ];

    let profile = build_volume_profile("MSTR", "2026-03-01", "30Min", &bars).expect("profile");
    let bin_size = volume_profile_bin_size(1600.65);

    assert!(bin_size > 0.01);
    assert!((profile.poc - 1600.0).abs() <= bin_size);
    assert_eq!(profile.total_volume, 450.0);
}

#[test]
fn confluence_counts_levels_inside_the_window() {
    let labels = confluence_labels(
        100.0,
        &[("poc", 100.5), ("val", 99.4), ("vwap", 102.0)],
        intraday_config::CONFLUENCE_WINDOW,
    );

    assert_eq!(labels, vec!["poc".to_string(), "val".to_string()]);
}

#[test]
fn trigger_detection_finds_breakout_dryup_and_cluster_signals() {
    let breakout_bars = vec![
        trigger_bar(0, 10.0, 9.95, 9.98, 100.0),
        trigger_bar(1, 10.0, 9.95, 9.99, 100.0),
        trigger_bar(2, 10.0, 9.95, 9.98, 100.0),
        trigger_bar(3, 10.0, 9.95, 9.99, 100.0),
        trigger_bar(4, 10.0, 9.95, 9.98, 100.0),
        trigger_bar(5, 10.0, 9.95, 9.99, 100.0),
        trigger_bar(6, 10.8, 10.0, 10.7, 250.0),
    ];
    let breakout_triggers = detect_intraday_triggers(
        "2026-03-01",
        "LEAD",
        "5Min",
        &breakout_bars,
        &[10.0],
        intraday_config::CONFLUENCE_WINDOW,
        intraday_config::DEFAULT_OPENING_RANGE_MINUTES,
    );
    let breakout_types = trigger_types(&breakout_triggers);

    assert!(breakout_types.contains(intraday_config::TRIGGER_ORB_BREAKOUT));
    assert!(breakout_types.contains(intraday_config::TRIGGER_HOD_BREAK));

    let confluence_bars = vec![
        trigger_bar(0, 10.02, 9.98, 10.0, 100.0),
        trigger_bar(1, 10.02, 9.98, 10.0, 100.0),
        trigger_bar(2, 10.02, 9.98, 10.0, 100.0),
        trigger_bar(3, 10.01, 9.99, 10.0, 50.0),
        trigger_bar(4, 10.18, 10.02, 10.15, 220.0),
    ];
    let confluence_triggers = detect_intraday_triggers(
        "2026-03-01",
        "LEAD",
        "5Min",
        &confluence_bars,
        &[10.0],
        intraday_config::CONFLUENCE_WINDOW,
        intraday_config::DEFAULT_OPENING_RANGE_MINUTES,
    );
    let confluence_types = trigger_types(&confluence_triggers);

    assert!(confluence_types.contains(intraday_config::TRIGGER_VOLUME_DRYUP_CONFIRMATION));
    assert!(confluence_types.contains(intraday_config::TRIGGER_MICRO_CLUSTER_BREAK));
}

#[test]
fn orb_lookback_uses_timeframe_and_opening_range_minutes() {
    assert_eq!(opening_range_bars(10, 2), 5);

    let bars = vec![
        trigger_bar(0, 10.0, 9.95, 9.98, 100.0),
        trigger_bar(1, 10.0, 9.95, 9.99, 100.0),
        trigger_bar(2, 10.0, 9.95, 9.98, 100.0),
        trigger_bar(3, 10.0, 9.95, 9.99, 100.0),
        trigger_bar(4, 10.0, 9.95, 9.98, 100.0),
        trigger_bar(5, 10.6, 10.0, 10.5, 260.0),
    ];

    let triggers = detect_intraday_triggers(
        "2026-03-01",
        "LEAD",
        "2Min",
        &bars,
        &[10.0],
        intraday_config::CONFLUENCE_WINDOW,
        10,
    );
    let trigger_types = trigger_types(&triggers);

    assert!(trigger_types.contains(intraday_config::TRIGGER_ORB_BREAKOUT));
}

#[test]
fn readiness_pipeline_produces_stage_one_two_and_three_rows() -> Result<()> {
    let symbols = fixture_symbols();
    let sector_maps = vec![SectorMap {
        sector: "Technology".to_string(),
        sector_etf: "XLK".to_string(),
        description: "Technology test proxy".to_string(),
    }];
    let daily_prices = fixture_daily_prices();
    let (_, candidates) =
        high_momentum_candidate_symbols(&symbols, &sector_maps, &daily_prices, "2026-03-01", 50);

    assert_eq!(candidates, vec!["LEAD".to_string()]);

    let profile_prices = profile_bars("LEAD", 161.0);
    let trigger_prices = trigger_ready_bars("LEAD", 161.0);
    let result = run_intraday_readiness(IntradayReadinessInput {
        date: "2026-03-01".to_string(),
        symbols,
        sector_maps,
        daily_prices,
        profile_prices,
        trigger_prices,
        profile_timeframe: "30Min".to_string(),
        trigger_timeframe: "5Min".to_string(),
        candidate_limit: 50,
        opening_range_minutes: intraday_config::DEFAULT_OPENING_RANGE_MINUTES,
    })?;

    assert_eq!(result.stage1_count, 1);
    assert_eq!(result.stage2_count, 1);
    assert!(result.stage3_trigger_count > 0);
    assert_eq!(result.setups[0].symbol, "LEAD");
    assert_eq!(
        result.setups[0].primary_label,
        intraday_config::LABEL_STAGE3
    );
    assert_eq!(result.profiles.len(), 1);

    Ok(())
}

fn trigger_types(triggers: &[merryl::domain::models::IntradayTrigger]) -> HashSet<&str> {
    triggers
        .iter()
        .map(|trigger| trigger.trigger_type.as_str())
        .collect()
}

fn fixture_symbols() -> Vec<Symbol> {
    let mut symbols = vec![
        symbol("SPY", "SPDR S&P 500 ETF Trust", "broad_etf"),
        symbol("XLK", "Technology Select Sector SPDR", "sector_etf"),
        symbol("LEAD", "Leader Inc.", "stock"),
    ];
    for idx in 1..=9 {
        symbols.push(symbol(
            &format!("STK{idx}"),
            &format!("Stock {idx}"),
            "stock",
        ));
    }
    symbols
}

fn fixture_daily_prices() -> Vec<DailyPrice> {
    let mut prices = Vec::new();
    prices.extend(daily_series("SPY", 100.0, 0.001, 60, 1_000.0, 1_000.0));
    prices.extend(daily_series("XLK", 100.0, 0.002, 60, 1_000.0, 1_000.0));
    prices.extend(daily_series("LEAD", 100.0, 0.008, 60, 1_000.0, 3_000.0));
    for idx in 1..=9 {
        prices.extend(daily_series(
            &format!("STK{idx}"),
            100.0,
            0.001 + idx as f64 * 0.0001,
            60,
            1_000.0,
            3_000.0,
        ));
    }
    prices
}

fn daily_series(
    symbol: &str,
    start_close: f64,
    daily_return: f64,
    count: usize,
    base_volume: f64,
    final_volume: f64,
) -> Vec<DailyPrice> {
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("fixture date");
    let mut close = start_close;
    (0..count)
        .map(|idx| {
            close *= 1.0 + daily_return;
            let date = start + Duration::days(idx as i64);
            let volume = if idx + 1 == count {
                final_volume
            } else {
                base_volume
            };
            DailyPrice {
                symbol: symbol.to_string(),
                date: date.format("%Y-%m-%d").to_string(),
                open: close * 0.99,
                high: close * 1.03,
                low: close * 0.97,
                close,
                adjusted_close: close,
                volume,
                source: "test-fixture".to_string(),
            }
        })
        .collect()
}

fn profile_bars(symbol: &str, level: f64) -> Vec<IntradayPrice> {
    vec![
        intraday_bar(
            symbol,
            "2026-03-01T14:30:00Z",
            level,
            level,
            level,
            level,
            500.0,
        ),
        intraday_bar(
            symbol,
            "2026-03-01T15:00:00Z",
            level,
            level,
            level,
            level,
            500.0,
        ),
        intraday_bar(
            symbol,
            "2026-03-01T15:30:00Z",
            level * 1.001,
            level,
            level,
            level,
            100.0,
        ),
        intraday_bar(
            symbol,
            "2026-03-01T16:00:00Z",
            level,
            level,
            level,
            level,
            100.0,
        ),
    ]
}

fn trigger_ready_bars(symbol: &str, level: f64) -> Vec<IntradayPrice> {
    (0..6)
        .map(|idx| trigger_bar_for_symbol(symbol, idx, level, level * 0.998, level, 100.0))
        .chain([trigger_bar_for_symbol(
            symbol,
            6,
            level * 1.02,
            level,
            level * 1.018,
            260.0,
        )])
        .collect()
}

fn trigger_bar(idx: usize, high: f64, low: f64, close: f64, volume: f64) -> IntradayPrice {
    trigger_bar_for_symbol("LEAD", idx, high, low, close, volume)
}

fn trigger_bar_for_symbol(
    symbol: &str,
    idx: usize,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
) -> IntradayPrice {
    intraday_bar(
        symbol,
        &format!("2026-03-01T{:02}:00:00Z", 14 + idx),
        close,
        high,
        low,
        close,
        volume,
    )
}

fn intraday_bar(
    symbol: &str,
    ts: &str,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
) -> IntradayPrice {
    IntradayPrice {
        symbol: symbol.to_string(),
        ts: ts.to_string(),
        timeframe: "5Min".to_string(),
        open,
        high,
        low,
        close,
        volume,
        vwap: None,
        source: "test-fixture".to_string(),
    }
}

fn symbol(ticker: &str, name: &str, asset_type: &str) -> Symbol {
    Symbol {
        symbol: ticker.to_string(),
        name: name.to_string(),
        asset_type: asset_type.to_string(),
        sector: Some("Technology".to_string()),
        industry: Some("Software".to_string()),
        exchange: "US".to_string(),
        market_cap: None,
        is_active: true,
    }
}
