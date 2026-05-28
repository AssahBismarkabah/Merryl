use std::collections::HashSet;
use std::env;

use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, Utc};
use csv::ReaderBuilder;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;

use crate::config::{USER_AGENT, event_data};
use crate::domain::models::{MarketEvent, MarketEventMetadata};

use super::provider::EarningsCalendarProvider;

pub struct AlphaVantageProvider {
    api_key: String,
    base_url: String,
    horizon: String,
    client: Client,
}

impl AlphaVantageProvider {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var(event_data::ALPHA_VANTAGE_API_KEY_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .with_context(|| {
                format!(
                    "missing {}; create a free Alpha Vantage API key before running Phase 5C catalyst ingestion",
                    event_data::ALPHA_VANTAGE_API_KEY_ENV
                )
            })?;
        let base_url = env::var(event_data::ALPHA_VANTAGE_API_URL_ENV)
            .unwrap_or_else(|_| event_data::ALPHA_VANTAGE_API_URL.to_string());
        let horizon = env::var(event_data::EARNINGS_CALENDAR_HORIZON_ENV)
            .unwrap_or_else(|_| event_data::DEFAULT_EARNINGS_CALENDAR_HORIZON.to_string());
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(event_data::http_timeout())
            .build()
            .context("failed to create Alpha Vantage HTTP client")?;

        Ok(Self {
            api_key,
            base_url,
            horizon,
            client,
        })
    }

    pub fn env_status() -> Vec<String> {
        if env::var(event_data::ALPHA_VANTAGE_API_KEY_ENV)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
        {
            vec!["ok: Alpha Vantage API key is set".to_string()]
        } else {
            vec![format!(
                "missing: Alpha Vantage API key ({})",
                event_data::ALPHA_VANTAGE_API_KEY_ENV
            )]
        }
    }

    pub fn parse_earnings_calendar_csv(
        symbols: &[String],
        response_csv: &str,
    ) -> Result<Vec<MarketEvent>> {
        let requested_symbols: HashSet<&str> = symbols.iter().map(String::as_str).collect();
        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(response_csv.as_bytes());
        let mut events = Vec::new();

        for row in reader.deserialize::<AlphaVantageEarningsRow>() {
            let Ok(row) = row else {
                continue;
            };
            let symbol = row.symbol.trim().to_uppercase();
            let report_date = row.report_date.trim();
            if symbol.is_empty() || !requested_symbols.contains(symbol.as_str()) {
                continue;
            }
            if NaiveDate::parse_from_str(report_date, "%Y-%m-%d").is_err() {
                continue;
            }

            let fiscal_period = none_if_empty(row.fiscal_date_ending);
            let estimate = parse_optional_f64(row.estimate.as_deref());
            let source_event_id = format!(
                "{}:{}:{}",
                event_data::ALPHA_VANTAGE_SOURCE_NAME,
                symbol,
                report_date
            );
            let raw_json = json!({
                "symbol": &symbol,
                "name": &row.name,
                "reportDate": report_date,
                "fiscalDateEnding": &fiscal_period,
                "estimate": &row.estimate,
                "currency": &row.currency
            })
            .to_string();

            events.push(MarketEvent {
                symbol: symbol.clone(),
                sector: None,
                event_date: report_date.to_string(),
                event_type: event_data::EVENT_TYPE_EARNINGS.to_string(),
                headline: format!(
                    "Expected earnings for {}",
                    label_or_symbol(&row.name, &symbol)
                ),
                source: event_data::ALPHA_VANTAGE_SOURCE_NAME.to_string(),
                url: None,
                metadata: MarketEventMetadata {
                    source_event_id: Some(source_event_id),
                    effective_date: Some(report_date.to_string()),
                    fetched_at: Some(Utc::now().to_rfc3339()),
                    estimate,
                    fiscal_period,
                    raw_json: Some(raw_json),
                    ..MarketEventMetadata::default()
                },
            });
        }

        Ok(events)
    }

    fn fetch_earnings_calendar(&self) -> Result<String> {
        let response = self
            .client
            .get(format!(
                "{}{}",
                self.base_url,
                event_data::ALPHA_VANTAGE_QUERY_PATH
            ))
            .query(&[
                (
                    "function",
                    event_data::ALPHA_VANTAGE_EARNINGS_CALENDAR_FUNCTION.to_string(),
                ),
                ("horizon", self.horizon.clone()),
                ("apikey", self.api_key.clone()),
            ])
            .send()
            .context("failed to fetch Alpha Vantage earnings calendar")?
            .error_for_status()
            .context("Alpha Vantage earnings calendar request failed")?
            .text()
            .context("failed to read Alpha Vantage earnings calendar response")?;

        if response.trim_start().starts_with('{') {
            bail!(
                "Alpha Vantage earnings calendar returned a non-CSV response; check API key and free API limits"
            );
        }

        Ok(response)
    }
}

impl EarningsCalendarProvider for AlphaVantageProvider {
    fn upcoming_earnings_events(&self, symbols: &[String]) -> Result<Vec<MarketEvent>> {
        let response = self.fetch_earnings_calendar()?;
        Self::parse_earnings_calendar_csv(symbols, &response)
    }
}

#[derive(Debug, Deserialize)]
struct AlphaVantageEarningsRow {
    symbol: String,
    #[serde(default)]
    name: String,
    #[serde(rename = "reportDate")]
    report_date: String,
    #[serde(rename = "fiscalDateEnding", default)]
    fiscal_date_ending: String,
    #[serde(default)]
    estimate: Option<String>,
    #[serde(default)]
    currency: Option<String>,
}

fn none_if_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_optional_f64(value: Option<&str>) -> Option<f64> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<f64>().ok())
}

fn label_or_symbol(name: &str, symbol: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        symbol.to_string()
    } else {
        name.to_string()
    }
}
