use crate::config::{APP_NAME, output_text, scoring};
use crate::domain::models::{SectorScore, StockScore};

use super::formatting::{multiple, pct, score};

pub fn daily_report_markdown(
    date: &str,
    sector_scores: &[SectorScore],
    stock_scores: &[StockScore],
) -> String {
    [
        report_header(date),
        sector_table(sector_scores),
        watchlist_table(stock_scores),
        explanation_list(stock_scores),
    ]
    .join("\n\n")
}

fn report_header(date: &str) -> String {
    format!(
        "# {APP_NAME} {}\n\nDate: `{date}`\n\n{}",
        output_text::DAILY_REPORT_TITLE,
        output_text::REPORT_RULE
    )
}

fn sector_table(sector_scores: &[SectorScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::SECTOR_SECTION),
        output_text::SECTOR_TABLE_HEADER.to_string(),
        output_text::SECTOR_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(sector_scores.iter().map(sector_row));
    rows.join("\n")
}

fn sector_row(sector: &SectorScore) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.0}% | {:.0}% |",
        sector.rank,
        sector.sector,
        sector.sector_etf,
        score(sector.score),
        pct(sector.return_1d),
        pct(sector.return_5d),
        pct(sector.return_20d),
        pct(sector.return_60d),
        multiple(sector.relative_volume),
        sector.breadth_20d,
        sector.breadth_50d,
    )
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
