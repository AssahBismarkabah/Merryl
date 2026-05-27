use std::collections::HashSet;

use crate::config::{APP_NAME, output_text, scoring};
use crate::domain::models::{IndustryScore, MarketRegimeScore, SectorScore, StockScore};

use super::formatting::{multiple, pct, score};

pub fn daily_report_markdown(
    date: &str,
    regime: &MarketRegimeScore,
    sector_scores: &[SectorScore],
    industry_scores: &[IndustryScore],
    stock_scores: &[StockScore],
    previous_watchlist_symbols: &HashSet<String>,
) -> String {
    [
        report_header(date),
        market_regime(regime),
        top_sector_table(sector_scores),
        weak_sector_table(sector_scores),
        sector_rank_changes(sector_scores),
        top_industry_table(industry_scores),
        watchlist_table(stock_scores),
        new_leaders(stock_scores, previous_watchlist_symbols),
        high_relative_volume_table(stock_scores),
        catalyst_flags(stock_scores),
        notes_for_chart_review(),
        explanation_list(stock_scores),
    ]
    .join("\n\n")
}

fn report_header(date: &str) -> String {
    format!(
        "# Market Report: {date}\n\nSystem: {APP_NAME} {}\n\n{}",
        output_text::DAILY_REPORT_TITLE,
        output_text::REPORT_RULE
    )
}

fn market_regime(regime: &MarketRegimeScore) -> String {
    [
        section_heading(output_text::MARKET_REGIME_SECTION),
        format!(
            "**{}** with regime score {}.",
            regime.label,
            score(regime.score)
        ),
        regime.explanation.clone(),
    ]
    .join("\n\n")
}

fn top_sector_table(sector_scores: &[SectorScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::TOP_SECTORS_SECTION),
        output_text::SECTOR_TABLE_HEADER.to_string(),
        output_text::SECTOR_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(
        sector_scores
            .iter()
            .take(scoring::TOP_SECTOR_REPORT_LIMIT)
            .map(sector_row),
    );
    rows.join("\n")
}

fn weak_sector_table(sector_scores: &[SectorScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::WEAK_SECTORS_SECTION),
        output_text::SECTOR_TABLE_HEADER.to_string(),
        output_text::SECTOR_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(
        sector_scores
            .iter()
            .rev()
            .take(scoring::WEAK_SECTOR_REPORT_LIMIT)
            .map(sector_row),
    );
    rows.join("\n")
}

fn sector_row(sector: &SectorScore) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.0}% | {:.0}% | {} |",
        sector.rank,
        sector.sector,
        sector.sector_etf,
        score(sector.score),
        pct(sector.return_1d),
        pct(sector.return_5d),
        pct(sector.return_20d),
        pct(sector.return_60d),
        pct(sector.relative_return_vs_spy),
        multiple(sector.relative_volume),
        sector.breadth_20d,
        sector.breadth_50d,
        rank_change(sector.rank_change),
    )
}

fn sector_rank_changes(sector_scores: &[SectorScore]) -> String {
    let mut changed: Vec<&SectorScore> = sector_scores
        .iter()
        .filter(|sector| sector.rank_change != 0.0)
        .collect();
    changed.sort_by(|a, b| b.rank_change.abs().total_cmp(&a.rank_change.abs()));

    if changed.is_empty() {
        return format!(
            "{}\n{}",
            section_heading(output_text::SECTOR_RANK_CHANGES_SECTION),
            output_text::NO_PRIOR_RANK_HISTORY
        );
    }

    let mut rows = vec![
        section_heading(output_text::SECTOR_RANK_CHANGES_SECTION),
        "| Sector | ETF | Current Rank | Rank Change | Score |".to_string(),
        "|---|---:|---:|---:|---:|".to_string(),
    ];
    rows.extend(changed.into_iter().map(|sector| {
        format!(
            "| {} | {} | {} | {} | {} |",
            sector.sector,
            sector.sector_etf,
            sector.rank,
            rank_change(sector.rank_change),
            score(sector.score)
        )
    }));
    rows.join("\n")
}

fn top_industry_table(industry_scores: &[IndustryScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::TOP_INDUSTRIES_SECTION),
        output_text::INDUSTRY_TABLE_HEADER.to_string(),
        output_text::INDUSTRY_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(
        industry_scores
            .iter()
            .take(scoring::TOP_INDUSTRY_REPORT_LIMIT)
            .map(|industry| {
                format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.0}% | {:.0}% | {:.0}% | {} |",
                    industry.rank,
                    industry.industry,
                    industry.sector,
                    score(industry.score),
                    pct(industry.return_5d),
                    pct(industry.return_20d),
                    pct(industry.return_60d),
                    pct(industry.relative_return_vs_sector),
                    pct(industry.relative_return_vs_spy),
                    multiple(industry.relative_volume),
                    industry.breadth_20d,
                    industry.breadth_50d,
                    industry.high_20d_rate,
                    industry.member_count
                )
            }),
    );
    rows.join("\n")
}

