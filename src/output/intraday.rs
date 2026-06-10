use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::paths;
use crate::domain::models::IntradaySetup;
use crate::intraday::IntradayReadinessResult;

use super::formatting::{multiple, pct};

#[derive(Debug, Clone)]
pub struct IntradayOutputPaths {
    pub report: PathBuf,
    pub export: PathBuf,
}

pub fn write_intraday_outputs(result: &IntradayReadinessResult) -> Result<IntradayOutputPaths> {
    fs::create_dir_all(paths::INTRADAY_REPORTS_DIR)
        .context("failed to create intraday reports directory")?;
    fs::create_dir_all(paths::INTRADAY_EXPORTS_DIR)
        .context("failed to create intraday exports directory")?;

    let paths = IntradayOutputPaths {
        report: paths::intraday_readiness_report_path(&result.date),
        export: paths::intraday_readiness_export_path(&result.date),
    };

    fs::write(&paths.report, intraday_report_markdown(result))
        .with_context(|| format!("failed to write {}", paths.report.display()))?;
    write_intraday_csv(&paths.export, &result.setups)?;

    Ok(paths)
}

fn intraday_report_markdown(result: &IntradayReadinessResult) -> String {
    [
        format!("# Merryl Intraday Execution Readiness: {}", result.date),
        "This report is signal-only. It identifies readiness conditions after the daily market map; it does not place orders, size positions, manage stops, or replace chart review.".to_string(),
        format!(
            "Evaluated stocks: `{}`\n\nStage 1 high-momentum candidates: `{}`\n\nStage 2 structural pullbacks: `{}`\n\nStage 3 trigger events: `{}`",
            result.evaluated_count,
            result.stage1_count,
            result.stage2_count,
            result.stage3_trigger_count
        ),
        readiness_table(&result.setups),
        trigger_summary(result),
    ]
    .join("\n\n")
}

fn readiness_table(setups: &[IntradaySetup]) -> String {
    let mut lines = vec![
        "## Readiness Queue".to_string(),
        "| Symbol | Label | Sector | ADR | rVOL | Mansfield SPY | EMA 10 | EMA 20 | Confluence | Triggers |".to_string(),
        "|---|---|---|---:|---:|---:|---:|---:|---:|---:|".to_string(),
    ];

    lines.extend(setups.iter().map(|setup| {
        format!(
            "| {} | {} | {} | {} | {} | {:.3} | {:.2} | {:.2} | {} | {} |",
            setup.symbol,
            setup.primary_label,
            setup.sector,
            pct(setup.adr_pct),
            multiple(setup.rvol_ratio),
            setup.mansfield_rs_spy,
            setup.ema_10,
            setup.ema_20,
            setup.confluence_count,
            setup.trigger_count
        )
    }));

    lines.join("\n")
}

fn trigger_summary(result: &IntradayReadinessResult) -> String {
    if result.triggers.is_empty() {
        return "## Trigger Events\n\nNo Stage 3 trigger events were detected for this run."
            .to_string();
    }

    let mut lines = vec![
        "## Trigger Events".to_string(),
        "| Symbol | Time | Trigger | Price | Reference | Volume Spike |".to_string(),
        "|---|---|---|---:|---:|---:|".to_string(),
    ];
    lines.extend(result.triggers.iter().map(|trigger| {
        format!(
            "| {} | {} | {} | {:.2} | {:.2} | {} |",
            trigger.symbol,
            trigger.ts,
            trigger.trigger_type,
            trigger.trigger_price,
            trigger.reference_level,
            multiple(trigger.volume_spike)
        )
    }));
    lines.join("\n")
}

fn write_intraday_csv(path: &Path, setups: &[IntradaySetup]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for setup in setups {
        writer.serialize(setup)?;
    }
    writer.flush()?;
    Ok(())
}
