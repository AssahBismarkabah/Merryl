use std::collections::{HashMap, HashSet};
use std::env;
use std::thread;

use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, NaiveDate, Utc};
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use serde::Deserialize;
use serde_json::json;

use crate::config::event_data;
use crate::domain::models::{MarketEvent, MarketEventMetadata};

use super::provider::FilingEventProvider;

pub struct SecEdgarProvider {
    lookback_calendar_days: i64,
    user_agent: String,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct SecCompany {
    pub cik: u64,
    pub ticker: String,
    pub title: String,
}

impl SecEdgarProvider {
    pub fn from_env() -> Result<Self> {
        let lookback_calendar_days = env::var(event_data::SEC_FILINGS_LOOKBACK_DAYS_ENV)
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(event_data::DEFAULT_SEC_FILINGS_LOOKBACK_DAYS);
        let user_agent = env::var(event_data::SEC_USER_AGENT_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .with_context(|| {
                format!(
                    "missing {}; set a SEC-compliant user agent with contact information before running Phase 5C filing ingestion",
                    event_data::SEC_USER_AGENT_ENV
                )
            })?;
        let client = Client::builder()
            .user_agent(user_agent.clone())
            .timeout(event_data::http_timeout())
            .build()
            .context("failed to create SEC EDGAR HTTP client")?;

        Ok(Self {
            lookback_calendar_days,
            user_agent,
            client,
        })
    }

    pub fn env_status() -> Vec<String> {
        if env::var(event_data::SEC_USER_AGENT_ENV)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
        {
            vec!["ok: SEC EDGAR user agent is set (no API key required)".to_string()]
        } else {
            vec![format!(
                "missing: SEC EDGAR user agent ({})",
                event_data::SEC_USER_AGENT_ENV
            )]
        }
    }

    pub fn parse_company_tickers_json(response_json: &str) -> Result<HashMap<String, SecCompany>> {
        let rows: HashMap<String, SecCompanyTickerRow> =
            serde_json::from_str(response_json).context("failed to parse SEC company tickers")?;
        let mut companies = HashMap::new();

        for row in rows.into_values() {
            companies.insert(
                row.ticker.to_uppercase(),
                SecCompany {
                    cik: row.cik_str,
                    ticker: row.ticker.to_uppercase(),
                    title: row.title,
                },
            );
        }

        Ok(companies)
    }

    pub fn parse_submissions_json(
        symbol: &str,
        cik: u64,
        response_json: &str,
        end_date: NaiveDate,
        lookback_calendar_days: i64,
    ) -> Result<Vec<MarketEvent>> {
        let response: SecSubmissionsResponse = serde_json::from_str(response_json)
            .with_context(|| format!("failed to parse SEC submissions for {symbol}"))?;
        let target_forms: HashSet<&str> = event_data::SEC_TARGET_FORMS.iter().copied().collect();
        let start_date = end_date - ChronoDuration::days(lookback_calendar_days);
        let company_name = response.name.unwrap_or_else(|| symbol.to_string());
        let recent = response.filings.recent;
        let mut events = Vec::new();

        for idx in 0..recent.form.len() {
            let form = recent.form[idx].trim();
            if !target_forms.contains(form) {
                continue;
            }

            let Some(filing_date) = string_at(&recent.filing_date, idx) else {
                continue;
            };
            let Ok(parsed_date) = NaiveDate::parse_from_str(&filing_date, "%Y-%m-%d") else {
                continue;
            };
            if parsed_date < start_date || parsed_date > end_date {
                continue;
            }

            let Some(accession_number) = string_at(&recent.accession_number, idx) else {
                continue;
            };
            let primary_document = string_at(&recent.primary_document, idx);
            let report_date = string_at(&recent.report_date, idx);
            let acceptance_time = string_at(&recent.acceptance_datetime, idx);
            let source_event_id = format!(
                "{}:{:010}:{}",
                event_data::SEC_SOURCE_NAME,
                cik,
                accession_number
            );
            let url = primary_document
                .as_ref()
                .map(|document| filing_url(cik, &accession_number, document));
            let raw_json = json!({
                "symbol": symbol,
                "cik": cik,
                "accessionNumber": &accession_number,
                "filingDate": &filing_date,
                "reportDate": &report_date,
                "acceptanceDateTime": &acceptance_time,
                "form": form,
                "primaryDocument": &primary_document
            })
            .to_string();

            events.push(MarketEvent {
                symbol: symbol.to_string(),
                sector: None,
                event_date: filing_date,
                event_type: event_data::EVENT_TYPE_FILING.to_string(),
                headline: format!("{form} filed by {company_name}"),
                source: event_data::SEC_SOURCE_NAME.to_string(),
                url,
                metadata: MarketEventMetadata {
                    event_time: acceptance_time.clone(),
                    source_event_id: Some(source_event_id),
                    effective_date: report_date,
                    processed_at: acceptance_time,
                    fetched_at: Some(Utc::now().to_rfc3339()),
                    raw_json: Some(raw_json),
                    ..MarketEventMetadata::default()
                },
            });
        }

        Ok(events)
    }

    fn fetch_company_tickers(&self) -> Result<HashMap<String, SecCompany>> {
        let response = self
            .client
            .get(event_data::SEC_COMPANY_TICKERS_URL)
            .header(ACCEPT, "application/json")
            .send()
            .context("failed to fetch SEC company tickers")?
            .error_for_status()
            .context("SEC company tickers request failed")?
            .text()
            .context("failed to read SEC company tickers response")?;

        Self::parse_company_tickers_json(&response)
    }

    fn fetch_submissions(&self, company: &SecCompany) -> Result<String> {
        let response = self
            .client
            .get(format!(
                "{}/CIK{:010}.json",
                event_data::SEC_SUBMISSIONS_URL,
                company.cik
            ))
            .header(ACCEPT, "application/json")
            .header("User-Agent", &self.user_agent)
            .send()
            .with_context(|| format!("failed to fetch SEC submissions for {}", company.ticker))?
            .error_for_status()
            .with_context(|| format!("SEC submissions request failed for {}", company.ticker))?
            .text()
            .with_context(|| format!("failed to read SEC submissions for {}", company.ticker))?;

        Ok(response)
    }
}

impl FilingEventProvider for SecEdgarProvider {
    fn recent_filing_events(
        &self,
        symbols: &[String],
        end_date: NaiveDate,
    ) -> Result<Vec<MarketEvent>> {
        let companies = self.fetch_company_tickers()?;
        let mut events = Vec::new();

        for symbol in symbols {
            let normalized = symbol.to_uppercase();
            let sec_ticker = normalized.replace('.', "-");
            let Some(company) = companies
                .get(normalized.as_str())
                .or_else(|| companies.get(sec_ticker.as_str()))
            else {
                continue;
            };
            let response = self.fetch_submissions(company)?;
            events.extend(Self::parse_submissions_json(
                &normalized,
                company.cik,
                &response,
                end_date,
                self.lookback_calendar_days,
            )?);
            thread::sleep(event_data::sec_request_sleep());
        }

        Ok(events)
    }
}

#[derive(Debug, Deserialize)]
struct SecCompanyTickerRow {
    cik_str: u64,
    ticker: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct SecSubmissionsResponse {
    #[serde(default)]
    name: Option<String>,
    filings: SecFilings,
}

#[derive(Debug, Deserialize)]
struct SecFilings {
    recent: SecRecentFilings,
}

#[derive(Debug, Deserialize)]
struct SecRecentFilings {
    #[serde(rename = "accessionNumber", default)]
    accession_number: Vec<String>,
    #[serde(rename = "filingDate", default)]
    filing_date: Vec<String>,
    #[serde(rename = "reportDate", default)]
    report_date: Vec<String>,
    #[serde(rename = "acceptanceDateTime", default)]
    acceptance_datetime: Vec<String>,
    #[serde(default)]
    form: Vec<String>,
    #[serde(rename = "primaryDocument", default)]
    primary_document: Vec<String>,
}

fn string_at(values: &[String], idx: usize) -> Option<String> {
    values
        .get(idx)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn filing_url(cik: u64, accession_number: &str, primary_document: &str) -> String {
    let accession_path = accession_number.replace('-', "");
    format!(
        "{}/{}/{}/{}",
        event_data::SEC_ARCHIVES_URL,
        cik,
        accession_path,
        primary_document
    )
}
