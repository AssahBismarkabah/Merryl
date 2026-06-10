use anyhow::Result;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tempfile::tempdir;
use tower::ServiceExt;

use merryl::config::macro_data;
use merryl::dashboard::{load_dashboard_for_date, load_latest_dashboard, router};
use merryl::domain::models::{
    IndustryScore, IntradaySetup, IntradayTrigger, MacroObservation, MarketRegimeScore,
    SectorScore, StockScore, Symbol, VolumeProfile,
};
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
    let macro_context = regime.macro_context.expect("macro context");
    assert_eq!(macro_context.date, "2026-05-27");
    assert_eq!(
        macro_context.active_flags,
        vec!["rate_pressure".to_string()]
    );
    assert_eq!(
        macro_context.covered_series_count,
        macro_data::MACRO_SERIES.len()
    );
    assert_eq!(snapshot.sectors.len(), 1);
    assert_eq!(snapshot.industries[0].industry, "Software");
    assert_eq!(snapshot.stocks[0].symbol, "MSFT");
    assert_eq!(snapshot.watchlist[0].name, "Microsoft Corporation");
    assert!(
        snapshot.watchlist[0]
            .classifications
            .contains(&"sector_leader".to_string())
    );
    assert!(
        snapshot.watchlist[0]
            .classifications
            .contains(&"macro_conflict_context".to_string())
    );
    assert_eq!(
        snapshot.watchlist[0].primary_actionability,
        "base_compression_candidate"
    );
    assert!(
        snapshot.watchlist[0]
            .actionability_labels
            .contains(&"base_compression_candidate".to_string())
    );
    assert_eq!(snapshot.stocks[0].distance_from_20d_ma_pct, 0.03);
    assert_eq!(snapshot.intraday_setups[0].symbol, "MSFT");
    assert_eq!(
        snapshot.intraday_setups[0].primary_label,
        "intraday_execution_ready"
    );
    assert_eq!(snapshot.intraday_triggers[0].trigger_type, "orb_breakout");
    assert!(snapshot.latest_backtest.is_none());
    assert_eq!(
        snapshot.data_health.latest_score_date.as_deref(),
        Some("2026-05-27")
    );

    Ok(())
}

#[test]
fn dashboard_snapshot_reads_selected_score_date() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    seed_dashboard_fixture(&db_path)?;

    let snapshot = load_dashboard_for_date(&db_path, "2026-05-20")?;

    assert_eq!(snapshot.score_date, "2026-05-20");
    assert_eq!(snapshot.regime.expect("regime").date, "2026-05-20");
    assert_eq!(snapshot.sectors[0].date, "2026-05-20");
    assert_eq!(snapshot.sectors[0].sector, "Health Care");
    assert_eq!(snapshot.industries[0].date, "2026-05-20");
    assert_eq!(snapshot.industries[0].industry, "Biotechnology");
    assert_eq!(snapshot.stocks[0].date, "2026-05-20");
    assert_eq!(snapshot.stocks[0].symbol, "AMGN");
    assert_eq!(snapshot.watchlist[0].date, "2026-05-20");
    assert_eq!(snapshot.watchlist[0].symbol, "AMGN");

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
    assert_eq!(json["regime"]["macro_context"]["date"], "2026-05-27");
    assert_eq!(
        json["regime"]["macro_context"]["active_flags"][0],
        "rate_pressure"
    );
    assert_eq!(json["stocks"][0]["symbol"], "MSFT");
    assert_eq!(
        json["stocks"][0]["primary_actionability"],
        "base_compression_candidate"
    );
    assert_eq!(json["watchlist"][0]["classifications"][0], "sector_leader");
    assert_eq!(
        json["watchlist"][0]["primary_actionability"],
        "base_compression_candidate"
    );
    assert_eq!(json["intraday_setups"][0]["symbol"], "MSFT");
    assert_eq!(
        json["intraday_setups"][0]["primary_label"],
        "intraday_execution_ready"
    );
    assert_eq!(json["intraday_triggers"][0]["trigger_type"], "orb_breakout");

    Ok(())
}

#[tokio::test]
async fn dashboard_api_returns_scored_dates_descending() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    seed_dashboard_fixture(&db_path)?;
    let app = router(db_path, dir.path().join("dist").as_path());

    let response = app
        .oneshot(Request::builder().uri("/api/dates").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;

    assert_eq!(json["dates"][0], "2026-05-27");
    assert_eq!(json["dates"][1], "2026-05-20");

    Ok(())
}

