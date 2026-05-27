use anyhow::Result;
use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use tempfile::tempdir;

use merryl::domain::models::{DailyPrice, SectorMap, Symbol};
use merryl::scoring::{score_market, score_market_history};
use merryl::storage::Database;

#[test]
fn historical_scoring_scores_multiple_dates_and_stores_rows() -> Result<()> {
    let (symbols, sector_maps, prices) = fixture_market();
    let history = score_market_history("2026-05-10", &symbols, &prices, &sector_maps);

    assert!(history.len() > 20);
    assert!(history.iter().skip(1).any(|scores| {
        scores
            .sectors
            .iter()
            .any(|sector| sector.rank_change != 0.0)
    }));

    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    let mut db = Database::open(&db_path)?;
    db.migrate()?;
    db.upsert_symbols(&symbols)?;
    db.upsert_sector_maps(&sector_maps)?;
    db.upsert_prices(&prices)?;

    for scores in &history {
        db.replace_market_regime(&scores.regime)?;
        db.replace_sector_scores(&scores.date, &scores.sectors)?;
        db.replace_industry_scores(&scores.date, &scores.industries)?;
        db.replace_stock_scores(&scores.date, &scores.stocks)?;
        db.replace_watchlist(&scores.date, &scores.stocks)?;
    }

    let conn = Connection::open(db_path)?;
    let score_dates: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT date) FROM sector_scores",
        [],
        |row| row.get(0),
    )?;
    let regime_rows: i64 =
        conn.query_row("SELECT COUNT(*) FROM market_regime_scores", [], |row| {
            row.get(0)
        })?;
    let stock_components: String = conn.query_row(
        "SELECT components_json FROM stock_scores LIMIT 1",
        [],
        |row| row.get(0),
    )?;
    let industry_components: String = conn.query_row(
        "SELECT components_json FROM industry_scores LIMIT 1",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(score_dates as usize, history.len());
    assert_eq!(regime_rows as usize, history.len());
    assert!(stock_components.contains("relative_strength_component"));
    assert!(industry_components.contains("relative_return_vs_sector"));
    assert!(industry_components.contains("breadth_20d"));

    Ok(())
}

#[test]
fn scoring_a_past_date_does_not_use_future_prices() {
    let (symbols, sector_maps, prices) = fixture_market();
    let target_date = "2026-03-25";
    let with_future = score_market_history(target_date, &symbols, &prices, &sector_maps);
    let cutoff_prices: Vec<DailyPrice> = prices
        .iter()
        .filter(|price| price.date.as_str() <= target_date)
        .cloned()
        .collect();
    let direct = score_market(target_date, &symbols, &cutoff_prices, &sector_maps);
    let historical = with_future
        .last()
        .expect("historical score for target date");

    assert_eq!(historical.date, target_date);
    assert_eq!(historical.sectors[0].sector, direct.sectors[0].sector);
    assert!((historical.sectors[0].return_20d - direct.sectors[0].return_20d).abs() < f64::EPSILON);
    assert_eq!(
        historical.industries[0].industry,
        direct.industries[0].industry
    );
    assert!((historical.industries[0].score - direct.industries[0].score).abs() < f64::EPSILON);
    assert_eq!(historical.stocks[0].symbol, direct.stocks[0].symbol);
}

fn fixture_market() -> (Vec<Symbol>, Vec<SectorMap>, Vec<DailyPrice>) {
    let symbols = vec![
        symbol("SPY", "SPDR S&P 500 ETF Trust", "broad_etf", None, None),
        symbol("QQQ", "Invesco QQQ Trust", "broad_etf", None, None),
        symbol("IWM", "iShares Russell 2000 ETF", "broad_etf", None, None),
        symbol(
            "DIA",
            "SPDR Dow Jones Industrial Average ETF",
            "broad_etf",
            None,
            None,
        ),
        symbol(
            "XLK",
            "Technology Select Sector SPDR",
            "sector_etf",
            Some("Technology"),
            None,
        ),
        symbol(
            "XLF",
            "Financial Select Sector SPDR",
            "sector_etf",
            Some("Financials"),
            None,
        ),
        symbol(
            "MSFT",
            "Microsoft Corporation",
            "stock",
            Some("Technology"),
            Some("Software"),
        ),
        symbol(
            "JPM",
            "JPMorgan Chase & Co.",
            "stock",
            Some("Financials"),
            Some("Banks"),
        ),
    ];
    let sector_maps = vec![
        sector_map("Technology", "XLK"),
        sector_map("Financials", "XLF"),
    ];
    let mut prices = Vec::new();
    prices.extend(series("SPY", 100.0, 0.0010, 0.0010, 80, 1_000_000.0));
    prices.extend(series("QQQ", 100.0, 0.0010, 0.0020, 80, 1_000_000.0));
    prices.extend(series("IWM", 100.0, 0.0010, 0.0015, 80, 1_000_000.0));
    prices.extend(series("DIA", 100.0, 0.0010, 0.0008, 80, 1_000_000.0));
    prices.extend(series("XLK", 120.0, 0.0002, 0.0045, 80, 1_000_000.0));
    prices.extend(series("XLF", 90.0, 0.0035, 0.0001, 80, 1_000_000.0));
    prices.extend(series("MSFT", 200.0, 0.0002, 0.0050, 80, 1_000_000.0));
    prices.extend(series("JPM", 150.0, 0.0040, 0.0001, 80, 1_000_000.0));

    (symbols, sector_maps, prices)
}

fn series(
    symbol: &str,
    base: f64,
    first_daily_return: f64,
    second_daily_return: f64,
    switch_idx: usize,
    volume: f64,
) -> Vec<DailyPrice> {
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid fixture date");
    let mut close = base;

    (0..130)
        .map(|idx| {
            let daily_return = if idx < switch_idx {
                first_daily_return
            } else {
                second_daily_return
            };
            close *= 1.0 + daily_return;
            let date = start + Duration::days(idx as i64);
            price(symbol, &date.format("%Y-%m-%d").to_string(), close, volume)
        })
        .collect()
}

fn price(symbol: &str, date: &str, close: f64, volume: f64) -> DailyPrice {
    DailyPrice {
        symbol: symbol.to_string(),
        date: date.to_string(),
        open: close * 0.99,
        high: close * 1.01,
        low: close * 0.98,
        close,
        adjusted_close: close,
        volume,
        source: "test-fixture".to_string(),
    }
}

fn symbol(
    ticker: &str,
    name: &str,
    asset_type: &str,
    sector: Option<&str>,
    industry: Option<&str>,
) -> Symbol {
    Symbol {
        symbol: ticker.to_string(),
        name: name.to_string(),
        asset_type: asset_type.to_string(),
        sector: sector.map(str::to_string),
        industry: industry.map(str::to_string),
        exchange: "US".to_string(),
        market_cap: None,
        is_active: true,
    }
}

fn sector_map(sector: &str, etf: &str) -> SectorMap {
    SectorMap {
        sector: sector.to_string(),
        sector_etf: etf.to_string(),
        description: format!("{sector} test proxy"),
    }
}
