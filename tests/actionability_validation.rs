use std::fs;

use anyhow::Result;

use merryl::config::actionability as actionability_config;
use merryl::domain::models::{DailyPrice, SectorMap, StockScore, WatchlistRow};
use merryl::output::write_actionability_validation_outputs;
use merryl::validation::{
    ActionabilitySummaryRow, ActionabilityValidationInput, run_actionability_validation,
};

#[test]
fn actionability_validation_groups_stored_labels_and_forward_returns() -> Result<()> {
    let metrics = run_actionability_validation(actionability_input())?;

    assert_eq!(
        metrics.validation_scope.purpose,
        "watchlist_actionability_validation"
    );
    assert_eq!(metrics.watchlist_row_count, 3);
    assert_eq!(metrics.scored_watchlist_row_count, 3);
    assert_eq!(metrics.extended_leader_row_count, 1);
    assert_eq!(metrics.useful_review_row_count, 1);
    assert_eq!(metrics.unclassified_leader_row_count, 1);
    assert_eq!(metrics.forward_observation_count, 3);
    assert_eq!(metrics.skipped_missing_future_bars, 12);

    assert_eq!(
        summary(
            &metrics.summaries,
            actionability_config::LABEL_ALL_WATCHLIST,
            1
        )
        .count,
        3
    );
    assert_eq!(
        summary(
            &metrics.summaries,
            actionability_config::LABEL_EXTENDED_LEADER,
            1
        )
        .count,
        1
    );
    assert_eq!(
        summary(
            &metrics.summaries,
            actionability_config::LABEL_EARLY_ROTATION_CANDIDATE,
            1
        )
        .count,
        1
    );
    assert_eq!(
        summary(
            &metrics.summaries,
            actionability_config::LABEL_UNCLASSIFIED_LEADER,
            1
        )
        .count,
        1
    );

    let extended = summary(
        &metrics.summaries,
        actionability_config::LABEL_EXTENDED_LEADER,
        1,
    );
    assert_close(extended.average_forward_return, 0.10);
    assert_close(extended.average_relative_return_vs_spy, 0.10);
    assert_close(extended.average_relative_return_vs_sector, 0.05);

    Ok(())
}

#[test]
fn actionability_validation_writes_markdown_and_csv_outputs() -> Result<()> {
    let metrics = run_actionability_validation(actionability_input())?;
    let outputs = write_actionability_validation_outputs(&metrics)?;

    assert!(outputs.report.exists());
    assert!(outputs.summary_export.exists());

    let report = fs::read_to_string(&outputs.report)?;
    assert!(report.contains("Actionability Validation"));
    assert!(report.contains("does not change score weights"));
    assert!(report.contains("Actionability Summary"));
    assert!(report.contains(actionability_config::LABEL_EXTENDED_LEADER));
    assert!(report.contains("not a trading recommendation"));

    let _ = fs::remove_file(outputs.report);
    let _ = fs::remove_file(outputs.summary_export);

    Ok(())
}

#[test]
fn actionability_validation_uses_stored_score_date_labels_not_future_prices() -> Result<()> {
    let mut input = actionability_input();
    input.stock_scores = vec![stock(
        "2026-01-01",
        1,
        "EARLY",
        actionability_config::LABEL_EARLY_ROTATION_CANDIDATE,
        &[actionability_config::LABEL_EARLY_ROTATION_CANDIDATE],
    )];
    input.watchlist_rows = watchlist_rows(&input.stock_scores);
    input.prices = vec![
        price("SPY", "2026-01-01", 100.0),
        price("SPY", "2026-01-02", 100.0),
        price("ETF1", "2026-01-01", 100.0),
        price("ETF1", "2026-01-02", 100.0),
        price("EARLY", "2026-01-01", 100.0),
        price("EARLY", "2026-01-02", 140.0),
    ];

    let metrics = run_actionability_validation(input)?;

    assert!(
        metrics
            .summaries
            .iter()
            .any(|row| row.group == actionability_config::LABEL_EARLY_ROTATION_CANDIDATE)
    );
    assert!(
        !metrics
            .summaries
            .iter()
            .any(|row| row.group == actionability_config::LABEL_EXTENDED_LEADER)
    );

    Ok(())
}

