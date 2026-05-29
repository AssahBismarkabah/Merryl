use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::paths;
use crate::validation::{ActionabilitySummaryRow, ActionabilityValidationMetrics};

use super::formatting::pct;

#[derive(Debug, Clone)]
pub struct ActionabilityValidationOutputPaths {
    pub report: PathBuf,
    pub summary_export: PathBuf,
}

pub fn write_actionability_validation_outputs(
    metrics: &ActionabilityValidationMetrics,
) -> Result<ActionabilityValidationOutputPaths> {
    fs::create_dir_all(paths::VALIDATION_REPORTS_DIR)
        .context("failed to create validation reports directory")?;
    fs::create_dir_all(paths::VALIDATION_EXPORTS_DIR)
        .context("failed to create validation exports directory")?;

    let paths = ActionabilityValidationOutputPaths {
        report: paths::actionability_validation_report_path(&metrics.from_date, &metrics.to_date),
        summary_export: paths::actionability_validation_export_path(
            &metrics.from_date,
            &metrics.to_date,
        ),
    };

    fs::write(&paths.report, actionability_validation_markdown(metrics))
        .with_context(|| format!("failed to write {}", paths.report.display()))?;
    write_actionability_validation_csv(&paths.summary_export, &metrics.summaries)?;

    Ok(paths)
}

fn actionability_validation_markdown(metrics: &ActionabilityValidationMetrics) -> String {
    [
        format!(
            "# Merryl Actionability Validation: {} to {}",
            metrics.from_date, metrics.to_date
        ),
        validation_scope(metrics),
        coverage_summary(metrics),
        summary_table(&metrics.summaries),
        limitations(),
    ]
    .join("\n\n")
}

fn validation_scope(metrics: &ActionabilityValidationMetrics) -> String {
    [
        "## Validation Scope".to_string(),
        format!("Purpose: `{}`.", metrics.validation_scope.purpose),
        "This validates actionability buckets for watchlist review. It is not a trading recommendation and does not change score weights.".to_string(),
        bullet_section("What this can show", &metrics.validation_scope.proves),
        bullet_section(
            "What this does not prove",
            &metrics.validation_scope.does_not_prove,
        ),
        format!(
            "Date alignment rule: {}",
            metrics.validation_scope.date_alignment_rule
        ),
        format!("Group policy: {}", metrics.validation_scope.group_policy),
    ]
    .join("\n\n")
}

fn coverage_summary(metrics: &ActionabilityValidationMetrics) -> String {
    [
        "## Coverage Summary".to_string(),
        format!("Watchlist rows reviewed: `{}`", metrics.watchlist_row_count),
        format!(
            "Scored watchlist rows matched: `{}`",
            metrics.scored_watchlist_row_count
        ),
        format!(
            "Extended leader rows: `{}`",
            metrics.extended_leader_row_count
        ),
        format!(
            "Useful review bucket rows: `{}`",
            metrics.useful_review_row_count
        ),
        format!(
            "Unclassified leader rows: `{}`",
            metrics.unclassified_leader_row_count
        ),
        format!(
            "Valid forward observations: `{}`",
            metrics.forward_observation_count
        ),
        format!(
            "Grouped forward observations: `{}`",
            metrics.grouped_forward_observation_count
        ),
        format!(
            "Skipped horizons missing future bars: `{}`",
            metrics.skipped_missing_future_bars
        ),
    ]
    .join("\n\n")
}

fn summary_table(rows: &[ActionabilitySummaryRow]) -> String {
    let mut lines = vec![
        "## Actionability Summary".to_string(),
        "| Group | Horizon | Count | Hit Rate vs Sector | Avg Fwd | Med Fwd | Avg vs SPY | Med vs SPY | Avg vs Sector | Med vs Sector |".to_string(),
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|".to_string(),
    ];

    lines.extend(rows.iter().map(|row| {
        format!(
            "| {} | {}D | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.group,
            row.horizon,
            row.count,
            pct(row.hit_rate),
            pct(row.average_forward_return),
            pct(row.median_forward_return),
            pct(row.average_relative_return_vs_spy),
            pct(row.median_relative_return_vs_spy),
            pct(row.average_relative_return_vs_sector),
            pct(row.median_relative_return_vs_sector)
        )
    }));

    if rows.is_empty() {
        lines.push("|  |  | 0 |  |  |  |  |  |  |  |".to_string());
    }

    lines.join("\n")
}

fn limitations() -> String {
    [
        "## Limitations".to_string(),
        "- Actionability is a watchlist review layer and is not a scoring input.".to_string(),
        "- Fresh databases may have too few future bars for strong conclusions.".to_string(),
        "- `extended_leader` means already stretched versus current thresholds; it is not a sell signal.".to_string(),
        "- `early_rotation_candidate`, `base_compression_candidate`, and `pullback_leader` still require manual chart confirmation.".to_string(),
        "- This report does not model trade entries, exits, costs, slippage, position sizing, or portfolio P&L.".to_string(),
    ]
    .join("\n")
}

fn bullet_section(title: &str, items: &[String]) -> String {
    let mut lines = vec![format!("### {title}")];
    lines.extend(items.iter().map(|item| format!("- {item}")));
    lines.join("\n")
}

fn write_actionability_validation_csv(path: &Path, rows: &[ActionabilitySummaryRow]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}
