use std::collections::{HashMap, HashSet};

use crate::config::{classification, scoring};
use crate::domain::models::{IndustryScore, SectorScore, StockScore};
use crate::validation::MacroContextOverlay;

pub struct WatchlistClassifier<'a> {
    sector_ranks: HashMap<&'a str, usize>,
    industry_ranks: HashMap<(&'a str, &'a str), usize>,
    previous_watchlist_symbols: &'a HashSet<String>,
    macro_conflict: bool,
}

impl<'a> WatchlistClassifier<'a> {
    pub fn new(
        sector_scores: &'a [SectorScore],
        industry_scores: &'a [IndustryScore],
        previous_watchlist_symbols: &'a HashSet<String>,
        regime_label: &str,
        macro_context: Option<&MacroContextOverlay>,
    ) -> Self {
        let sector_ranks = sector_scores
            .iter()
            .map(|sector| (sector.sector.as_str(), sector.rank))
            .collect();
        let industry_ranks = industry_scores
            .iter()
            .map(|industry| {
                (
                    (industry.sector.as_str(), industry.industry.as_str()),
                    industry.rank,
                )
            })
            .collect();

        Self {
            sector_ranks,
            industry_ranks,
            previous_watchlist_symbols,
            macro_conflict: macro_context_conflict(regime_label, macro_context),
        }
    }

    pub fn labels_for(&self, stock: &StockScore) -> Vec<String> {
        let mut labels = Vec::new();

        if self
            .sector_ranks
            .get(stock.sector.as_str())
            .is_some_and(|rank| *rank <= classification::LEADING_SECTOR_MAX_RANK)
        {
            labels.push(classification::LABEL_SECTOR_LEADER.to_string());
        }
        if self
            .industry_ranks
            .get(&(stock.sector.as_str(), stock.industry.as_str()))
            .is_some_and(|rank| *rank <= classification::LEADING_INDUSTRY_MAX_RANK)
        {
            labels.push(classification::LABEL_INDUSTRY_LEADER.to_string());
        }
        if stock.relative_return_vs_sector > classification::RELATIVE_STRENGTH_MIN_RETURN
            && stock.relative_return_vs_spy > classification::RELATIVE_STRENGTH_MIN_RETURN
        {
            labels.push(classification::LABEL_RELATIVE_STRENGTH_LEADER.to_string());
        }
        if stock.relative_volume >= classification::VOLUME_CONFIRMED_MIN_RELATIVE_VOLUME {
            labels.push(classification::LABEL_VOLUME_CONFIRMED.to_string());
        }
        if !self.previous_watchlist_symbols.is_empty()
            && !self.previous_watchlist_symbols.contains(&stock.symbol)
        {
            labels.push(classification::LABEL_NEW_LEADER.to_string());
        }
        if stock.catalyst_status != scoring::CATALYST_PENDING_SOURCE {
            labels.push(classification::LABEL_EVENT_CONTEXT.to_string());
        }
        if stock.catalyst_status.contains("earnings:") || stock.catalyst_status.contains("filing:")
        {
            labels.push(classification::LABEL_EVENT_RISK.to_string());
        }
        if self.macro_conflict {
            labels.push(classification::LABEL_MACRO_CONFLICT_CONTEXT.to_string());
        }

        labels
    }
}

fn macro_context_conflict(regime_label: &str, macro_context: Option<&MacroContextOverlay>) -> bool {
    let Some(context) = macro_context else {
        return false;
    };
    let label = regime_label.to_ascii_lowercase().replace('-', "_");
    let risk_on = label.contains("risk_on");
    let defensive_or_mixed =
        label.contains("defensive") || label.contains("mixed") || label.contains("risk_off");

    (risk_on && !context.active_flags.is_empty())
        || (defensive_or_mixed && context.active_flags.is_empty())
}
