use std::collections::{HashMap, HashSet};

use crate::config::{APP_NAME, event_data, macro_data, market_data, output_text, scoring};
use crate::domain::models::{
    IndustryScore, MacroObservation, MarketEvent, MarketRegimeScore, SectorScore, StockScore,
};
use crate::validation::MacroContextOverlay;

use super::formatting::{multiple, pct, score};

pub struct DailyReportInput<'a> {
    pub date: &'a str,
    pub regime: &'a MarketRegimeScore,
    pub sector_scores: &'a [SectorScore],
    pub industry_scores: &'a [IndustryScore],
    pub stock_scores: &'a [StockScore],
    pub events: &'a [MarketEvent],
    pub macro_observations: &'a [MacroObservation],
    pub macro_context: Option<&'a MacroContextOverlay>,
    pub previous_watchlist_symbols: &'a HashSet<String>,
}

pub fn daily_report_markdown(input: &DailyReportInput<'_>) -> String {
    [
        report_header(input.date),
        market_regime(input.regime),
        macro_context_overlay(input.macro_context),
        macro_context_coverage(input.macro_observations),
        sector_map_note(),
        top_sector_table(input.sector_scores),
        weak_sector_table(input.sector_scores),
        sector_rank_changes(input.sector_scores),
        top_industry_table(input.industry_scores),
        watchlist_table(input.stock_scores),
        new_leaders(input.stock_scores, input.previous_watchlist_symbols),
        high_relative_volume_table(input.stock_scores),
        catalyst_flags(input.stock_scores, input.events),
        notes_for_chart_review(),
        explanation_list(input.stock_scores),
    ]
    .join("\n\n")
}

fn report_header(date: &str) -> String {
    format!(
        "# Market Report: {date}\n\nSystem: {APP_NAME} {}\n\n{}",
        output_text::DAILY_REPORT_TITLE,
        output_text::REPORT_RULE
    )
}

fn market_regime(regime: &MarketRegimeScore) -> String {
    [
        section_heading(output_text::MARKET_REGIME_SECTION),
        output_text::MARKET_REGIME_V1_NOTE.to_string(),
        format!(
            "**{}** with regime score {}.",
            regime.label,
            score(regime.score)
        ),
        regime.explanation.clone(),
    ]
    .join("\n\n")
}

fn macro_context_overlay(macro_context: Option<&MacroContextOverlay>) -> String {
    let Some(context) = macro_context else {
        return [
            section_heading("Macro Context Overlay"),
            "Macro context overlay unavailable for this report date.".to_string(),
        ]
        .join("\n\n");
    };

    let flags = if context.active_flags.is_empty() {
        "none".to_string()
    } else {
        context.active_flags.join(", ")
    };
    let stale = if context.stale_series.is_empty() {
        "none".to_string()
    } else {
        context.stale_series.join(", ")
    };

    [
        section_heading("Macro Context Overlay"),
        "Macro flags are as-of context only. They are not Market Regime V1 score inputs."
            .to_string(),
        format!(
            "As-of {}: `{}` active macro flag(s). Coverage: {}/{} required series. Stale series: `{}`.",
            context.date, flags, context.covered_series_count, context.required_series_count, stale
        ),
        context.interpretation.clone(),
    ]
    .join("\n\n")
}

fn sector_map_note() -> String {
    output_text::SECTOR_MAP_NOTE.to_string()
}

