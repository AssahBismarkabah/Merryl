use anyhow::Result;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tempfile::tempdir;
use tower::ServiceExt;

use merryl::dashboard::{load_latest_dashboard, router};
use merryl::domain::models::{IndustryScore, MarketRegimeScore, SectorScore, StockScore, Symbol};
use merryl::storage::Database;

#[test]
fn dashboard_snapshot_reads_latest_market_map() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    seed_dashboard_fixture(&db_path)?;

    let snapshot = load_latest_dashboard(&db_path)?;

    assert_eq!(snapshot.score_date, "2026-05-27");
    let regime = snapshot.regime.expect("regime");
    assert_eq!(regime.label, "risk_on");
    assert_eq!(regime.tlt_return_20d, 0.01);
    assert_eq!(regime.gld_return_20d, 0.02);
    assert_eq!(regime.uso_return_20d, -0.03);
    assert_eq!(snapshot.sectors.len(), 1);
    assert_eq!(snapshot.industries[0].industry, "Software");
    assert_eq!(snapshot.stocks[0].symbol, "MSFT");
    assert_eq!(snapshot.watchlist[0].name, "Microsoft Corporation");
    assert!(snapshot.latest_backtest.is_none());
    assert_eq!(
        snapshot.data_health.latest_score_date.as_deref(),
        Some("2026-05-27")
    );

    Ok(())
}

#[tokio::test]
async fn dashboard_api_returns_latest_snapshot_json() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    seed_dashboard_fixture(&db_path)?;
    let app = router(db_path, dir.path().join("dist").as_path());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/dashboard/latest")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;

    assert_eq!(json["score_date"], "2026-05-27");
    assert_eq!(json["regime"]["label"], "risk_on");
    assert_eq!(json["regime"]["tlt_return_20d"], 0.01);
    assert_eq!(json["regime"]["gld_return_20d"], 0.02);
    assert_eq!(json["regime"]["uso_return_20d"], -0.03);
    assert_eq!(json["stocks"][0]["symbol"], "MSFT");

    Ok(())
}

#[tokio::test]
async fn dashboard_api_reports_missing_data_directly() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("missing.db");
    let app = router(db_path, dir.path().join("dist").as_path());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/dashboard/latest")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;

    assert!(
        json["message"]
            .as_str()
            .expect("message")
            .contains("run `merryl run daily --date latest` first")
    );

    Ok(())
}

fn seed_dashboard_fixture(db_path: &std::path::Path) -> Result<()> {
    let mut db = Database::open(db_path)?;
    db.migrate()?;
    let date = "2026-05-27";

    db.upsert_symbols(&[symbol("SPY", "SPDR S&P 500 ETF Trust", "broad_etf")])?;
    db.replace_market_regime(&market_regime_score(date))?;
    db.replace_sector_scores(date, &[sector_score(date)])?;
    db.replace_industry_scores(date, &[industry_score(date)])?;
    db.replace_stock_scores(date, &[stock_score(date)])?;
    db.replace_watchlist(date, &[stock_score(date)])?;

    Ok(())
}

fn symbol(ticker: &str, name: &str, asset_type: &str) -> Symbol {
    Symbol {
        symbol: ticker.to_string(),
        name: name.to_string(),
        asset_type: asset_type.to_string(),
        sector: None,
        industry: None,
        exchange: "US".to_string(),
        market_cap: None,
        is_active: true,
    }
}

fn market_regime_score(date: &str) -> MarketRegimeScore {
    MarketRegimeScore {
        date: date.to_string(),
        label: "risk_on".to_string(),
        score: 72.0,
        spy_return_20d: 0.04,
        spy_return_60d: 0.08,
        qqq_relative_return_vs_spy: 0.02,
        iwm_relative_return_vs_spy: 0.01,
        dia_relative_return_vs_spy: 0.0,
        components_json: r#"{"tlt_return_20d":0.01,"gld_return_20d":0.02,"uso_return_20d":-0.03}"#
            .to_string(),
        explanation: "fixture regime".to_string(),
    }
}

fn sector_score(date: &str) -> SectorScore {
    SectorScore {
        date: date.to_string(),
        sector: "Technology".to_string(),
        sector_etf: "XLK".to_string(),
        score: 88.0,
        rank: 1,
        return_1d: 0.01,
        return_5d: 0.03,
        return_20d: 0.06,
        return_60d: 0.12,
        relative_return_vs_spy: 0.02,
        relative_volume: 1.4,
        breadth_20d: 0.7,
        breadth_50d: 0.6,
        rank_change: 1.0,
        explanation: "fixture sector".to_string(),
    }
}

fn industry_score(date: &str) -> IndustryScore {
    IndustryScore {
        date: date.to_string(),
        industry: "Software".to_string(),
        sector: "Technology".to_string(),
        score: 82.0,
        rank: 1,
        return_5d: 0.02,
        return_20d: 0.05,
        return_60d: 0.10,
        relative_return_vs_sector: 0.01,
        relative_return_vs_spy: 0.03,
        relative_volume: 1.2,
        breadth_20d: 0.6,
        breadth_50d: 0.5,
        high_20d_rate: 0.2,
        member_count: 1,
        components_json: r#"{"member_count":1,"return_20d":0.05}"#.to_string(),
    }
}

fn stock_score(date: &str) -> StockScore {
    StockScore {
        date: date.to_string(),
        rank: 1,
        symbol: "MSFT".to_string(),
        name: "Microsoft Corporation".to_string(),
        sector: "Technology".to_string(),
        industry: "Software".to_string(),
        score: 91.0,
        sector_score: 88.0,
        return_1d: 0.01,
        return_5d: 0.04,
        return_20d: 0.08,
        return_60d: 0.16,
        relative_return_vs_sector: 0.02,
        relative_return_vs_spy: 0.04,
        relative_volume: 1.5,
        avg_dollar_volume: 50_000_000.0,
        trend_state: "above_20d_50d".to_string(),
        catalyst_status: "recent_news:2".to_string(),
        components_json: r#"{"relative_strength_component":80.0}"#.to_string(),
        explanation: "fixture stock".to_string(),
    }
}
