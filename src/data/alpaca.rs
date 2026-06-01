use std::collections::{HashMap, HashSet};
use std::env;
use std::thread;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::config::{USER_AGENT, market_data};
use crate::domain::models::{
    DailyPrice, IndustryMap, MarketEvent, MarketEventMetadata, SectorMap, Symbol,
};

use super::provider::{CatalystEventProvider, DailyOhlcvProvider};
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
            let response = self.get_json_with_retries::<AlpacaBarsResponse>(
                market_data::ALPACA_BARS_PATH,
                &query,
                &format!("Alpaca daily bars for {}", symbols.join(",")),
            )?;

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

    fn fetch_news_batch(
        &self,
        symbols: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<MarketEvent>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        let mut next_page_token: Option<String> = None;

        for _ in 0..market_data::ALPACA_NEWS_MAX_PAGES {
            let query = self.news_query(symbols, start_date, end_date, &next_page_token);
            let response = self.get_json_with_retries::<AlpacaNewsResponse>(
                market_data::ALPACA_NEWS_PATH,
                &query,
                &format!("Alpaca news for {}", symbols.join(",")),
            )?;

            events.extend(response_events(response.news, symbols)?);

            match response.next_page_token {
                Some(token) if !token.is_empty() => {
                    next_page_token = Some(token);
                }
                _ => break,
            }
        }

        Ok(events)
    }

    fn news_query(
        &self,
        symbols: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
        next_page_token: &Option<String>,
    ) -> Vec<(String, String)> {
        let mut query = vec![
            ("symbols".to_string(), symbols.join(",")),
            (
                "start".to_string(),
                start_date.format("%Y-%m-%d").to_string(),
            ),
            ("end".to_string(), end_date.format("%Y-%m-%d").to_string()),
            ("sort".to_string(), market_data::SORT_DESC.to_string()),
            (
                "limit".to_string(),
                market_data::ALPACA_NEWS_PAGE_LIMIT.to_string(),
            ),
            ("include_content".to_string(), "false".to_string()),
            ("exclude_contentless".to_string(), "false".to_string()),
        ];

        if let Some(token) = next_page_token {
            query.push(("page_token".to_string(), token.clone()));
        }

        query
    }

    fn get_json_with_retries<T>(
        &self,
        path: &str,
        query: &[(String, String)],
        label: &str,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut last_error = None;

        for attempt in 1..=market_data::ALPACA_REQUEST_ATTEMPTS {
            let result = self
                .client
                .get(format!("{}{}", self.base_url, path))
                .header(market_data::ALPACA_KEY_HEADER, &self.key_id)
                .header(market_data::ALPACA_SECRET_HEADER, &self.secret_key)
                .query(query)
                .send()
                .with_context(|| format!("failed to fetch {label}"))
                .and_then(|response| {
                    response
                        .error_for_status()
                        .with_context(|| format!("{label} request failed"))
                })
                .and_then(|response| {
                    response
                        .json::<T>()
                        .with_context(|| format!("failed to parse {label} response"))
                });

            match result {
                Ok(response) => return Ok(response),
                Err(err) if attempt < market_data::ALPACA_REQUEST_ATTEMPTS => {
                    last_error = Some(err);
                    thread::sleep(market_data::retry_sleep());
                }
                Err(err) => last_error = Some(err),
            }
        }

        Err(last_error.expect("Alpaca request attempted at least once")).with_context(|| {
            format!(
                "{label} failed after {} attempts",
                market_data::ALPACA_REQUEST_ATTEMPTS
            )
        })
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
        let batch_count = symbol_values.len().div_ceil(market_data::ALPACA_BATCH_SIZE);

        for (idx, batch) in symbol_values
            .chunks(market_data::ALPACA_BATCH_SIZE)
            .enumerate()
        {
            eprintln!(
                "progress: fetching Alpaca price batch {}/{} ({} symbols: {}..{})",
                idx + 1,
                batch_count,
                batch.len(),
                batch.first().map(String::as_str).unwrap_or("?"),
                batch.last().map(String::as_str).unwrap_or("?"),
            );
            prices.extend(self.fetch_batch(batch, start_date, end_date)?);
            thread::sleep(market_data::batch_sleep());
        }

        Ok(prices)
    }
}

impl CatalystEventProvider for AlpacaProvider {
    fn recent_news_events(
        &self,
        symbols: &[String],
        end_date: NaiveDate,
    ) -> Result<Vec<MarketEvent>> {
        let start_date = end_date - ChronoDuration::days(market_data::NEWS_LOOKBACK_DAYS);
        let request_end_date = end_date + ChronoDuration::days(1);
        self.fetch_news_batch(symbols, start_date, request_end_date)
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

#[derive(Debug, Deserialize)]
struct AlpacaNewsResponse {
    news: Vec<AlpacaNewsArticle>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlpacaNewsArticle {
    created_at: String,
    headline: String,
    source: String,
    url: Option<String>,
    #[serde(default)]
    symbols: Vec<String>,
}

fn response_events(
    articles: Vec<AlpacaNewsArticle>,
    requested_symbols: &[String],
) -> Result<Vec<MarketEvent>> {
    let requested: HashSet<&str> = requested_symbols.iter().map(String::as_str).collect();
    let mut events = Vec::new();

    for article in articles {
        let event_date = alpaca_date(&article.created_at)?;
        for symbol in article.symbols {
            if !requested.contains(symbol.as_str()) {
                continue;
            }
            let metadata = MarketEventMetadata {
                event_time: Some(article.created_at.clone()),
                fetched_at: Some(Utc::now().to_rfc3339()),
                raw_json: Some(
                    serde_json::json!({
                        "created_at": &article.created_at,
                        "headline": &article.headline,
                        "source": &article.source,
                        "symbol": &symbol,
                        "url": &article.url
                    })
                    .to_string(),
                ),
                ..MarketEventMetadata::default()
            };
            events.push(MarketEvent {
                symbol,
                sector: None,
                event_date: event_date.clone(),
                event_type: market_data::NEWS_EVENT_TYPE.to_string(),
                headline: article.headline.clone(),
                source: format!("{}:{}", market_data::NEWS_SOURCE_PREFIX, article.source),
                url: article.url.clone(),
                metadata,
            });
        }
    }

    Ok(events)
}
