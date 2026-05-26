use std::path::PathBuf;

use crate::config::paths;

#[derive(Debug)]
pub struct ReportPaths {
    pub report: PathBuf,
    pub sector_export: PathBuf,
    pub watchlist_export: PathBuf,
}

impl ReportPaths {
    pub fn for_date(date: &str) -> Self {
        Self {
            report: paths::report_path(date),
            sector_export: paths::sector_scores_export_path(date),
            watchlist_export: paths::stock_watchlist_export_path(date),
        }
    }
}
