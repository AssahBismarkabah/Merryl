use std::collections::HashMap;

use chrono::{Duration, NaiveDate};

use merryl::domain::models::{DailyPrice, SectorMap, Symbol};
use merryl::scoring::{apply_sector_rank_changes, latest_date, score_market};

#[test]
fn latest_date_uses_max_available_price_date() {
    let prices = vec![
        price("SPY", "2026-01-02", 100.0, 1000.0),
        price("SPY", "2026-01-03", 101.0, 1000.0),
    ];

    assert_eq!(latest_date(&prices).as_deref(), Some("2026-01-03"));
}

#[test]
fn daily_scoring_produces_sector_and_stock_rankings() {
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
    prices.extend(series("SPY", 100.0, 0.0010, 1_000_000.0));
    prices.extend(series("QQQ", 100.0, 0.0020, 1_000_000.0));
    prices.extend(series("IWM", 100.0, 0.0015, 1_000_000.0));
    prices.extend(series("DIA", 100.0, 0.0008, 1_000_000.0));
    prices.extend(series("XLK", 120.0, 0.0020, 1_000_000.0));
    prices.extend(series("XLF", 90.0, 0.0005, 1_000_000.0));
    prices.extend(series("MSFT", 200.0, 0.0030, 1_000_000.0));
    prices.extend(series("JPM", 150.0, 0.0008, 1_000_000.0));

    let mut scores = score_market("2026-03-11", &symbols, &prices, &sector_maps);

    assert_eq!(scores.sectors.len(), 2);
    assert!(!scores.industries.is_empty());
    assert!(scores.stocks.iter().any(|stock| stock.symbol == "MSFT"));
    assert_eq!(scores.sectors[0].rank, 1);
    assert_eq!(scores.stocks[0].rank, 1);
    assert!(!scores.regime.label.is_empty());

    let previous_ranks = HashMap::from([
        ("Technology".to_string(), 2usize),
        ("Financials".to_string(), 1usize),
    ]);
    apply_sector_rank_changes(&mut scores.sectors, &previous_ranks);

    let technology = scores
        .sectors
        .iter()
        .find(|sector| sector.sector == "Technology")
        .expect("Technology sector score");
    assert_eq!(technology.rank_change, 1.0);
}

fn series(symbol: &str, base: f64, daily_return: f64, volume: f64) -> Vec<DailyPrice> {
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid fixture date");
    let mut close = base;

    (0..70)
        .map(|idx| {
            close *= 1.0 + daily_return;
            let date = start + Duration::days(idx);
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
