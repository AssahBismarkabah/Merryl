use std::collections::HashSet;

use merryl::domain::models::{
    IndustryScore, MacroObservation, MarketEvent, MarketEventMetadata, MarketRegimeScore,
    SectorScore, StockScore,
};
use merryl::output::{DailyReportInput, daily_report_markdown};
use merryl::validation::MacroContextOverlay;

#[test]
fn daily_report_contains_documented_sections() {
    let regime = MarketRegimeScore {
        date: "2026-05-26".to_string(),
        label: "Risk-on".to_string(),
        score: 66.0,
        spy_return_20d: 0.04,
        spy_return_60d: 0.08,
        qqq_relative_return_vs_spy: 0.03,
        iwm_relative_return_vs_spy: 0.01,
        dia_relative_return_vs_spy: -0.01,
        components_json: "{}".to_string(),
        explanation: "Risk-on: broad ETF proxies are constructive.".to_string(),
    };
    let sectors = vec![
        sector("Technology", "XLK", 1, 82.0, 1.0),
        sector("Financials", "XLF", 2, 51.0, 0.0),
        sector("Utilities", "XLU", 3, 31.0, -1.0),
    ];
    let industries = vec![IndustryScore {
        date: "2026-05-26".to_string(),
        industry: "Semiconductors".to_string(),
        sector: "Technology".to_string(),
        score: 88.0,
        rank: 1,
        return_5d: 0.05,
        return_20d: 0.12,
        return_60d: 0.18,
        relative_return_vs_sector: 0.04,
        relative_return_vs_spy: 0.06,
        relative_volume: 1.8,
        breadth_20d: 80.0,
        breadth_50d: 70.0,
        high_20d_rate: 60.0,
        member_count: 4,
        components_json: "{}".to_string(),
    }];
    let stocks = vec![
        stock("NVDA", 1, 91.0, 2.2, "recent_news:1"),
        stock("AMD", 2, 84.0, 1.6, "pending_source"),
    ];
    let events = vec![
        MarketEvent {
            symbol: "NVDA".to_string(),
            sector: Some("Technology".to_string()),
            event_date: "2026-05-26".to_string(),
            event_type: "news".to_string(),
            headline: "NVDA announces new AI platform".to_string(),
            source: "alpaca_news:benzinga".to_string(),
            url: Some("https://example.com/nvda".to_string()),
            metadata: MarketEventMetadata::default(),
        },
        MarketEvent {
            symbol: "NVDA".to_string(),
            sector: Some("Technology".to_string()),
            event_date: "2026-06-12".to_string(),
            event_type: "earnings".to_string(),
            headline: "Expected earnings for NVIDIA Corporation".to_string(),
            source: "alpha_vantage:earnings_calendar".to_string(),
            url: None,
            metadata: MarketEventMetadata {
                estimate: Some(5.25),
                ..MarketEventMetadata::default()
            },
        },
        MarketEvent {
            symbol: "NVDA".to_string(),
            sector: Some("Technology".to_string()),
            event_date: "2026-05-25".to_string(),
            event_type: "filing".to_string(),
            headline: "8-K filed by NVIDIA Corporation".to_string(),
            source: "sec_edgar:submissions".to_string(),
            url: Some("https://www.sec.gov/Archives/test".to_string()),
            metadata: MarketEventMetadata::default(),
        },
    ];
    let previous_watchlist = HashSet::from(["AMD".to_string()]);
    let macro_observations = [macro_observation(
        "VIXCLS",
        "CBOE Volatility Index: VIX",
        "Daily",
    )];
    let macro_context = MacroContextOverlay {
        date: "2026-05-26".to_string(),
        active_flags: vec!["rate_pressure".to_string()],
        stale_series: Vec::new(),
        covered_series_count: 1,
        required_series_count: 11,
        interpretation: "ETF-proxy regime is risk-on while macro stress flags are active."
            .to_string(),
    };
    let report = daily_report_markdown(&DailyReportInput {
        date: "2026-05-26",
        regime: &regime,
        sector_scores: &sectors,
        industry_scores: &industries,
        stock_scores: &stocks,
        events: &events,
        macro_observations: &macro_observations,
        macro_context: Some(&macro_context),
        previous_watchlist_symbols: &previous_watchlist,
    });

    for section in [
        "## Market Regime",
        "## Macro Context Overlay",
        "## Macro Context Coverage",
        "## Top Sectors",
        "## Weak Sectors",
        "## Sector Rank Changes",
        "## Top Industries Or Themes",
        "## Top Stocks Worth Charting",
        "## New Leaders",
        "## High Relative Volume Names",
        "## Catalyst / Event Flags",
        "## Notes For Chart Review",
    ] {
        assert!(report.contains(section), "missing section {section}");
    }
    assert!(report.contains("## New Leaders"));
    assert!(report.contains("Market regime score: daily ETF price proxies"));
    assert!(report.contains("FRED macro context is stored separately"));
    assert!(report.contains("they are not scoring inputs yet.\n\n| Series | Name | Frequency | Latest | Observations | Status |"));
    assert!(report.contains("Macro flags are as-of context only"));
    assert!(report.contains("rate_pressure"));
    assert!(
        report
            .contains("| VIXCLS | CBOE Volatility Index: VIX | Daily | 2026-05-26 | 1 | stored |")
    );
    assert!(report.contains("Sector ranking is a market-map and attention layer."));
    assert!(report.contains("Event sources: Alpaca News, Alpha Vantage Earnings Calendar"));
    assert!(report.contains("- **NVDA** `recent_news:1`"));
    assert!(report.contains("NVDA announces new AI platform"));
    assert!(report.contains("Earnings calendar 2026-06-12 estimate 5.25."));
    assert!(report.contains("SEC filing 2026-05-25: 8-K filed by NVIDIA Corporation"));
    assert!(report.contains("| 1 | NVDA |"));
    assert!(report.contains("| 1 | Semiconductors | Technology | 88.0 | 5.00% | 12.00% |"));
}

