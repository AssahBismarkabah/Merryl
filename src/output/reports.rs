use std::collections::HashSet;
use std::fs;

use anyhow::{Context, Result};

use crate::config::paths;
use crate::domain::models::{
    IndustryScore, MarketEvent, MarketRegimeScore, SectorScore, StockScore,
};

use super::csv::{write_sector_csv, write_watchlist_csv};
use super::markdown::daily_report_markdown;
use super::paths::ReportPaths;

pub fn write_daily_outputs(
    date: &str,
    regime: &MarketRegimeScore,
    sector_scores: &[SectorScore],
    industry_scores: &[IndustryScore],
    stock_scores: &[StockScore],
    events: &[MarketEvent],
    previous_watchlist_symbols: &HashSet<String>,
) -> Result<ReportPaths> {
    fs::create_dir_all(paths::REPORTS_DIR).context("failed to create reports directory")?;
    fs::create_dir_all(paths::EXPORTS_DIR).context("failed to create exports directory")?;

    let paths = ReportPaths::for_date(date);
    fs::write(
        &paths.report,
        daily_report_markdown(
            date,
            regime,
            sector_scores,
            industry_scores,
            stock_scores,
            events,
            previous_watchlist_symbols,
        ),
    )
    .with_context(|| format!("failed to write {}", paths.report.display()))?;
    write_sector_csv(&paths.sector_export, sector_scores)?;
    write_watchlist_csv(&paths.watchlist_export, stock_scores)?;

    Ok(paths)
}
