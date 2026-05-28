use crate::config::{event_data, market_data, scoring};
use crate::domain::models::{MarketEvent, StockScore};

pub fn apply_catalyst_status(stock_scores: &mut [StockScore], events: &[MarketEvent]) {
    for stock in stock_scores {
        stock.catalyst_status = catalyst_status_for_symbol(&stock.symbol, events);
    }
}

pub fn catalyst_status_for_symbol(symbol: &str, events: &[MarketEvent]) -> String {
    let symbol_events: Vec<&MarketEvent> = events
        .iter()
        .filter(|event| event.symbol == symbol)
        .collect();
    let mut labels = Vec::new();

    let news_count = symbol_events
        .iter()
        .filter(|event| event.event_type == market_data::NEWS_EVENT_TYPE)
        .count();
    if news_count > 0 {
        labels.push(format!(
            "{}:{}",
            scoring::CATALYST_RECENT_NEWS_PREFIX,
            news_count
        ));
    }

    if let Some(earnings) = earliest_event(&symbol_events, event_data::EVENT_TYPE_EARNINGS) {
        labels.push(format!(
            "{}:{}",
            scoring::CATALYST_EARNINGS_PREFIX,
            earnings.event_date
        ));
    }

    if let Some(filing) = latest_event(&symbol_events, event_data::EVENT_TYPE_FILING) {
        labels.push(format!(
            "{}:{}",
            scoring::CATALYST_FILING_PREFIX,
            filing_form(filing)
        ));
    }

    if labels.is_empty() {
        scoring::CATALYST_PENDING_SOURCE.to_string()
    } else {
        labels.join(scoring::CATALYST_SEPARATOR)
    }
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

fn filing_form(event: &MarketEvent) -> String {
    event
        .headline
        .split_whitespace()
        .next()
        .unwrap_or(event_data::EVENT_TYPE_FILING)
        .to_string()
}
