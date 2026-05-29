use chrono::{Duration, NaiveDate};

use merryl::actionability::{ActionabilityInput, ActionabilityMetrics, classify_actionability};
use merryl::config::actionability as actionability_config;
use merryl::domain::models::DailyPrice;
use merryl::scoring::{
    average_true_range, distance_pct, gap_pct, highest_close, lowest_close, range_pct, true_range,
};

#[test]
fn indicator_helpers_calculate_price_structure_metrics() {
    let mut history = Vec::new();
    for idx in 0..15 {
        let close = 100.0 + idx as f64;
        history.push(price(idx, close, close + 1.0, close - 1.0, close, close));
    }
    history[14].open = 120.0;
    history[14].high = 124.0;
    history[14].low = 110.0;
    history[14].close = 122.0;
    history[14].adjusted_close = 122.0;

    assert_eq!(highest_close(&history, 14, 10), Some(122.0));
    assert_eq!(lowest_close(&history, 14, 10), Some(105.0));
    assert_close(distance_pct(122.0, 100.0).expect("distance"), 0.22);
    assert_close(range_pct(122.0, 105.0, 122.0).expect("range"), 17.0 / 122.0);
    assert_close(gap_pct(&history, 14).expect("gap"), 120.0 / 113.0 - 1.0);
    assert_close(true_range(&history, 14).expect("true range"), 14.0);
    assert!(average_true_range(&history, 14, 14).expect("atr") > 0.0);
}

#[test]
fn actionability_classifier_assigns_each_primary_bucket() {
    assert_primary(
        input_with_return(0.09),
        metrics(),
        actionability_config::LABEL_EXTENDED_LEADER,
    );

    let mut pullback_metrics = metrics();
    pullback_metrics.distance_from_20d_high_pct = -0.06;
    pullback_metrics.distance_from_20d_ma_pct = 0.02;
    assert_primary(
        input_with_return(0.02),
        pullback_metrics,
        actionability_config::LABEL_PULLBACK_LEADER,
    );

    let mut base_metrics = metrics();
    base_metrics.distance_from_20d_high_pct = -0.01;
    base_metrics.range_10d_pct = 0.04;
    assert_primary(
        input_with_return(0.02),
        base_metrics,
        actionability_config::LABEL_BASE_COMPRESSION_CANDIDATE,
    );

    let mut early_metrics = metrics();
    early_metrics.range_10d_pct = 0.12;
    early_metrics.distance_from_20d_high_pct = -0.20;
    early_metrics.distance_from_20d_ma_pct = 0.02;
    assert_primary(
        input_with_return(0.03),
        early_metrics,
        actionability_config::LABEL_EARLY_ROTATION_CANDIDATE,
    );

    let mut actionable_metrics = metrics();
    actionable_metrics.range_10d_pct = 0.12;
    actionable_metrics.distance_from_20d_high_pct = -0.20;
    assert_primary(
        input_with_return(0.0),
        actionable_metrics,
        actionability_config::LABEL_ACTIONABLE_LEADER,
    );

    let event_input = ActionabilityInput {
        relative_return_vs_sector: -0.01,
        relative_return_vs_spy: -0.01,
        trend_state: "below_trend",
        catalyst_status: "recent_news:1",
        ..input_with_return(0.0)
    };
    assert_primary(
        event_input,
        metrics(),
        actionability_config::LABEL_EVENT_WATCH_UNCONFIRMED,
    );

    let unclassified_input = ActionabilityInput {
        relative_return_vs_sector: -0.01,
        relative_return_vs_spy: -0.01,
        trend_state: "below_trend",
        ..input_with_return(0.0)
    };
    assert_primary(
        unclassified_input,
        metrics(),
        actionability_config::LABEL_UNCLASSIFIED_LEADER,
    );
}

fn assert_primary(
    input: ActionabilityInput<'_>,
    metrics: ActionabilityMetrics,
    expected_primary: &str,
) {
    let classification = classify_actionability(&input, &metrics);
    assert_eq!(classification.primary, expected_primary);
    assert!(
        classification
            .labels
            .contains(&expected_primary.to_string())
    );
}

fn input_with_return(return_5d: f64) -> ActionabilityInput<'static> {
    ActionabilityInput {
        score: 80.0,
        sector_score: 75.0,
        return_1d: 0.0,
        return_5d,
        relative_return_vs_sector: 0.02,
        relative_return_vs_spy: 0.03,
        relative_volume: 1.3,
        trend_state: "above_20d_50d",
        catalyst_status: "pending_source",
    }
}

fn metrics() -> ActionabilityMetrics {
    ActionabilityMetrics {
        ma_20d: 100.0,
        ma_50d: 98.0,
        distance_from_20d_ma_pct: 0.03,
        distance_from_50d_ma_pct: 0.05,
        atr_14d: 2.0,
        atr_14d_pct: 0.02,
        atr_extension_from_20d_ma: 1.5,
        atr_extension_from_50d_ma: 2.4,
        high_20d: 104.0,
        high_60d: 106.0,
        distance_from_20d_high_pct: -0.02,
        distance_from_60d_high_pct: -0.04,
        range_10d_pct: 0.10,
        gap_pct: 0.0,
        true_range_pct: 0.02,
        complete: true,
    }
}

fn price(idx: i64, open: f64, high: f64, low: f64, close: f64, adjusted_close: f64) -> DailyPrice {
    let date =
        NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid fixture date") + Duration::days(idx);
    DailyPrice {
        symbol: "TEST".to_string(),
        date: date.format("%Y-%m-%d").to_string(),
        open,
        high,
        low,
        close,
        adjusted_close,
        volume: 1_000_000.0,
        source: "test-fixture".to_string(),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_000_001,
        "actual {actual} expected {expected}"
    );
}