fn macro_context_coverage(macro_observations: &[MacroObservation]) -> String {
    let mut coverage = macro_coverage(macro_observations);
    coverage.sort_by(|a, b| a.series.cmp(&b.series));

    let mut rows = vec![
        section_heading(output_text::MACRO_CONTEXT_SECTION),
        output_text::MACRO_CONTEXT_NOTE.to_string(),
        String::new(),
        output_text::MACRO_TABLE_HEADER.to_string(),
        output_text::MACRO_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(coverage.into_iter().map(|coverage| {
        format!(
            "| {} | {} | {} | {} | {} | {} |",
            coverage.series,
            coverage.name,
            coverage.frequency,
            coverage.latest_date.unwrap_or_else(|| "none".to_string()),
            coverage.observations,
            if coverage.observations > 0 {
                "stored"
            } else {
                "missing"
            }
        )
    }));

    rows.join("\n")
}

fn macro_coverage(macro_observations: &[MacroObservation]) -> Vec<MacroCoverage> {
    let mut coverage: HashMap<&str, MacroCoverage> = macro_data::MACRO_SERIES
        .iter()
        .map(|(series, name, frequency, _)| {
            (
                *series,
                MacroCoverage {
                    series: (*series).to_string(),
                    name: (*name).to_string(),
                    frequency: (*frequency).to_string(),
                    latest_date: None,
                    observations: 0,
                },
            )
        })
        .collect();

    for observation in macro_observations {
        if let Some(entry) = coverage.get_mut(observation.series.as_str()) {
            entry.observations += 1;
            if entry
                .latest_date
                .as_deref()
                .is_none_or(|latest| observation.date.as_str() > latest)
            {
                entry.latest_date = Some(observation.date.clone());
            }
        }
    }

    coverage.into_values().collect()
}

fn top_sector_table(sector_scores: &[SectorScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::TOP_SECTORS_SECTION),
        output_text::SECTOR_TABLE_HEADER.to_string(),
        output_text::SECTOR_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(
        sector_scores
            .iter()
            .take(scoring::TOP_SECTOR_REPORT_LIMIT)
            .map(sector_row),
    );
    rows.join("\n")
}

struct MacroCoverage {
    series: String,
    name: String,
    frequency: String,
    latest_date: Option<String>,
    observations: usize,
}

fn weak_sector_table(sector_scores: &[SectorScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::WEAK_SECTORS_SECTION),
        output_text::SECTOR_TABLE_HEADER.to_string(),
        output_text::SECTOR_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(
        sector_scores
            .iter()
            .rev()
            .take(scoring::WEAK_SECTOR_REPORT_LIMIT)
            .map(sector_row),
    );
    rows.join("\n")
}

fn sector_row(sector: &SectorScore) -> String {
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.0}% | {:.0}% | {} |",
        sector.rank,
        sector.sector,
        sector.sector_etf,
        score(sector.score),
        pct(sector.return_1d),
        pct(sector.return_5d),
        pct(sector.return_20d),
        pct(sector.return_60d),
        pct(sector.relative_return_vs_spy),
        multiple(sector.relative_volume),
        sector.breadth_20d,
        sector.breadth_50d,
        rank_change(sector.rank_change),
    )
}

fn sector_rank_changes(sector_scores: &[SectorScore]) -> String {
    let mut changed: Vec<&SectorScore> = sector_scores
        .iter()
        .filter(|sector| sector.rank_change != 0.0)
        .collect();
    changed.sort_by(|a, b| b.rank_change.abs().total_cmp(&a.rank_change.abs()));

    if changed.is_empty() {
        return format!(
            "{}\n{}",
            section_heading(output_text::SECTOR_RANK_CHANGES_SECTION),
            output_text::NO_PRIOR_RANK_HISTORY
        );
    }

    let mut rows = vec![
        section_heading(output_text::SECTOR_RANK_CHANGES_SECTION),
        "| Sector | ETF | Current Rank | Rank Change | Score |".to_string(),
        "|---|---:|---:|---:|---:|".to_string(),
    ];
    rows.extend(changed.into_iter().map(|sector| {
        format!(
            "| {} | {} | {} | {} | {} |",
            sector.sector,
            sector.sector_etf,
            sector.rank,
            rank_change(sector.rank_change),
            score(sector.score)
        )
    }));
    rows.join("\n")
}

fn top_industry_table(industry_scores: &[IndustryScore]) -> String {
    let mut rows = vec![
        section_heading(output_text::TOP_INDUSTRIES_SECTION),
        output_text::INDUSTRY_TABLE_HEADER.to_string(),
        output_text::INDUSTRY_TABLE_ALIGNMENT.to_string(),
    ];

    rows.extend(
        industry_scores
            .iter()
            .take(scoring::TOP_INDUSTRY_REPORT_LIMIT)
            .map(|industry| {
                format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.0}% | {:.0}% | {:.0}% | {} |",
                    industry.rank,
                    industry.industry,
                    industry.sector,
                    score(industry.score),
                    pct(industry.return_5d),
                    pct(industry.return_20d),
                    pct(industry.return_60d),
                    pct(industry.relative_return_vs_sector),
                    pct(industry.relative_return_vs_spy),
                    multiple(industry.relative_volume),
                    industry.breadth_20d,
                    industry.breadth_50d,
                    industry.high_20d_rate,
                    industry.member_count
                )
            }),
    );
    rows.join("\n")
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

fn new_leaders(
    stock_scores: &[StockScore],
    previous_watchlist_symbols: &HashSet<String>,
) -> String {
    if previous_watchlist_symbols.is_empty() {
        return format!(
            "{}\n{}",
            section_heading(output_text::NEW_LEADERS_SECTION),
            output_text::NO_PRIOR_WATCHLIST_HISTORY
        );
    }

    let new_names: Vec<&StockScore> = stock_scores
        .iter()
        .take(scoring::REPORT_WATCHLIST_LIMIT)
        .filter(|stock| !previous_watchlist_symbols.contains(&stock.symbol))
        .collect();

    if new_names.is_empty() {
        return format!(
            "{}\n{}",
            section_heading(output_text::NEW_LEADERS_SECTION),
            output_text::NO_NEW_LEADERS
        );
    }

    let mut rows = vec![
        section_heading(output_text::NEW_LEADERS_SECTION),
        output_text::WATCHLIST_TABLE_HEADER.to_string(),
        output_text::WATCHLIST_TABLE_ALIGNMENT.to_string(),
    ];
    rows.extend(new_names.into_iter().map(watchlist_row));
    rows.join("\n")
}

