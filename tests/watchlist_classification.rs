use std::collections::HashSet;

use merryl::classification::WatchlistClassifier;
use merryl::config::classification;
use merryl::domain::models::{IndustryScore, SectorScore, StockScore};
use merryl::validation::MacroContextOverlay;

#[test]
fn watchlist_classification_labels_all_current_contexts() {
    let sectors = vec![sector("Technology", 1)];
    let industries = vec![industry("Semiconductors", "Technology", 1)];
    let previous = HashSet::from(["AMD".to_string()]);
    let macro_context = MacroContextOverlay {
        date: "2026-05-28".to_string(),
        active_flags: vec!["liquidity_tightening".to_string()],
        stale_series: Vec::new(),
        covered_series_count: 11,
        required_series_count: 11,
        interpretation: "ETF-proxy regime is risk-on while macro stress flags are active."
            .to_string(),
    };
    let classifier = WatchlistClassifier::new(
        &sectors,
        &industries,
        &previous,
        "Risk-on",
        Some(&macro_context),
    );

    let labels = classifier.labels_for(&stock("NVDA"));

    assert_eq!(
        labels,
        vec![
            classification::LABEL_SECTOR_LEADER,
            classification::LABEL_INDUSTRY_LEADER,
            classification::LABEL_RELATIVE_STRENGTH_LEADER,
            classification::LABEL_VOLUME_CONFIRMED,
            classification::LABEL_NEW_LEADER,
            classification::LABEL_EVENT_CONTEXT,
            classification::LABEL_EVENT_RISK,
            classification::LABEL_MACRO_CONFLICT_CONTEXT,
        ]
    );
}

#[test]
fn watchlist_classification_omits_unproven_contexts() {
    let sectors = vec![sector("Utilities", 7)];
    let industries = vec![industry("Electric Utilities", "Utilities", 17)];
    let previous = HashSet::from(["NEE".to_string()]);
    let classifier = WatchlistClassifier::new(&sectors, &industries, &previous, "Mixed", None);
    let mut stock = stock("NEE");
    stock.sector = "Utilities".to_string();
    stock.industry = "Electric Utilities".to_string();
    stock.relative_return_vs_sector = -0.01;
    stock.relative_return_vs_spy = -0.02;
    stock.relative_volume = 0.9;
    stock.catalyst_status = "pending_source".to_string();

    let labels = classifier.labels_for(&stock);

    assert!(labels.is_empty());
}

fn sector(name: &str, rank: usize) -> SectorScore {
    SectorScore {
        date: "2026-05-28".to_string(),
        sector: name.to_string(),
        sector_etf: "XLK".to_string(),
        score: 88.0,
        rank,
        return_1d: 0.0,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        breadth_20d: 0.0,
        breadth_50d: 0.0,
        rank_change: 0.0,
        explanation: "fixture sector".to_string(),
    }
}

fn industry(name: &str, sector: &str, rank: usize) -> IndustryScore {
    IndustryScore {
        date: "2026-05-28".to_string(),
        industry: name.to_string(),
        sector: sector.to_string(),
        score: 82.0,
        rank,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_sector: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        breadth_20d: 0.0,
        breadth_50d: 0.0,
        high_20d_rate: 0.0,
        member_count: 1,
        components_json: "{}".to_string(),
    }
}

fn stock(symbol: &str) -> StockScore {
    StockScore {
        date: "2026-05-28".to_string(),
        rank: 1,
        symbol: symbol.to_string(),
        name: symbol.to_string(),
        sector: "Technology".to_string(),
        industry: "Semiconductors".to_string(),
        score: 91.0,
        sector_score: 88.0,
        return_1d: 0.01,
        return_5d: 0.04,
        return_20d: 0.08,
        return_60d: 0.16,
        relative_return_vs_sector: 0.04,
        relative_return_vs_spy: 0.06,
        relative_volume: 1.5,
        avg_dollar_volume: 50_000_000.0,
        trend_state: "above_20d_50d".to_string(),
        catalyst_status: "recent_news:2 | earnings:2026-06-01 | filing:8-K".to_string(),
        components_json: "{}".to_string(),
        explanation: "fixture stock".to_string(),
    }
}