fn sector(name: &str, etf: &str, rank: usize, score: f64, rank_change: f64) -> SectorScore {
    SectorScore {
        date: "2026-05-26".to_string(),
        sector: name.to_string(),
        sector_etf: etf.to_string(),
        score,
        rank,
        return_1d: 0.01,
        return_5d: 0.02,
        return_20d: 0.04,
        return_60d: 0.08,
        relative_return_vs_spy: 0.02,
        relative_volume: 1.2,
        breadth_20d: 70.0,
        breadth_50d: 65.0,
        rank_change,
        explanation: format!("{name} explanation"),
    }
}

fn stock(
    symbol: &str,
    rank: usize,
    score: f64,
    relative_volume: f64,
    catalyst_status: &str,
) -> StockScore {
    StockScore {
        date: "2026-05-26".to_string(),
        rank,
        symbol: symbol.to_string(),
        name: symbol.to_string(),
        sector: "Technology".to_string(),
        industry: "Semiconductors".to_string(),
        score,
        sector_score: 82.0,
        return_1d: 0.01,
        return_5d: 0.03,
        return_20d: 0.08,
        return_60d: 0.12,
        relative_return_vs_sector: 0.04,
        relative_return_vs_spy: 0.05,
        relative_volume,
        avg_dollar_volume: 100_000_000.0,
        trend_state: "above_20d_50d".to_string(),
        catalyst_status: catalyst_status.to_string(),
        components_json: "{}".to_string(),
        explanation: format!("{symbol} explanation"),
    }
}

fn macro_observation(series: &str, name: &str, frequency: &str) -> MacroObservation {
    MacroObservation {
        series: series.to_string(),
        series_name: name.to_string(),
        date: "2026-05-26".to_string(),
        value: 18.44,
        source: format!("fred:{series}"),
        frequency: frequency.to_string(),
        units: "Index".to_string(),
        realtime_start: "2026-05-26".to_string(),
        realtime_end: "2026-05-26".to_string(),
        raw_json: "{}".to_string(),
        quality_status: "ok".to_string(),
    }
}