fn watchlist_table(stock_scores: &[StockScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::WATCHLIST_SECTION),
        output_text::WATCHLIST_TABLE_HEADER.to_string(),
        output_text::WATCHLIST_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(
        stock_scores
            .iter()
            .take(scoring::REPORT_WATCHLIST_LIMIT)
            .map(watchlist_row),
    );
    rows.join("\n")
}

fn watchlist_row(stock: &StockScore) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        stock.rank,
        stock.symbol,
        stock.name,
        stock.sector,
        stock.industry,
        score(stock.score),
        pct(stock.return_20d),
        pct(stock.relative_return_vs_sector),
        multiple(stock.relative_volume),
        stock.trend_state,
        stock.catalyst_status,
    )
}

fn new_leaders(
    stock_scores: &[StockScore],
    previous_watchlist_symbols: &HashSet<String>,
) -> String {
    if previous_watchlist_symbols.is_empty() {
        return format!(
            "{}\n{}",
            section_heading(output_text::NEW_LEADERS_SECTION),
            output_text::NO_PRIOR_WATCHLIST_HISTORY
        );
    }

    let new_names: Vec<&StockScore> = stock_scores
        .iter()
        .take(scoring::REPORT_WATCHLIST_LIMIT)
        .filter(|stock| !previous_watchlist_symbols.contains(&stock.symbol))
        .collect();

    if new_names.is_empty() {
        return format!(
            "{}\n{}",
            section_heading(output_text::NEW_LEADERS_SECTION),
            output_text::NO_NEW_LEADERS
        );
    }

    let mut rows = vec![
        section_heading(output_text::NEW_LEADERS_SECTION),
        output_text::WATCHLIST_TABLE_HEADER.to_string(),
        output_text::WATCHLIST_TABLE_ALIGNMENT.to_string(),
    ];
    rows.extend(new_names.into_iter().map(watchlist_row));
    rows.join("\n")
}

fn high_relative_volume_table(stock_scores: &[StockScore]) -> String {
    let mut high_volume: Vec<&StockScore> = stock_scores.iter().collect();
    high_volume.sort_by(|a, b| b.relative_volume.total_cmp(&a.relative_volume));

    let mut rows = vec![
        section_heading(output_text::HIGH_RELATIVE_VOLUME_SECTION),
        output_text::HIGH_RELATIVE_VOLUME_TABLE_HEADER.to_string(),
        output_text::HIGH_RELATIVE_VOLUME_TABLE_ALIGNMENT.to_string(),
    ];
    rows.extend(
        high_volume
            .into_iter()
            .take(scoring::HIGH_RELATIVE_VOLUME_REPORT_LIMIT)
            .map(|stock| {
                format!(
                    "| {} | {} | {} | {} | {} | {} | {} |",
                    stock.rank,
                    stock.symbol,
                    stock.sector,
                    score(stock.score),
                    multiple(stock.relative_volume),
                    pct(stock.return_20d),
                    pct(stock.relative_return_vs_sector)
                )
            }),
    );
    rows.join("\n")
}

fn catalyst_flags(stock_scores: &[StockScore]) -> String {
    let flagged: Vec<&StockScore> = stock_scores
        .iter()
        .filter(|stock| stock.catalyst_status != scoring::CATALYST_PENDING_SOURCE)
        .collect();

    if flagged.is_empty() {
        return format!(
            "{}\n{}",
            section_heading(output_text::CATALYST_SECTION),
            output_text::CATALYST_PENDING_NOTE
        );
    }

    let mut rows = vec![
        section_heading(output_text::CATALYST_SECTION),
        "| Symbol | Catalyst / Earnings Status |".to_string(),
        "|---|---|".to_string(),
    ];
    rows.extend(
        flagged
            .into_iter()
            .map(|stock| format!("| {} | {} |", stock.symbol, stock.catalyst_status)),
    );
    rows.join("\n")
}

fn notes_for_chart_review() -> String {
    let mut rows = vec![section_heading(output_text::NOTES_SECTION)];
    rows.extend(
        output_text::CHART_REVIEW_NOTES
            .iter()
            .map(|note| format!("- {note}")),
    );
    rows.join("\n")
}

fn explanation_list(stock_scores: &[StockScore]) -> String {
    let mut rows = vec![section_heading(output_text::EXPLANATION_SECTION)];
    rows.extend(
        stock_scores
            .iter()
            .take(scoring::EXPLANATION_LIMIT)
            .map(|stock| format!("- **{}**: {}", stock.symbol, stock.explanation)),
    );
    rows.join("\n")
}

fn section_heading(title: &str) -> String {
    format!("## {title}")
}

fn rank_change(value: f64) -> String {
    if value > 0.0 {
        format!("+{value:.0}")
    } else {
        format!("{value:.0}")
    }
}