fn actionability_input() -> ActionabilityValidationInput {
    let date = "2026-01-01";
    let next_date = "2026-01-02";
    let stock_scores = vec![
        stock(
            date,
            1,
            "EXT",
            actionability_config::LABEL_EXTENDED_LEADER,
            &[actionability_config::LABEL_EXTENDED_LEADER],
        ),
        stock(
            date,
            2,
            "EARLY",
            actionability_config::LABEL_EARLY_ROTATION_CANDIDATE,
            &[actionability_config::LABEL_EARLY_ROTATION_CANDIDATE],
        ),
        stock(
            date,
            3,
            "UNCL",
            actionability_config::LABEL_UNCLASSIFIED_LEADER,
            &[actionability_config::LABEL_UNCLASSIFIED_LEADER],
        ),
    ];
    let mut prices = vec![
        price("SPY", date, 100.0),
        price("SPY", next_date, 100.0),
        price("ETF1", date, 100.0),
        price("ETF1", next_date, 105.0),
    ];
    for (symbol, next_close) in [("EXT", 110.0), ("EARLY", 108.0), ("UNCL", 98.0)] {
        prices.push(price(symbol, date, 100.0));
        prices.push(price(symbol, next_date, next_close));
    }

    ActionabilityValidationInput {
        from_date: date.to_string(),
        to_date: date.to_string(),
        watchlist_rows: watchlist_rows(&stock_scores),
        stock_scores,
        sector_maps: vec![SectorMap {
            sector: "Sector1".to_string(),
            sector_etf: "ETF1".to_string(),
            description: "Sector 1 test proxy".to_string(),
        }],
        prices,
    }
}

fn watchlist_rows(stock_scores: &[StockScore]) -> Vec<WatchlistRow> {
    stock_scores
        .iter()
        .map(|score| WatchlistRow {
            date: score.date.clone(),
            rank: score.rank,
            symbol: score.symbol.clone(),
            score: score.score,
            reason: score.explanation.clone(),
        })
        .collect()
}

fn summary<'a>(
    rows: &'a [ActionabilitySummaryRow],
    group: &str,
    horizon: usize,
) -> &'a ActionabilitySummaryRow {
    rows.iter()
        .find(|row| row.group == group && row.horizon == horizon)
        .expect("actionability summary row")
}

fn stock(
    date: &str,
    rank: usize,
    symbol: &str,
    primary_actionability: &str,
    actionability_labels: &[&str],
) -> StockScore {
    StockScore {
        date: date.to_string(),
        rank,
        symbol: symbol.to_string(),
        name: format!("{symbol} Inc."),
        sector: "Sector1".to_string(),
        industry: "Industry1".to_string(),
        score: 100.0 - rank as f64,
        sector_score: 75.0,
        return_1d: 0.0,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_sector: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        avg_dollar_volume: 1_000_000.0,
        trend_state: "fixture".to_string(),
        catalyst_status: "pending_source".to_string(),
        components_json: serde_json::json!({
            "primary_actionability": primary_actionability,
            "actionability_labels": actionability_labels
        })
        .to_string(),
        explanation: "fixture stock score".to_string(),
    }
}

fn price(symbol: &str, date: &str, close: f64) -> DailyPrice {
    DailyPrice {
        symbol: symbol.to_string(),
        date: date.to_string(),
        open: close,
        high: close,
        low: close,
        close,
        adjusted_close: close,
        volume: 1_000_000.0,
        source: "test-fixture".to_string(),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_000_001,
        "expected {expected}, got {actual}"
    );
}
