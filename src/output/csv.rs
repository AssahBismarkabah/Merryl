use std::path::Path;

use anyhow::{Context, Result};

use crate::config::scoring::REPORT_WATCHLIST_LIMIT;
use crate::domain::models::{SectorScore, StockScore};

pub fn write_sector_csv(path: &Path, scores: &[SectorScore]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for score in scores {
        writer.serialize(score)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn write_watchlist_csv(path: &Path, scores: &[StockScore]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for score in scores.iter().take(REPORT_WATCHLIST_LIMIT) {
        writer.serialize(score)?;
    }
    writer.flush()?;
    Ok(())
}
