use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::paths;
use crate::validation::{
    MacroFlagSummary, MacroRegimeValidationMetrics, MacroSectorLeadershipSummary,
    MacroSeriesFreshness,
};

use super::formatting::{pct, score};

const SECTOR_LEADERSHIP_ROWS_PER_FLAG: usize = 5;

#[derive(Debug, Clone)]
pub struct MacroRegimeValidationOutputPaths {
    pub report: PathBuf,
    pub summary_export: PathBuf,
}

pub fn write_macro_regime_validation_outputs(
    metrics: &MacroRegimeValidationMetrics,
) -> Result<MacroRegimeValidationOutputPaths> {
    fs::create_dir_all(paths::VALIDATION_REPORTS_DIR)
        .context("failed to create validation reports directory")?;
    fs::create_dir_all(paths::VALIDATION_EXPORTS_DIR)
        .context("failed to create validation exports directory")?;

    let paths = MacroRegimeValidationOutputPaths {
        report: paths::macro_regime_validation_report_path(&metrics.from_date, &metrics.to_date),
        summary_export: paths::macro_regime_validation_export_path(
            &metrics.from_date,
            &metrics.to_date,
        ),
    };

    fs::write(&paths.report, macro_regime_validation_markdown(metrics))
        .with_context(|| format!("failed to write {}", paths.report.display()))?;
    write_macro_regime_validation_csv(&paths.summary_export, &metrics.flag_summaries)?;

    Ok(paths)
}

fn macro_regime_validation_markdown(metrics: &MacroRegimeValidationMetrics) -> String {
    [
        format!(
            "# Merryl Macro Regime Validation: {} to {}",
            metrics.from_date, metrics.to_date
        ),
        validation_scope(metrics),
        coverage_summary(metrics),
        series_freshness_table(&metrics.series_freshness),
        flag_summary_table(&metrics.flag_summaries),
        sector_leadership_table(&metrics.sector_leadership),
        disagreement_examples(metrics),
        limitations(),
    ]
    .join("\n\n")
}

fn validation_scope(metrics: &MacroRegimeValidationMetrics) -> String {
    [
        "## Validation Scope".to_string(),
        format!("Purpose: `{}`.", metrics.validation_scope.purpose),
        "This report validates macro context against stored ETF-proxy regime scores. It is not a trading recommendation and does not change score weights.".to_string(),
        bullet_section("What this can show", &metrics.validation_scope.proves),
        bullet_section(
            "What this does not prove",
            &metrics.validation_scope.does_not_prove,
        ),
        format!(
            "Date alignment rule: {}",
            metrics.validation_scope.date_alignment_rule
        ),
        format!(
            "Revision limitation: {}",
            metrics.validation_scope.revision_limitation
        ),
    ]
    .join("\n\n")
}

fn coverage_summary(metrics: &MacroRegimeValidationMetrics) -> String {
    [
        "## Coverage Summary".to_string(),
        format!("Score dates: `{}`", metrics.score_date_count),
        format!("Macro snapshots: `{}`", metrics.macro_snapshot_count),
        format!(
            "Complete macro snapshots: `{}`",
            metrics.complete_macro_snapshot_count
        ),
        format!(
            "Missing macro snapshots: `{}`",
            metrics.missing_macro_snapshot_count
        ),
        format!(
            "Snapshots with at least one stale macro series: `{}`",
            metrics.stale_macro_snapshot_count
        ),
        format!(
            "Risk-on dates with active macro stress flags: `{}`",
            metrics.risk_on_with_stress_count
        ),
        format!(
            "Defensive/mixed dates with no active macro stress flags: `{}`",
            metrics.defensive_or_mixed_with_improving_count
        ),
    ]
    .join("\n\n")
}

