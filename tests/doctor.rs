use anyhow::Result;
use chrono::{Duration, NaiveDate};
use tempfile::tempdir;

use merryl::config::{
    quality, scoring,
    universe::{
        ASSET_BROAD_ETF, ASSET_MACRO_ETF, ASSET_SECTOR_ETF, BROAD_ETFS, EXCHANGE_US, MACRO_ETFS,
        SECTOR_ETFS,
    },
};
use merryl::domain::models::{
    DailyPrice, IndustryScore, MarketRegimeScore, SectorMap, SectorScore, StockScore, Symbol,
};
use merryl::storage::Database;
use merryl::workflows::doctor_for_db_path;

#[test]
fn doctor_reports_missing_core_data_for_empty_database() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let db = Database::open(&db_path)?;
    db.migrate()?;

    let checks = doctor_for_db_path(&db_path)?;

    assert_contains(&checks, "missing: required market symbols");
    assert_contains(&checks, "missing: required sector map entries");
    assert_contains(&checks, "missing: required ETF price coverage");
    assert_contains(&checks, "missing: historical score coverage");
    assert_contains(&checks, "missing: latest score date");
    assert_contains(&checks, "missing: latest score rows");

    Ok(())
}

#[test]
fn doctor_accepts_complete_core_data_fixture() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    let dates = dates(quality::MIN_REQUIRED_PRICE_BARS as usize);
    db.upsert_symbols(&required_symbols())?;
    db.upsert_sector_maps(&sector_maps())?;
    db.upsert_prices(&required_prices(&dates))?;

    for date in &dates {
        db.replace_market_regime(&market_regime_score(date))?;
        db.replace_sector_scores(date, &sector_scores(date))?;
        db.replace_industry_scores(date, &industry_scores(date))?;
        db.replace_stock_scores(date, &stock_scores(date))?;
        db.replace_watchlist(date, &stock_scores(date))?;
    }
    drop(db);

    let checks = doctor_for_db_path(&db_path)?;

    assert_contains(&checks, "ok: required market symbols present");
    assert_contains(&checks, "ok: required sector map entries present");
    assert_contains(&checks, "ok: required ETF price coverage");
    assert_contains(&checks, "ok: historical score coverage");
    assert_contains(
        &checks,
        "ok: latest score date matches benchmark price date",
    );
    assert_contains(&checks, "ok: latest score rows");
    assert_not_contains(&checks, "missing: required market symbols");
    assert_not_contains(&checks, "missing: required sector map entries");
    assert_not_contains(&checks, "missing: required ETF price coverage");
    assert_not_contains(&checks, "missing: historical score coverage");
    assert_not_contains(&checks, "missing: latest score date");
    assert_not_contains(&checks, "missing: latest score rows");

    Ok(())
}

fn assert_contains(checks: &[String], expected: &str) {
    assert!(
        checks.iter().any(|check| check.contains(expected)),
        "expected doctor output to contain {expected}, got {checks:#?}"
    );
}

fn assert_not_contains(checks: &[String], unexpected: &str) {
    assert!(
        checks.iter().all(|check| !check.contains(unexpected)),
        "expected doctor output not to contain {unexpected}, got {checks:#?}"
    );
}

fn required_symbols() -> Vec<Symbol> {
    let broad = BROAD_ETFS
        .iter()
        .map(|(ticker, name)| symbol(ticker, name, ASSET_BROAD_ETF, None));
    let macro_symbols = MACRO_ETFS
        .iter()
        .map(|(ticker, name)| symbol(ticker, name, ASSET_MACRO_ETF, None));
    let sectors = SECTOR_ETFS.iter().map(|(sector, ticker)| {
        symbol(
            ticker,
            &format!("{sector} sector ETF proxy"),
            ASSET_SECTOR_ETF,
            Some(sector),
        )
    });

    broad.chain(macro_symbols).chain(sectors).collect()
}

