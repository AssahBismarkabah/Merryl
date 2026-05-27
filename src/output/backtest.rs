use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::backtest::{BacktestMetrics, BacktestSummaryRow};
use crate::config::paths;

use super::formatting::pct;

#[derive(Debug, Clone)]
pub struct BacktestOutputPaths {
    pub report: PathBuf,
    pub summary_export: PathBuf,
}

pub fn write_backtest_outputs(metrics: &BacktestMetrics) -> Result<BacktestOutputPaths> {
    fs::create_dir_all(paths::BACKTEST_REPORTS_DIR)
        .context("failed to create backtest reports directory")?;
    fs::create_dir_all(paths::BACKTEST_EXPORTS_DIR)
        .context("failed to create backtest exports directory")?;

    let paths = BacktestOutputPaths {
        report: paths::backtest_report_path(&metrics.from_date, &metrics.to_date),
        summary_export: paths::backtest_summary_export_path(&metrics.from_date, &metrics.to_date),
    };

    fs::write(&paths.report, backtest_report_markdown(metrics))
        .with_context(|| format!("failed to write {}", paths.report.display()))?;
    write_backtest_summary_csv(&paths.summary_export, &metrics.summaries)?;

    Ok(paths)
}

fn backtest_report_markdown(metrics: &BacktestMetrics) -> String {
    [
        format!(
            "# Merryl Backtest Report: {} to {}",
            metrics.from_date, metrics.to_date
        ),
        "Rule: this report validates historical score behavior. It is not a trading recommendation and does not model execution, slippage, taxes, or portfolio constraints."
            .to_string(),
        format!(
            "Sector observations: `{}`\n\nSector component observations: `{}`\n\nStock observations: `{}`\n\nIndustry validation observations: `{}`",
            metrics.sector_observation_count,
            metrics.sector_component_observation_count,
            metrics.stock_observation_count,
            metrics.industry_stock_observation_count
        ),
        "Primary relative return means sector ETF vs SPY for sectors, and stock vs sector ETF for stocks."
            .to_string(),
        "`sector_component_*` rows group sector ETF forward returns by individual same-day sector component deciles. Decile 10 means the strongest value for that component on that score date."
            .to_string(),
        "`stock_by_industry` rows group stock forward returns by the same-day industry/theme score decile. Decile 10 means the strongest industries/themes for that score date."
            .to_string(),
        summary_table(&metrics.summaries),
    ]
    .join("\n\n")
}

fn summary_table(rows: &[BacktestSummaryRow]) -> String {
    let mut lines = vec![
        "## Decile Summary".to_string(),
        "| Type | Horizon | Decile | Count | Hit Rate | Avg Fwd | Med Fwd | Avg Relative | Med Relative | Avg vs SPY | Med vs SPY | Avg vs Sector | Med vs Sector |".to_string(),
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|".to_string(),
    ];

    lines.extend(rows.iter().map(|row| {
        format!(
            "| {} | {}D | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.entity_type,
            row.horizon,
            row.decile,
            row.count,
            pct(row.hit_rate),
            pct(row.average_forward_return),
            pct(row.median_forward_return),
            pct(row.average_relative_return),
            pct(row.median_relative_return),
            pct(row.average_relative_return_vs_spy),
            pct(row.median_relative_return_vs_spy),
            optional_pct(row.average_relative_return_vs_sector),
            optional_pct(row.median_relative_return_vs_sector)
        )
    }));

    lines.join("\n")
}

fn optional_pct(value: Option<f64>) -> String {
    value.map(pct).unwrap_or_default()
}

fn write_backtest_summary_csv(path: &Path, rows: &[BacktestSummaryRow]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}
