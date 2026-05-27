use std::fs;

use anyhow::Result;
use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use tempfile::tempdir;

use merryl::backtest::{BacktestInput, BacktestSummaryRow, run_backtest_analysis};
use merryl::domain::models::{
    DailyPrice, IndustryScore, IndustryScoreSnapshot, SectorMap, SectorScore, StockScore,
};
use merryl::output::write_backtest_outputs;
use merryl::storage::Database;

#[test]
fn backtest_calculates_forward_returns_deciles_and_summary_metrics() -> Result<()> {
    let fixture = BacktestFixture::new(62, 2);

    let metrics = run_backtest_analysis(fixture.input())?;

    let sector_decile_10 = summary(&metrics.summaries, "sector", 1, 10);
    assert_eq!(sector_decile_10.count, 2);
    assert_close(sector_decile_10.hit_rate, 0.5);
    assert_close(sector_decile_10.average_forward_return, 0.025);
    assert_close(sector_decile_10.median_forward_return, 0.025);
    assert_close(
        sector_decile_10.average_relative_return,
        (0.04 - 0.00990099009900991) / 2.0,
    );
    assert_eq!(sector_decile_10.average_relative_return_vs_sector, None);

    let sector_component_decile_10 =
        summary(&metrics.summaries, "sector_component_return_20d", 1, 10);
    assert_eq!(sector_component_decile_10.count, 2);
    assert_eq!(
        metrics.sector_component_observation_count,
        metrics.sector_observation_count * 7
    );

    let stock_decile_10 = summary(&metrics.summaries, "stock", 1, 10);
    assert_eq!(stock_decile_10.count, 2);
    assert_close(stock_decile_10.hit_rate, 0.5);
    assert_close(stock_decile_10.average_forward_return, 0.0);
    assert_close(stock_decile_10.median_forward_return, 0.0);
    assert_close(stock_decile_10.average_relative_return, -0.025);
    assert_close(stock_decile_10.median_relative_return, -0.025);
    assert_close(
        stock_decile_10.average_relative_return_vs_spy,
        (0.09 - 0.10990099009900991) / 2.0,
    );
    assert_close(
        stock_decile_10
            .average_relative_return_vs_sector
            .expect("stock sector-relative return"),
        -0.025,
    );

    let stock_decile_1 = summary(&metrics.summaries, "stock", 1, 1);
    assert_eq!(stock_decile_1.count, 2);

    let industry_decile_10 = summary(&metrics.summaries, "stock_by_industry", 1, 10);
    assert_eq!(industry_decile_10.count, 20);
    assert_eq!(
        metrics.industry_stock_observation_count,
        metrics.stock_observation_count
    );

    Ok(())
}

#[test]
fn backtest_groups_stock_returns_by_industry_score_decile() -> Result<()> {
    let date = "2026-01-01".to_string();
    let next_date = "2026-01-02".to_string();
    let metrics = run_backtest_analysis(BacktestInput {
        from_date: date.clone(),
        to_date: date.clone(),
        sector_scores: vec![SectorScore {
            date: date.clone(),
            sector: "Sector1".to_string(),
            sector_etf: "ETF1".to_string(),
            score: 50.0,
            rank: 1,
            return_1d: 0.0,
            return_5d: 0.0,
            return_20d: 0.0,
            return_60d: 0.0,
            relative_return_vs_spy: 0.0,
            relative_volume: 1.0,
            breadth_20d: 0.5,
            breadth_50d: 0.5,
            rank_change: 0.0,
            explanation: "fixture sector score".to_string(),
        }],
        industry_scores: (1..=10)
            .map(|idx| IndustryScoreSnapshot {
                date: date.clone(),
                industry: format!("Industry{idx}"),
                sector: "Sector1".to_string(),
                score: idx as f64,
                rank: 11 - idx,
            })
            .collect(),
        stock_scores: (1..=10)
            .map(|idx| StockScore {
                date: date.clone(),
                rank: idx,
                symbol: format!("STK{idx}"),
                name: format!("Stock {idx}"),
                sector: "Sector1".to_string(),
                industry: format!("Industry{idx}"),
                score: 50.0,
                sector_score: 50.0,
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
                components_json: "{}".to_string(),
                explanation: "fixture stock score".to_string(),
            })
            .collect(),
        sector_maps: vec![SectorMap {
            sector: "Sector1".to_string(),
            sector_etf: "ETF1".to_string(),
            description: "Sector 1 test proxy".to_string(),
        }],
        prices: industry_validation_prices(&date, &next_date),
    })?;

    let strongest = summary(&metrics.summaries, "stock_by_industry", 1, 10);
    let weakest = summary(&metrics.summaries, "stock_by_industry", 1, 1);

    assert_eq!(strongest.count, 1);
    assert_eq!(weakest.count, 1);
    assert_close(
        strongest
            .average_relative_return_vs_sector
            .expect("strong industry sector-relative return"),
        0.10,
    );
    assert_close(
        weakest
            .average_relative_return_vs_sector
            .expect("weak industry sector-relative return"),
        -0.10,
    );
    assert!(
        strongest.average_relative_return > weakest.average_relative_return,
        "stronger industry decile should beat weaker industry decile"
    );

    Ok(())
}