#[tokio::test]
async fn dashboard_api_returns_selected_date_snapshot_json() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("market.db");
    seed_dashboard_fixture(&db_path)?;
    let app = router(db_path, dir.path().join("dist").as_path());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/dashboard/2026-05-20")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;

    assert_eq!(json["score_date"], "2026-05-20");
    assert_eq!(json["regime"]["date"], "2026-05-20");
    assert_eq!(json["sectors"][0]["date"], "2026-05-20");
    assert_eq!(json["sectors"][0]["sector"], "Health Care");
    assert_eq!(json["industries"][0]["date"], "2026-05-20");
    assert_eq!(json["industries"][0]["industry"], "Biotechnology");
    assert_eq!(json["stocks"][0]["date"], "2026-05-20");
    assert_eq!(json["stocks"][0]["symbol"], "AMGN");
    assert_eq!(json["watchlist"][0]["date"], "2026-05-20");
    assert_eq!(json["watchlist"][0]["symbol"], "AMGN");

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
    db.upsert_symbols(&[
        symbol("SPY", "SPDR S&P 500 ETF Trust", "broad_etf"),
        symbol("MSFT", "Microsoft Corporation", "stock"),
        symbol("AMGN", "Amgen Inc.", "stock"),
    ])?;
    db.upsert_macro_observations(&macro_observations())?;

    for date in ["2026-05-20", "2026-05-27"] {
        db.replace_market_regime(&market_regime_score(date))?;
        db.replace_sector_scores(date, &[sector_score(date)])?;
        db.replace_industry_scores(date, &[industry_score(date)])?;
        db.replace_stock_scores(date, &[stock_score(date)])?;
        db.replace_watchlist(date, &[stock_score(date)])?;
    }
    db.replace_intraday_readiness(
        "2026-05-27",
        &[volume_profile()],
        &[intraday_setup()],
        &[intraday_trigger()],
    )?;

    Ok(())
}

fn macro_observations() -> Vec<MacroObservation> {
    [
        observations_for_date(
            "2026-05-20",
            &[
                ("VIXCLS", 15.0),
                ("DGS10", 4.0),
                ("DGS2", 3.5),
                ("T10Y2Y", 0.5),
                ("DFF", 4.0),
                ("CPIAUCSL", 300.0),
                ("UNRATE", 4.0),
                ("PAYEMS", 100.0),
                ("BAMLC0A0CM", 1.0),
                ("DTWEXBGS", 100.0),
                ("WALCL", 7000.0),
            ],
        ),
        observations_for_date(
            "2026-05-27",
            &[
                ("VIXCLS", 16.0),
                ("DGS10", 4.5),
                ("DGS2", 3.4),
                ("T10Y2Y", 0.4),
                ("DFF", 4.0),
                ("CPIAUCSL", 300.0),
                ("UNRATE", 4.0),
                ("PAYEMS", 100.0),
                ("BAMLC0A0CM", 1.0),
                ("DTWEXBGS", 100.0),
                ("WALCL", 7000.0),
            ],
        ),
    ]
    .concat()
}

fn observations_for_date(date: &str, values: &[(&str, f64)]) -> Vec<MacroObservation> {
    values
        .iter()
        .map(|(series, value)| macro_observation(series, date, *value))
        .collect()
}

fn macro_observation(series: &str, date: &str, value: f64) -> MacroObservation {
    let (_, series_name, frequency, units) = macro_data::MACRO_SERIES
        .iter()
        .find(|(candidate, _, _, _)| *candidate == series)
        .expect("known macro series");
    MacroObservation {
        series: series.to_string(),
        series_name: (*series_name).to_string(),
        date: date.to_string(),
        value,
        source: format!("fred:{series}"),
        frequency: (*frequency).to_string(),
        units: (*units).to_string(),
        realtime_start: date.to_string(),
        realtime_end: date.to_string(),
        raw_json: "{}".to_string(),
        quality_status: "ok".to_string(),
    }
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
    let latest = date == "2026-05-27";
    MarketRegimeScore {
        date: date.to_string(),
        label: if latest { "risk_on" } else { "mixed" }.to_string(),
        score: if latest { 72.0 } else { 51.0 },
        spy_return_20d: if latest { 0.04 } else { 0.01 },
        spy_return_60d: if latest { 0.08 } else { 0.02 },
        qqq_relative_return_vs_spy: if latest { 0.02 } else { -0.01 },
        iwm_relative_return_vs_spy: if latest { 0.01 } else { -0.02 },
        dia_relative_return_vs_spy: 0.0,
        components_json: r#"{"tlt_return_20d":0.01,"gld_return_20d":0.02,"uso_return_20d":-0.03}"#
            .to_string(),
        explanation: "fixture regime".to_string(),
    }
}

