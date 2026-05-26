use std::collections::HashMap;
use std::env;
use std::thread;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::config::{USER_AGENT, market_data};
use crate::domain::models::{DailyPrice, IndustryMap, SectorMap, Symbol};

use super::provider::DailyOhlcvProvider;
use super::sector_map::sector_maps;
use super::universe::{etf_symbols, fetch_sp500_symbols, industry_maps};

pub struct AlpacaProvider {
    key_id: String,
    secret_key: String,
    feed: String,
    base_url: String,
    lookback_calendar_days: i64,
    client: Client,
}

impl AlpacaProvider {
    pub fn from_env() -> Result<Self> {
        let key_id = env::var(market_data::ALPACA_API_KEY_ID_ENV).with_context(|| {
            format!(
                "missing {}; create a free Alpaca market-data key or export an existing one",
                market_data::ALPACA_API_KEY_ID_ENV
            )
        })?;
        let secret_key = env::var(market_data::ALPACA_API_SECRET_KEY_ENV).with_context(|| {
            format!(
                "missing {}; create a free Alpaca market-data key or export an existing one",
                market_data::ALPACA_API_SECRET_KEY_ENV
            )
        })?;
        let feed = env::var(market_data::ALPACA_FEED_ENV)
            .unwrap_or_else(|_| market_data::DEFAULT_ALPACA_FEED.to_string());
        let base_url = env::var(market_data::ALPACA_DATA_URL_ENV)
            .unwrap_or_else(|_| market_data::ALPACA_DATA_URL.to_string());
        let lookback_calendar_days = env::var(market_data::LOOKBACK_DAYS_ENV)
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(market_data::DEFAULT_LOOKBACK_DAYS);
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(market_data::http_timeout())
            .build()
            .context("failed to create HTTP client")?;

        Ok(Self {
            key_id,
            secret_key,
            feed,
            base_url,
            lookback_calendar_days,
            client,
        })
    }

    pub fn env_status() -> Vec<String> {
        [
            (market_data::ALPACA_API_KEY_ID_ENV, "Alpaca API key id"),
            (
                market_data::ALPACA_API_SECRET_KEY_ENV,
                "Alpaca API secret key",
            ),
        ]
        .into_iter()
        .map(|(name, label)| {
            if env::var(name).is_ok() {
                format!("ok: {label} is set")
            } else {
                format!("missing: {label} ({name})")
            }
        })
        .collect()
    }

    fn fetch_batch(
        &self,
        symbols: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyPrice>> {
        let mut all_prices = Vec::new();
        let mut next_page_token: Option<String> = None;

        loop {
            let query = self.bars_query(symbols, start_date, end_date, &next_page_token);
            let response = self
                .client
                .get(format!(
                    "{}{}",
                    self.base_url,
                    market_data::ALPACA_BARS_PATH
                ))
                .header(market_data::ALPACA_KEY_HEADER, &self.key_id)
                .header(market_data::ALPACA_SECRET_HEADER, &self.secret_key)
                .query(&query)
                .send()
                .with_context(|| {
                    format!(
                        "failed to fetch Alpaca daily bars for {}",
                        symbols.join(",")
                    )
                })?
                .error_for_status()
                .with_context(|| {
                    format!("Alpaca daily bars request failed for {}", symbols.join(","))
                })?
                .json::<AlpacaBarsResponse>()
                .context("failed to parse Alpaca daily bars response")?;

            all_prices.extend(self.response_prices(response.bars)?);

            match response.next_page_token {
                Some(token) if !token.is_empty() => {
                    next_page_token = Some(token);
                }
                _ => break,
            }
        }

        Ok(all_prices)
    }

    fn bars_query(
        &self,
        symbols: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
        next_page_token: &Option<String>,
    ) -> Vec<(String, String)> {
        let mut query = vec![
            ("symbols".to_string(), symbols.join(",")),
            (
                "timeframe".to_string(),
                market_data::DAILY_TIMEFRAME.to_string(),
            ),
            (
                "start".to_string(),
                start_date.format("%Y-%m-%d").to_string(),
            ),
            ("end".to_string(), end_date.format("%Y-%m-%d").to_string()),
            (
                "limit".to_string(),
                market_data::ALPACA_PAGE_LIMIT.to_string(),
            ),
            (
                "adjustment".to_string(),
                market_data::PRICE_ADJUSTMENT.to_string(),
            ),
            ("feed".to_string(), self.feed.clone()),
            ("sort".to_string(), market_data::SORT_ASC.to_string()),
        ];

        if let Some(token) = next_page_token {
            query.push(("page_token".to_string(), token.clone()));
        }

        query
    }

    fn response_prices(
        &self,
        bars_by_symbol: HashMap<String, Vec<AlpacaBar>>,
    ) -> Result<Vec<DailyPrice>> {
        let mut prices = Vec::new();
        for (symbol, bars) in bars_by_symbol {
            for bar in bars {
                prices.push(DailyPrice {
                    symbol: symbol.clone(),
                    date: alpaca_date(&bar.t)?,
                    open: bar.o,
                    high: bar.h,
                    low: bar.l,
                    close: bar.c,
                    adjusted_close: bar.c,
                    volume: bar.v,
                    source: format!("{}:{}", market_data::SOURCE_PREFIX, self.feed),
                });
            }
        }

        Ok(prices)
    }
}

impl DailyOhlcvProvider for AlpacaProvider {
    fn symbols(&self) -> Result<Vec<Symbol>> {
        let mut symbols = etf_symbols();
        symbols.extend(fetch_sp500_symbols(&self.client)?);
        symbols.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        symbols.dedup_by(|a, b| a.symbol == b.symbol);
        Ok(symbols)
    }

    fn sector_maps(&self) -> Vec<SectorMap> {
        sector_maps()
    }

    fn industry_maps(&self, symbols: &[Symbol]) -> Vec<IndustryMap> {
        industry_maps(symbols)
    }

    fn daily_prices(&self, symbols: &[Symbol], end_date: NaiveDate) -> Result<Vec<DailyPrice>> {
        let start_date = end_date - ChronoDuration::days(self.lookback_calendar_days);
        let symbol_values: Vec<String> = symbols
            .iter()
            .filter(|symbol| symbol.is_active)
            .map(|symbol| symbol.symbol.clone())
            .collect();
        let mut prices = Vec::new();

        for batch in symbol_values.chunks(market_data::ALPACA_BATCH_SIZE) {
            prices.extend(self.fetch_batch(batch, start_date, end_date)?);
            thread::sleep(market_data::batch_sleep());
        }

        Ok(prices)
    }
}

fn alpaca_date(timestamp: &str) -> Result<String> {
    let parsed = DateTime::parse_from_rfc3339(timestamp)
        .with_context(|| format!("invalid Alpaca timestamp: {timestamp}"))?;
    Ok(parsed.date_naive().format("%Y-%m-%d").to_string())
}

pub fn default_end_date() -> NaiveDate {
    Utc::now().date_naive() + ChronoDuration::days(1)
}

#[derive(Debug, Deserialize)]
struct AlpacaBarsResponse {
    bars: HashMap<String, Vec<AlpacaBar>>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlpacaBar {
    t: String,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: f64,
}