#[test]
fn backtest_skips_only_horizons_without_future_bars() -> Result<()> {
    let fixture = BacktestFixture::new(6, 1);

    let metrics = run_backtest_analysis(fixture.input())?;

    assert!(metrics.summaries.iter().any(|row| row.horizon == 1));
    assert!(metrics.summaries.iter().any(|row| row.horizon == 5));
    assert!(!metrics.summaries.iter().any(|row| row.horizon == 10));
    assert!(!metrics.summaries.iter().any(|row| row.horizon == 20));
    assert!(!metrics.summaries.iter().any(|row| row.horizon == 60));

    Ok(())
}

#[test]
fn backtest_reads_sqlite_inputs_stores_results_and_writes_outputs() -> Result<()> {
    let fixture = BacktestFixture::new(62, 2);
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;
    db.upsert_sector_maps(&fixture.sector_maps)?;
    db.upsert_prices(&fixture.prices)?;

    for date in &fixture.score_dates {
        db.replace_sector_scores(date, &sector_scores(date))?;
        db.replace_industry_scores(date, &persisted_industry_scores(date))?;
        db.replace_stock_scores(date, &stock_scores(date))?;
    }

    let sector_scores = db.sector_scores_between(&fixture.from_date, &fixture.to_date)?;
    let industry_scores = db.industry_scores_between(&fixture.from_date, &fixture.to_date)?;
    let stock_scores = db.stock_scores_between(&fixture.from_date, &fixture.to_date)?;
    let sector_maps = db.sector_maps()?;
    let prices = db.daily_prices()?;
    let metrics = run_backtest_analysis(BacktestInput {
        from_date: fixture.from_date.clone(),
        to_date: fixture.to_date.clone(),
        sector_scores,
        industry_scores,
        stock_scores,
        sector_maps,
        prices,
    })?;

    let outputs = write_backtest_outputs(&metrics)?;
    assert!(outputs.report.exists());
    assert!(outputs.summary_export.exists());
    let report = fs::read_to_string(&outputs.report)?;
    assert!(report.contains("not a trading recommendation"));
    assert!(report.contains("sector_component_return_20d"));
    assert!(report.contains("stock_by_industry"));

    let metrics_json = serde_json::to_string(&metrics)?;
    let result_id = db.insert_backtest_result(
        "test_backtest",
        &fixture.from_date,
        &fixture.to_date,
        "{}",
        &metrics_json,
    )?;
    assert!(result_id > 0);

    let conn = Connection::open(db_path)?;
    let stored_metrics: String =
        conn.query_row("SELECT metrics_json FROM backtest_results", [], |row| {
            row.get(0)
        })?;
    assert!(stored_metrics.contains("summaries"));

    let _ = fs::remove_file(outputs.report);
    let _ = fs::remove_file(outputs.summary_export);

    Ok(())
}

fn summary<'a>(
    rows: &'a [BacktestSummaryRow],
    entity_type: &str,
    horizon: usize,
    decile: usize,
) -> &'a BacktestSummaryRow {
    rows.iter()
        .find(|row| {
            row.entity_type == entity_type && row.horizon == horizon && row.decile == decile
        })
        .expect("backtest summary row")
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000000001,
        "expected {expected}, got {actual}"
    );
}

struct BacktestFixture {
    from_date: String,
    to_date: String,
    score_dates: Vec<String>,
    sector_maps: Vec<SectorMap>,
    prices: Vec<DailyPrice>,
}

impl BacktestFixture {
    fn new(price_days: usize, score_days: usize) -> Self {
        let dates = dates(price_days);
        let score_dates = dates.iter().take(score_days).cloned().collect::<Vec<_>>();

        Self {
            from_date: score_dates.first().expect("from date").clone(),
            to_date: score_dates.last().expect("to date").clone(),
            score_dates,
            sector_maps: sector_maps(),
            prices: prices(&dates),
        }
    }

