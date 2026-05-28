use std::env;

use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, NaiveDate};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;

use crate::config::{USER_AGENT, macro_data};
use crate::domain::models::MacroObservation;

use super::provider::MacroSeriesProvider;

pub struct FredProvider {
    api_key: String,
    base_url: String,
    lookback_calendar_days: i64,
    client: Client,
}

impl FredProvider {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var(macro_data::FRED_API_KEY_ENV).with_context(|| {
            format!(
                "missing {}; create a free FRED API key and export it before running Phase 5 macro ingestion",
                macro_data::FRED_API_KEY_ENV
            )
        })?;
        let base_url = env::var(macro_data::FRED_API_URL_ENV)
            .unwrap_or_else(|_| macro_data::FRED_API_URL.to_string());
        let lookback_calendar_days = env::var(macro_data::MACRO_LOOKBACK_DAYS_ENV)
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(macro_data::DEFAULT_MACRO_LOOKBACK_DAYS);
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(macro_data::http_timeout())
            .build()
            .context("failed to create FRED HTTP client")?;

        Ok(Self {
            api_key,
            base_url,
            lookback_calendar_days,
            client,
        })
    }

    pub fn env_status() -> Vec<String> {
        if env::var(macro_data::FRED_API_KEY_ENV).is_ok() {
            vec!["ok: FRED API key is set".to_string()]
        } else {
            vec![format!(
                "missing: FRED API key ({})",
                macro_data::FRED_API_KEY_ENV
            )]
        }
    }

    pub fn parse_observations_json(
        series_id: &str,
        series_name: &str,
        frequency: &str,
        units: &str,
        response_json: &str,
    ) -> Result<Vec<MacroObservation>> {
        let response: FredSeriesResponse = serde_json::from_str(response_json)
            .with_context(|| format!("failed to parse FRED response for {series_id}"))?;
        response_observations(series_id, series_name, frequency, units, response)
    }

    fn fetch_series(
        &self,
        series_id: &str,
        series_name: &str,
        frequency: &str,
        units: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<MacroObservation>> {
        let response = self
            .client
            .get(format!(
                "{}{}",
                self.base_url,
                macro_data::FRED_SERIES_OBSERVATIONS_PATH
            ))
            .query(&[
                ("series_id", series_id.to_string()),
                ("api_key", self.api_key.clone()),
                ("file_type", macro_data::FRED_FILE_TYPE_JSON.to_string()),
                (
                    "observation_start",
                    start_date.format("%Y-%m-%d").to_string(),
                ),
                ("observation_end", end_date.format("%Y-%m-%d").to_string()),
                ("sort_order", macro_data::FRED_SORT_ASC.to_string()),
            ])
            .send()
            .with_context(|| format!("failed to fetch FRED series {series_id}"))?
            .error_for_status()
            .with_context(|| format!("FRED request failed for series {series_id}"))?
            .json::<FredSeriesResponse>()
            .with_context(|| format!("failed to parse FRED series {series_id} response"))?;

        response_observations(series_id, series_name, frequency, units, response)
    }
}

impl MacroSeriesProvider for FredProvider {
    fn macro_observations(&self, end_date: NaiveDate) -> Result<Vec<MacroObservation>> {
        let start_date = end_date - ChronoDuration::days(self.lookback_calendar_days);
        let mut observations = Vec::new();

        for (series_id, series_name, frequency, units) in macro_data::MACRO_SERIES {
            observations.extend(self.fetch_series(
                series_id,
                series_name,
                frequency,
                units,
                start_date,
                end_date,
            )?);
        }

        Ok(observations)
    }
}

fn response_observations(
    series_id: &str,
    series_name: &str,
    frequency: &str,
    units: &str,
    response: FredSeriesResponse,
) -> Result<Vec<MacroObservation>> {
    let mut observations = Vec::new();

    for observation in response.observations {
        if observation.value == "." {
            continue;
        }
        let value = observation.value.parse::<f64>().with_context(|| {
            format!("invalid FRED value for {series_id} on {}", observation.date)
        })?;
        let raw_json = json!({
            "series_id": series_id,
            "realtime_start": observation.realtime_start,
            "realtime_end": observation.realtime_end,
            "date": observation.date,
            "value": observation.value
        })
        .to_string();

        observations.push(MacroObservation {
            series: series_id.to_string(),
            series_name: series_name.to_string(),
            date: observation.date,
            value,
            source: format!("{}:{series_id}", macro_data::SOURCE_NAME),
            frequency: frequency.to_string(),
            units: units.to_string(),
            realtime_start: observation.realtime_start,
            realtime_end: observation.realtime_end,
            raw_json,
            quality_status: macro_data::QUALITY_OK.to_string(),
        });
    }

    Ok(observations)
}

#[derive(Debug, Deserialize)]
struct FredSeriesResponse {
    observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
struct FredObservation {
    realtime_start: String,
    realtime_end: String,
    date: String,
    value: String,
}
