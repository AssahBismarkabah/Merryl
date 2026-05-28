use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::paths;
use crate::validation::{EventContextSummaryRow, EventContextValidationMetrics};

use super::formatting::pct;

#[derive(Debug, Clone)]
pub struct EventContextValidationOutputPaths {
    pub report: PathBuf,
    pub summary_export: PathBuf,
}

pub fn write_event_context_validation_outputs(
    metrics: &EventContextValidationMetrics,
) -> Result<EventContextValidationOutputPaths> {
    fs::create_dir_all(paths::VALIDATION_REPORTS_DIR)
        .context("failed to create validation reports directory")?;
    fs::create_dir_all(paths::VALIDATION_EXPORTS_DIR)
        .context("failed to create validation exports directory")?;

    let paths = EventContextValidationOutputPaths {
        report: paths::event_context_validation_report_path(&metrics.from_date, &metrics.to_date),
        summary_export: paths::event_context_validation_export_path(
            &metrics.from_date,
            &metrics.to_date,
        ),
    };

    fs::write(&paths.report, event_context_validation_markdown(metrics))
        .with_context(|| format!("failed to write {}", paths.report.display()))?;
    write_event_context_validation_csv(&paths.summary_export, &metrics.summaries)?;

    Ok(paths)
}

fn event_context_validation_markdown(metrics: &EventContextValidationMetrics) -> String {
    [
        format!(
            "# Merryl Event Context Validation: {} to {}",
            metrics.from_date, metrics.to_date
        ),
        validation_scope(metrics),
        coverage_summary(metrics),
        summary_table(&metrics.summaries),
        limitations(),
    ]
    .join("\n\n")
}

fn validation_scope(metrics: &EventContextValidationMetrics) -> String {
    [
        "## Validation Scope".to_string(),
        format!("Purpose: `{}`.", metrics.validation_scope.purpose),
        "This validates event context behavior for watchlist review. It is not a trading recommendation and does not change score weights.".to_string(),
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

fn coverage_summary(metrics: &EventContextValidationMetrics) -> String {
    [
        "## Coverage Summary".to_string(),
        format!("Watchlist rows reviewed: `{}`", metrics.watchlist_row_count),
        format!(
            "Scored watchlist rows matched: `{}`",
            metrics.scored_watchlist_row_count
        ),
        format!(
            "Rows with event context: `{}`",
            metrics.event_context_row_count
        ),
        format!(
            "Rows with pending source: `{}`",
            metrics.pending_source_row_count
        ),
        format!(
            "Valid forward observations: `{}`",
            metrics.forward_observation_count
        ),
        format!(
            "Event-context forward observations: `{}`",
            metrics.event_context_forward_observation_count
        ),
        format!(
            "Grouped forward observations: `{}`",
            metrics.grouped_forward_observation_count
        ),
        format!(
            "Skipped horizons missing future bars: `{}`",
            metrics.skipped_missing_future_bars
        ),
        event_context_forward_note(metrics),
    ]
    .join("\n\n")
}

fn event_context_forward_note(metrics: &EventContextValidationMetrics) -> String {
    if metrics.event_context_row_count > 0 && metrics.event_context_forward_observation_count == 0 {
        "Event-context rows exist, but they do not yet have enough future bars for forward validation in this date range.".to_string()
    } else {
        "Event-context rows with future bars are included in the grouped summary below.".to_string()
    }
}

fn summary_table(rows: &[EventContextSummaryRow]) -> String {
    let mut lines = vec![
        "## Event Context Summary".to_string(),
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
        "- Event context is validation context only and is not a scoring input.".to_string(),
        "- Fresh databases may have too few historical event-labeled watchlist rows for strong conclusions.".to_string(),
        "- `pending_source` means no stored event label for that score date; it is not a bearish or neutral signal.".to_string(),
        "- Earnings and filing labels describe event risk/context, not direction.".to_string(),
        "- This report does not model trade entries, exits, costs, slippage, position sizing, or portfolio P&L.".to_string(),
    ]
    .join("\n")
}

fn bullet_section(title: &str, items: &[String]) -> String {
    let mut lines = vec![format!("### {title}")];
    lines.extend(items.iter().map(|item| format!("- {item}")));
    lines.join("\n")
}

fn write_event_context_validation_csv(path: &Path, rows: &[EventContextSummaryRow]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}
