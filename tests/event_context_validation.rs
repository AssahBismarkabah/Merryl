use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use tempfile::tempdir;

use merryl::domain::models::{DailyPrice, SectorMap, StockScore, WatchlistRow};
use merryl::output::write_event_context_validation_outputs;
use merryl::scoring::preserve_existing_catalyst_statuses;
use merryl::storage::Database;
use merryl::validation::{
    EventContextSummaryRow, EventContextValidationInput, run_event_context_validation,
};

#[test]
fn event_context_validation_groups_watchlist_labels_and_forward_returns() -> Result<()> {
    let input = event_context_input();

    let metrics = run_event_context_validation(input)?;

    assert_eq!(
        metrics.validation_scope.purpose,
        "event_context_watchlist_validation"
    );
    assert_eq!(metrics.watchlist_row_count, 5);
    assert_eq!(metrics.scored_watchlist_row_count, 5);
    assert_eq!(metrics.event_context_row_count, 4);
    assert_eq!(metrics.pending_source_row_count, 1);
    assert_eq!(metrics.forward_observation_count, 5);
    assert_eq!(metrics.event_context_forward_observation_count, 4);
    assert_eq!(metrics.skipped_missing_future_bars, 20);

    assert_eq!(summary(&metrics.summaries, "all_watchlist", 1).count, 5);
    assert_eq!(summary(&metrics.summaries, "event_context", 1).count, 4);
    assert_eq!(summary(&metrics.summaries, "pending_source", 1).count, 1);
    assert_eq!(summary(&metrics.summaries, "recent_news", 1).count, 2);
    assert_eq!(summary(&metrics.summaries, "earnings", 1).count, 2);
    assert_eq!(summary(&metrics.summaries, "filing", 1).count, 2);
    assert_eq!(summary(&metrics.summaries, "event_risk", 1).count, 3);
    assert_eq!(
        summary(&metrics.summaries, "multiple_event_types", 1).count,
        1
    );

    let event_context = summary(&metrics.summaries, "event_context", 1);
    assert_close(event_context.hit_rate, 0.75);
    assert_close(event_context.average_forward_return, 0.0625);
    assert_close(event_context.average_relative_return_vs_spy, 0.0625);
    assert_close(event_context.average_relative_return_vs_sector, 0.0625);

    assert!(!metrics.summaries.iter().any(|row| row.horizon == 5));

    Ok(())
}

#[test]
fn event_context_validation_writes_markdown_and_csv_outputs() -> Result<()> {
    let metrics = run_event_context_validation(event_context_input())?;
    let outputs = write_event_context_validation_outputs(&metrics)?;

    assert!(outputs.report.exists());
    assert!(outputs.summary_export.exists());

    let report = fs::read_to_string(&outputs.report)?;
    assert!(report.contains("Event Context Validation"));
    assert!(report.contains("does not change score weights"));
    assert!(report.contains("Event Context Summary"));
    assert!(report.contains("pending_source"));
    assert!(report.contains("not a trading recommendation"));

    let _ = fs::remove_file(outputs.report);
    let _ = fs::remove_file(outputs.summary_export);

    Ok(())
}

#[test]
fn daily_rewrite_preserves_prior_non_pending_catalyst_statuses_only() {
    let mut stocks = vec![
        stock("2026-01-01", 1, "OLD", "pending_source"),
        stock("2026-01-01", 2, "NONE", "pending_source"),
        stock("2026-01-02", 1, "OLD", "pending_source"),
    ];
    let existing_statuses = HashMap::from([
        (
            ("2026-01-01".to_string(), "OLD".to_string()),
            "recent_news:1".to_string(),
        ),
        (
            ("2026-01-01".to_string(), "NONE".to_string()),
            "pending_source".to_string(),
        ),
        (
            ("2026-01-02".to_string(), "OLD".to_string()),
            "recent_news:9".to_string(),
        ),
    ]);

    preserve_existing_catalyst_statuses(&mut stocks, &existing_statuses, "2026-01-02");

    assert_eq!(stocks[0].catalyst_status, "recent_news:1");
    assert_eq!(stocks[1].catalyst_status, "pending_source");
    assert_eq!(stocks[2].catalyst_status, "pending_source");
}

#[test]
fn storage_reads_prior_non_pending_catalyst_statuses_and_watchlists() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    db.replace_stock_scores(
        "2026-01-01",
        &[
            stock("2026-01-01", 1, "OLD", "recent_news:1"),
            stock("2026-01-01", 2, "NONE", "pending_source"),
        ],
    )?;
    db.replace_stock_scores(
        "2026-01-02",
        &[stock("2026-01-02", 1, "LATEST", "filing:8-K")],
    )?;
    db.replace_watchlist(
        "2026-01-01",
        &[
            stock("2026-01-01", 1, "OLD", "recent_news:1"),
            stock("2026-01-01", 2, "NONE", "pending_source"),
        ],
    )?;

    let statuses = db.non_pending_stock_catalyst_statuses_before("2026-01-02")?;
    assert_eq!(statuses.len(), 1);
    assert_eq!(
        statuses.get(&("2026-01-01".to_string(), "OLD".to_string())),
        Some(&"recent_news:1".to_string())
    );

    let watchlists = db.watchlists_between("2026-01-01", "2026-01-02")?;
    assert_eq!(watchlists.len(), 2);
    assert_eq!(watchlists[0].symbol, "OLD");

    Ok(())
}

fn event_context_input() -> EventContextValidationInput {
    let date = "2026-01-01";
    let next_date = "2026-01-02";
    let symbols = [
        ("NEWS", "recent_news:2", 110.0),
        ("EARN", "earnings:2026-01-10", 90.0),
        ("FILE", "filing:8-K", 105.0),
        (
            "MULTI",
            "recent_news:1 | earnings:2026-01-11 | filing:10-Q",
            120.0,
        ),
        ("PEND", "pending_source", 95.0),
    ];

    let mut prices = vec![
        price("SPY", date, 100.0),
        price("SPY", next_date, 100.0),
        price("ETF1", date, 100.0),
        price("ETF1", next_date, 100.0),
    ];
    let stock_scores = symbols
        .iter()
        .enumerate()
        .map(|(idx, (symbol, catalyst_status, _))| stock(date, idx + 1, symbol, catalyst_status))
        .collect::<Vec<_>>();
    let watchlist_rows = stock_scores
        .iter()
        .map(|score| WatchlistRow {
            date: score.date.clone(),
            rank: score.rank,
            symbol: score.symbol.clone(),
            score: score.score,
            reason: score.explanation.clone(),
        })
        .collect();

    for (symbol, _, next_close) in symbols {
        prices.push(price(symbol, date, 100.0));
        prices.push(price(symbol, next_date, next_close));
    }

    EventContextValidationInput {
        from_date: date.to_string(),
        to_date: date.to_string(),
        stock_scores,
        watchlist_rows,
        sector_maps: vec![SectorMap {
            sector: "Sector1".to_string(),
            sector_etf: "ETF1".to_string(),
            description: "Sector 1 test proxy".to_string(),
        }],
        prices,
    }
}

fn summary<'a>(
    rows: &'a [EventContextSummaryRow],
    group: &str,
    horizon: usize,
) -> &'a EventContextSummaryRow {
    rows.iter()
        .find(|row| row.group == group && row.horizon == horizon)
        .expect("event context summary row")
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000000001,
        "expected {expected}, got {actual}"
    );
}

fn stock(date: &str, rank: usize, symbol: &str, catalyst_status: &str) -> StockScore {
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
        catalyst_status: catalyst_status.to_string(),
        components_json: "{}".to_string(),
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