fn high_relative_volume_table(stock_scores: &[StockScore]) -> String {
    let mut high_volume: Vec<&StockScore> = stock_scores.iter().collect();
    high_volume.sort_by(|a, b| b.relative_volume.total_cmp(&a.relative_volume));

    let mut rows = vec![
        section_heading(output_text::HIGH_RELATIVE_VOLUME_SECTION),
        output_text::HIGH_RELATIVE_VOLUME_TABLE_HEADER.to_string(),
        output_text::HIGH_RELATIVE_VOLUME_TABLE_ALIGNMENT.to_string(),
    ];
    rows.extend(
        high_volume
            .into_iter()
            .take(scoring::HIGH_RELATIVE_VOLUME_REPORT_LIMIT)
            .map(|stock| {
                format!(
                    "| {} | {} | {} | {} | {} | {} | {} |",
                    stock.rank,
                    stock.symbol,
                    stock.sector,
                    score(stock.score),
                    multiple(stock.relative_volume),
                    pct(stock.return_20d),
                    pct(stock.relative_return_vs_sector)
                )
            }),
    );
    rows.join("\n")
}

fn catalyst_flags(stock_scores: &[StockScore], events: &[MarketEvent]) -> String {
    let flagged: Vec<&StockScore> = stock_scores
        .iter()
        .filter(|stock| stock.catalyst_status != scoring::CATALYST_PENDING_SOURCE)
        .take(scoring::REPORT_WATCHLIST_LIMIT)
        .collect();

    if flagged.is_empty() {
        return format!(
            "{}\n{}\n{}",
            section_heading(output_text::CATALYST_SECTION),
            output_text::CATALYST_SOURCE_NOTE,
            output_text::CATALYST_PENDING_NOTE
        );
    }

    let events_by_symbol = events_by_symbol(events);
    let mut rows = vec![
        section_heading(output_text::CATALYST_SECTION),
        output_text::CATALYST_SOURCE_NOTE.to_string(),
    ];
    for stock in flagged {
        rows.push(format!(
            "- **{}** `{}`",
            stock.symbol, stock.catalyst_status
        ));
        if let Some(symbol_events) = events_by_symbol.get(&stock.symbol) {
            rows.extend(event_detail_lines(symbol_events));
        }
    }
    rows.join("\n")
}

fn events_by_symbol(events: &[MarketEvent]) -> HashMap<String, Vec<&MarketEvent>> {
    let mut by_symbol = HashMap::new();
    for event in events {
        by_symbol
            .entry(event.symbol.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }
    by_symbol
}

fn event_detail_lines(events: &[&MarketEvent]) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(news) = latest_event(events, market_data::NEWS_EVENT_TYPE) {
        lines.push(format!(
            "  - News {} from {}: {}",
            news.event_date,
            news.source,
            clean_markdown_line(&news.headline)
        ));
    }
    if let Some(earnings) = earliest_event(events, event_data::EVENT_TYPE_EARNINGS) {
        let estimate = earnings
            .metadata
            .estimate
            .map(|value| format!(" estimate {value:.2}"))
            .unwrap_or_default();
        lines.push(format!(
            "  - Earnings calendar {}{}.",
            earnings.event_date, estimate
        ));
    }
    if let Some(filing) = latest_event(events, event_data::EVENT_TYPE_FILING) {
        let link = filing
            .url
            .as_deref()
            .map(|url| format!(" ({url})"))
            .unwrap_or_default();
        lines.push(format!(
            "  - SEC filing {}: {}{}",
            filing.event_date,
            clean_markdown_line(&filing.headline),
            link
        ));
    }

    lines
}

fn earliest_event<'a>(events: &[&'a MarketEvent], event_type: &str) -> Option<&'a MarketEvent> {
    events
        .iter()
        .copied()
        .filter(|event| event.event_type == event_type)
        .min_by(|left, right| left.event_date.cmp(&right.event_date))
}

fn latest_event<'a>(events: &[&'a MarketEvent], event_type: &str) -> Option<&'a MarketEvent> {
    events
        .iter()
        .copied()
        .filter(|event| event.event_type == event_type)
        .max_by(|left, right| left.event_date.cmp(&right.event_date))
}

fn clean_markdown_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn notes_for_chart_review() -> String {
    let mut rows = vec![section_heading(output_text::NOTES_SECTION)];
    rows.extend(
        output_text::CHART_REVIEW_NOTES
            .iter()
            .map(|note| format!("- {note}")),
    );
    rows.join("\n")
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

fn rank_change(value: f64) -> String {
    if value > 0.0 {
        format!("+{value:.0}")
    } else {
        format!("{value:.0}")
    }
}