    fn input(&self) -> BacktestInput {
        BacktestInput {
            from_date: self.from_date.clone(),
            to_date: self.to_date.clone(),
            sector_scores: self
                .score_dates
                .iter()
                .flat_map(|date| sector_scores(date))
                .collect(),
            industry_scores: self
                .score_dates
                .iter()
                .flat_map(|date| industry_score_snapshots(date))
                .collect(),
            stock_scores: self
                .score_dates
                .iter()
                .flat_map(|date| stock_scores(date))
                .collect(),
            sector_maps: self.sector_maps.clone(),
            prices: self.prices.clone(),
        }
    }
}

fn dates(days: usize) -> Vec<String> {
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid fixture date");
    (0..days)
        .map(|idx| {
            (start + Duration::days(idx as i64))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect()
}

fn sector_maps() -> Vec<SectorMap> {
    (1..=10)
        .map(|idx| SectorMap {
            sector: format!("Sector{idx}"),
            sector_etf: format!("ETF{idx}"),
            description: format!("Sector {idx} test proxy"),
        })
        .collect()
}

fn sector_scores(date: &str) -> Vec<SectorScore> {
    (1..=10)
        .map(|idx| SectorScore {
            date: date.to_string(),
            sector: format!("Sector{idx}"),
            sector_etf: format!("ETF{idx}"),
            score: idx as f64,
            rank: 11 - idx,
            return_1d: 0.0,
            return_5d: 0.0,
            return_20d: idx as f64 / 100.0,
            return_60d: 0.0,
            relative_return_vs_spy: 0.0,
            relative_volume: 1.0,
            breadth_20d: 0.5,
            breadth_50d: 0.5,
            rank_change: 0.0,
            explanation: "fixture sector score".to_string(),
        })
        .collect()
}

fn industry_score_snapshots(date: &str) -> Vec<IndustryScoreSnapshot> {
    vec![IndustryScoreSnapshot {
        date: date.to_string(),
        industry: "Fixture Industry".to_string(),
        sector: "Sector10".to_string(),
        score: 50.0,
        rank: 1,
    }]
}

fn persisted_industry_scores(date: &str) -> Vec<IndustryScore> {
    vec![IndustryScore {
        date: date.to_string(),
        industry: "Fixture Industry".to_string(),
        sector: "Sector10".to_string(),
        score: 50.0,
        rank: 1,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_sector: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        breadth_20d: 0.0,
        breadth_50d: 0.0,
        high_20d_rate: 0.0,
        member_count: 10,
        components_json: "{}".to_string(),
    }]
}

fn stock_scores(date: &str) -> Vec<StockScore> {
    (1..=10)
        .map(|idx| StockScore {
            date: date.to_string(),
            rank: 11 - idx,
            symbol: format!("STK{idx}"),
            name: format!("Stock {idx}"),
            sector: "Sector10".to_string(),
            industry: "Fixture Industry".to_string(),
            score: idx as f64,
            sector_score: 10.0,
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
            components_json: "{}".to_string(),
            explanation: "fixture stock score".to_string(),
        })
        .collect()
}

fn industry_validation_prices(date: &str, next_date: &str) -> Vec<DailyPrice> {
    let mut prices = vec![
        price("SPY", date, 100.0),
        price("SPY", next_date, 100.0),
        price("ETF1", date, 100.0),
        price("ETF1", next_date, 100.0),
    ];

    for idx in 1..=10 {
        let close = match idx {
            1 => 90.0,
            10 => 110.0,
            _ => 100.0,
        };
        prices.push(price(&format!("STK{idx}"), date, 100.0));
        prices.push(price(&format!("STK{idx}"), next_date, close));
    }

    prices
}

fn prices(dates: &[String]) -> Vec<DailyPrice> {
    let mut prices = Vec::new();
    prices.extend(price_series(dates, "SPY", &[100.0, 101.0, 102.0]));

    for idx in 1..=10 {
        let symbol = format!("ETF{idx}");
        let closes = if idx == 10 {
            vec![100.0, 105.0, 105.0]
        } else {
            vec![100.0, 100.0 + idx as f64, 100.0 + idx as f64]
        };
        prices.extend(price_series(dates, &symbol, &closes));
    }

    for idx in 1..=10 {
        let symbol = format!("STK{idx}");
        let closes = if idx == 10 {
            vec![100.0, 110.0, 99.0]
        } else {
            vec![100.0, 100.0 + idx as f64, 100.0 + idx as f64]
        };
        prices.extend(price_series(dates, &symbol, &closes));
    }

    prices
}

fn price_series(dates: &[String], symbol: &str, seed_closes: &[f64]) -> Vec<DailyPrice> {
    dates
        .iter()
        .enumerate()
        .map(|(idx, date)| {
            let close = seed_closes
                .get(idx)
                .copied()
                .unwrap_or(*seed_closes.last().expect("seed close"));
            price(symbol, date, close)
        })
        .collect()
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