fn sector_score(date: &str) -> SectorScore {
    let latest = date == "2026-05-27";
    SectorScore {
        date: date.to_string(),
        sector: if latest { "Technology" } else { "Health Care" }.to_string(),
        sector_etf: if latest { "XLK" } else { "XLV" }.to_string(),
        score: if latest { 88.0 } else { 77.0 },
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
    let latest = date == "2026-05-27";
    IndustryScore {
        date: date.to_string(),
        industry: if latest { "Software" } else { "Biotechnology" }.to_string(),
        sector: if latest { "Technology" } else { "Health Care" }.to_string(),
        score: if latest { 82.0 } else { 79.0 },
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
    let latest = date == "2026-05-27";
    StockScore {
        date: date.to_string(),
        rank: 1,
        symbol: if latest { "MSFT" } else { "AMGN" }.to_string(),
        name: if latest {
            "Microsoft Corporation"
        } else {
            "Amgen Inc."
        }
        .to_string(),
        sector: if latest { "Technology" } else { "Health Care" }.to_string(),
        industry: if latest { "Software" } else { "Biotechnology" }.to_string(),
        score: if latest { 91.0 } else { 83.0 },
        sector_score: if latest { 88.0 } else { 77.0 },
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
        components_json: r#"{"relative_strength_component":80.0,"ma_20d":100.0,"ma_50d":98.0,"distance_from_20d_ma_pct":0.03,"distance_from_50d_ma_pct":0.05,"atr_14d":2.0,"atr_14d_pct":0.02,"atr_extension_from_20d_ma":1.5,"atr_extension_from_50d_ma":2.4,"high_20d":104.0,"high_60d":106.0,"distance_from_20d_high_pct":-0.01,"distance_from_60d_high_pct":-0.04,"range_10d_pct":0.04,"gap_pct":0.0,"true_range_pct":0.02,"primary_actionability":"base_compression_candidate","actionability_labels":["base_compression_candidate","early_rotation_candidate","actionable_leader"]}"#.to_string(),
        explanation: "fixture stock".to_string(),
    }
}

fn volume_profile() -> VolumeProfile {
    VolumeProfile {
        symbol: "MSFT".to_string(),
        date: "2026-05-27".to_string(),
        timeframe: "30Min".to_string(),
        poc: 100.0,
        vah: 101.0,
        val: 99.0,
        vwap: 100.2,
        high: 102.0,
        low: 98.0,
        total_volume: 1_000_000.0,
        source: "test-fixture".to_string(),
        components_json: "{}".to_string(),
    }
}

fn intraday_setup() -> IntradaySetup {
    IntradaySetup {
        date: "2026-05-27".to_string(),
        symbol: "MSFT".to_string(),
        name: "Microsoft Corporation".to_string(),
        sector: "Technology".to_string(),
        industry: "Software".to_string(),
        direction: "long".to_string(),
        stage1_passed: true,
        stage2_passed: true,
        stage3_passed: true,
        primary_label: "intraday_execution_ready".to_string(),
        adr_pct: 0.05,
        rvol_ratio: 1.8,
        mansfield_rs_spy: 1.08,
        mansfield_rs_sector: 1.03,
        ema_10: 100.0,
        ema_20: 99.0,
        latest_price: 100.5,
        confluence_count: 3,
        confluence_json: r#"["poc","val","vwap"]"#.to_string(),
        trigger_count: 1,
        components_json: "{}".to_string(),
    }
}

fn intraday_trigger() -> IntradayTrigger {
    IntradayTrigger {
        date: "2026-05-27".to_string(),
        symbol: "MSFT".to_string(),
        ts: "2026-05-27T15:00:00Z".to_string(),
        timeframe: "5Min".to_string(),
        trigger_type: "orb_breakout".to_string(),
        direction: "long".to_string(),
        trigger_price: 101.0,
        reference_level: 100.0,
        volume_spike: 2.0,
        price_action: "fixture trigger".to_string(),
        components_json: "{}".to_string(),
        source: "test-fixture".to_string(),
    }
}