fn required_prices(dates: &[String]) -> Vec<DailyPrice> {
    required_tickers()
        .iter()
        .flat_map(|ticker| {
            dates.iter().enumerate().map(|(idx, date)| {
                let close = 100.0 + idx as f64;
                DailyPrice {
                    symbol: (*ticker).to_string(),
                    date: date.clone(),
                    open: close,
                    high: close,
                    low: close,
                    close,
                    adjusted_close: close,
                    volume: 1_000_000.0,
                    source: "test-fixture".to_string(),
                }
            })
        })
        .collect()
}

fn required_tickers() -> Vec<&'static str> {
    BROAD_ETFS
        .iter()
        .map(|(ticker, _)| *ticker)
        .chain(MACRO_ETFS.iter().map(|(ticker, _)| *ticker))
        .chain(SECTOR_ETFS.iter().map(|(_, ticker)| *ticker))
        .collect()
}

fn sector_maps() -> Vec<SectorMap> {
    SECTOR_ETFS
        .iter()
        .map(|(sector, sector_etf)| SectorMap {
            sector: (*sector).to_string(),
            sector_etf: (*sector_etf).to_string(),
            description: format!("{sector} sector ETF proxy"),
        })
        .collect()
}

fn market_regime_score(date: &str) -> MarketRegimeScore {
    MarketRegimeScore {
        date: date.to_string(),
        label: "risk_on".to_string(),
        score: 75.0,
        spy_return_20d: 0.10,
        spy_return_60d: 0.20,
        qqq_relative_return_vs_spy: 0.01,
        iwm_relative_return_vs_spy: 0.02,
        dia_relative_return_vs_spy: 0.0,
        components_json: "{}".to_string(),
        explanation: "fixture regime".to_string(),
    }
}

fn sector_scores(date: &str) -> Vec<SectorScore> {
    SECTOR_ETFS
        .iter()
        .enumerate()
        .map(|(idx, (sector, sector_etf))| SectorScore {
            date: date.to_string(),
            sector: (*sector).to_string(),
            sector_etf: (*sector_etf).to_string(),
            score: 100.0 - idx as f64,
            rank: idx + 1,
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
        })
        .collect()
}

fn industry_scores(date: &str) -> Vec<IndustryScore> {
    vec![IndustryScore {
        date: date.to_string(),
        industry: "Fixture Industry".to_string(),
        sector: "Technology".to_string(),
        score: 80.0,
        rank: 1,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_sector: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        breadth_20d: 0.5,
        breadth_50d: 0.5,
        high_20d_rate: 0.0,
        member_count: scoring::STOCK_WATCHLIST_LIMIT,
        components_json: "{}".to_string(),
    }]
}

fn stock_scores(date: &str) -> Vec<StockScore> {
    (1..=scoring::STOCK_WATCHLIST_LIMIT)
        .map(|idx| StockScore {
            date: date.to_string(),
            rank: idx,
            symbol: format!("STK{idx}"),
            name: format!("Stock {idx}"),
            sector: "Technology".to_string(),
            industry: "Fixture Industry".to_string(),
            score: 100.0 - idx as f64,
            sector_score: 90.0,
            return_1d: 0.0,
            return_5d: 0.0,
            return_20d: 0.0,
            return_60d: 0.0,
            relative_return_vs_sector: 0.0,
            relative_return_vs_spy: 0.0,
            relative_volume: 1.0,
            avg_dollar_volume: 1_000_000.0,
            trend_state: "fixture".to_string(),
            catalyst_status: scoring::CATALYST_PENDING_SOURCE.to_string(),
            components_json: "{}".to_string(),
            explanation: "fixture stock score".to_string(),
        })
        .collect()
}

fn symbol(ticker: &str, name: &str, asset_type: &str, sector: Option<&str>) -> Symbol {
    Symbol {
        symbol: ticker.to_string(),
        name: name.to_string(),
        asset_type: asset_type.to_string(),
        sector: sector.map(str::to_string),
        industry: None,
        exchange: EXCHANGE_US.to_string(),
        market_cap: None,
        is_active: true,
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
