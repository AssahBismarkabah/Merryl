use std::fs;

use anyhow::{Context, Result};

use crate::config::paths;

use super::csv::{write_sector_csv, write_watchlist_csv};
use super::markdown::{DailyReportInput, daily_report_markdown};
use super::paths::ReportPaths;

pub fn write_daily_outputs(input: DailyReportInput<'_>) -> Result<ReportPaths> {
    fs::create_dir_all(paths::REPORTS_DIR).context("failed to create reports directory")?;
    fs::create_dir_all(paths::EXPORTS_DIR).context("failed to create exports directory")?;

    let paths = ReportPaths::for_date(input.date);
    fs::write(&paths.report, daily_report_markdown(&input))
        .with_context(|| format!("failed to write {}", paths.report.display()))?;
    write_sector_csv(&paths.sector_export, input.sector_scores)?;
    write_watchlist_csv(&paths.watchlist_export, input.stock_scores)?;

    Ok(paths)
}