fn series_freshness_table(rows: &[MacroSeriesFreshness]) -> String {
    let mut lines = vec![
        "## Macro Series Freshness".to_string(),
        "| Series | Frequency | Covered | Missing | Stale | Avg Age | Max Age | Latest Observation |".to_string(),
        "|---|---|---:|---:|---:|---:|---:|---|".to_string(),
    ];
    lines.extend(rows.iter().map(|row| {
        format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            row.series,
            row.frequency,
            row.score_dates_covered,
            row.missing_score_dates,
            row.stale_score_dates,
            optional_days(row.average_age_days),
            row.max_age_days
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.latest_observation_date.clone().unwrap_or_default()
        )
    }));
    lines.join("\n")
}

fn flag_summary_table(rows: &[MacroFlagSummary]) -> String {
    let mut lines = vec![
        "## Macro Flag Summary".to_string(),
        "| Flag | Active Dates | Active Share | Avg Regime Score | Risk-on Dates | Defensive/Mixed Dates |".to_string(),
        "|---|---:|---:|---:|---:|---:|".to_string(),
    ];
    lines.extend(rows.iter().map(|row| {
        format!(
            "| {} | {} | {} | {} | {} | {} |",
            row.flag,
            row.active_dates,
            pct(row.active_share),
            row.average_regime_score_when_active
                .map(score)
                .unwrap_or_default(),
            row.risk_on_dates_when_active,
            row.defensive_or_mixed_dates_when_active
        )
    }));
    lines.join("\n")
}

fn sector_leadership_table(rows: &[MacroSectorLeadershipSummary]) -> String {
    let mut by_flag: HashMap<&str, usize> = HashMap::new();
    let mut lines = vec![
        "## Sector Leadership Under Active Macro Flags".to_string(),
        "| Flag | Sector | Observations | Top Rank Count | Avg Rank | Avg Score |".to_string(),
        "|---|---|---:|---:|---:|---:|".to_string(),
    ];

    for row in rows {
        let count = by_flag.entry(&row.flag).or_default();
        if *count >= SECTOR_LEADERSHIP_ROWS_PER_FLAG {
            continue;
        }
        *count += 1;
        lines.push(format!(
            "| {} | {} | {} | {} | {:.1} | {} |",
            row.flag,
            row.sector,
            row.observations,
            row.top_rank_count,
            row.average_rank,
            score(row.average_score)
        ));
    }

    if lines.len() == 3 {
        lines.push("|  |  | 0 | 0 |  |  |".to_string());
    }

    lines.join("\n")
}

fn disagreement_examples(metrics: &MacroRegimeValidationMetrics) -> String {
    let mut lines = vec![
        "## Regime/Macro Disagreement Examples".to_string(),
        "| Date | ETF-Proxy Regime | Score | Macro Flags | Reason |".to_string(),
        "|---|---|---:|---|---|".to_string(),
    ];
    lines.extend(metrics.disagreement_examples.iter().map(|example| {
        format!(
            "| {} | {} | {} | {} | {} |",
            example.date,
            example.regime_label,
            score(example.regime_score),
            example.active_flags.join(", "),
            example.reason
        )
    }));
    if metrics.disagreement_examples.is_empty() {
        lines.push("|  |  |  |  | No disagreement examples in this date range. |".to_string());
    }
    lines.join("\n")
}

fn limitations() -> String {
    [
        "## Limitations".to_string(),
        "- Macro context is validation context only and is not a scoring input.".to_string(),
        "- FRED observations use the stored latest vintage, not reconstructed point-in-time vintages.".to_string(),
        "- Macro flags are simple transparent conditions, not optimized predictive signals.".to_string(),
        "- This report does not model trade entries, exits, costs, slippage, position sizing, or portfolio P&L.".to_string(),
    ]
    .join("\n")
}

fn bullet_section(title: &str, items: &[String]) -> String {
    let mut lines = vec![format!("### {title}")];
    lines.extend(items.iter().map(|item| format!("- {item}")));
    lines.join("\n")
}

fn optional_days(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.1}")).unwrap_or_default()
}

fn write_macro_regime_validation_csv(path: &Path, rows: &[MacroFlagSummary]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}
