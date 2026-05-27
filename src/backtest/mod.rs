use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::Serialize;

use crate::config::scoring;
use crate::domain::models::{
    DailyPrice, IndustryScoreSnapshot, SectorMap, SectorScore, StockScore,
};

const ENTITY_SECTOR: &str = "sector";
const ENTITY_STOCK: &str = "stock";
const ENTITY_STOCK_BY_INDUSTRY: &str = "stock_by_industry";

#[derive(Debug, Clone)]
pub struct BacktestInput {
    pub from_date: String,
    pub to_date: String,
    pub sector_scores: Vec<SectorScore>,
    pub industry_scores: Vec<IndustryScoreSnapshot>,
    pub stock_scores: Vec<StockScore>,
    pub sector_maps: Vec<SectorMap>,
    pub prices: Vec<DailyPrice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacktestSummaryRow {
    pub entity_type: String,
    pub horizon: usize,
    pub decile: usize,
    pub count: usize,
    pub hit_rate: f64,
    pub average_forward_return: f64,
    pub median_forward_return: f64,
    pub average_relative_return: f64,
    pub median_relative_return: f64,
    pub average_relative_return_vs_spy: f64,
    pub median_relative_return_vs_spy: f64,
    pub average_relative_return_vs_sector: Option<f64>,
    pub median_relative_return_vs_sector: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacktestMetrics {
    pub from_date: String,
    pub to_date: String,
    pub sector_observation_count: usize,
    pub stock_observation_count: usize,
    pub industry_stock_observation_count: usize,
    pub summaries: Vec<BacktestSummaryRow>,
}

#[derive(Debug, Clone)]
struct Observation {
    entity_type: &'static str,
    horizon: usize,
    decile: usize,
    forward_return: f64,
    relative_return: f64,
    relative_return_vs_spy: f64,
    relative_return_vs_sector: Option<f64>,
}

pub fn run_backtest_analysis(input: BacktestInput) -> Result<BacktestMetrics> {
    if input.sector_scores.is_empty()
        || input.industry_scores.is_empty()
        || input.stock_scores.is_empty()
        || input.sector_maps.is_empty()
        || input.prices.is_empty()
    {
        bail!("missing historical scores or prices; run `merryl run daily --date latest` first");
    }

    let histories = histories_by_symbol(&input.prices);
    let sector_etfs: HashMap<&str, &str> = input
        .sector_maps
        .iter()
        .map(|sector_map| (sector_map.sector.as_str(), sector_map.sector_etf.as_str()))
        .collect();

    let mut observations = Vec::new();
    observations.extend(sector_observations(&input.sector_scores, &histories));
    observations.extend(stock_observations(
        &input.stock_scores,
        &histories,
        &sector_etfs,
    ));
    observations.extend(stock_by_industry_observations(
        &input.stock_scores,
        &input.industry_scores,
        &histories,
        &sector_etfs,
    ));

    if observations.is_empty() {
        bail!("no valid backtest observations had enough future price bars");
    }

    let sector_observation_count = observations
        .iter()
        .filter(|observation| observation.entity_type == ENTITY_SECTOR)
        .count();
    let stock_observation_count = observations
        .iter()
        .filter(|observation| observation.entity_type == ENTITY_STOCK)
        .count();
    let industry_stock_observation_count = observations
        .iter()
        .filter(|observation| observation.entity_type == ENTITY_STOCK_BY_INDUSTRY)
        .count();
    let summaries = summarize_observations(&observations);

    Ok(BacktestMetrics {
        from_date: input.from_date,
        to_date: input.to_date,
        sector_observation_count,
        stock_observation_count,
        industry_stock_observation_count,
        summaries,
    })
}

fn sector_observations(
    scores: &[SectorScore],
    histories: &HashMap<String, Vec<DailyPrice>>,
) -> Vec<Observation> {
    let mut observations = Vec::new();
    for daily_scores in sector_scores_by_date(scores).values() {
        for (score, decile) in ranked_deciles(daily_scores) {
            for horizon in scoring::BACKTEST_HORIZONS {
                let Some(entity_forward_return) =
                    forward_return(histories, &score.sector_etf, &score.date, *horizon)
                else {
                    continue;
                };
                let Some(spy_return) =
                    forward_return(histories, scoring::BENCHMARK_SYMBOL, &score.date, *horizon)
                else {
                    continue;
                };
                observations.push(Observation {
                    entity_type: ENTITY_SECTOR,
                    horizon: *horizon,
                    decile,
                    forward_return: entity_forward_return,
                    relative_return: entity_forward_return - spy_return,
                    relative_return_vs_spy: entity_forward_return - spy_return,
                    relative_return_vs_sector: None,
                });
            }
        }
    }
    observations
}

fn stock_observations(
    scores: &[StockScore],
    histories: &HashMap<String, Vec<DailyPrice>>,
    sector_etfs: &HashMap<&str, &str>,
) -> Vec<Observation> {
    let mut observations = Vec::new();
    for daily_scores in stock_scores_by_date(scores).values() {
        for (score, decile) in ranked_deciles(daily_scores) {
            observations.extend(stock_forward_observations(
                ENTITY_STOCK,
                score,
                decile,
                histories,
                sector_etfs,
            ));
        }
    }
    observations
}

fn stock_by_industry_observations(
    scores: &[StockScore],
    industry_scores: &[IndustryScoreSnapshot],
    histories: &HashMap<String, Vec<DailyPrice>>,
    sector_etfs: &HashMap<&str, &str>,
) -> Vec<Observation> {
    let industry_deciles = industry_deciles_by_date(industry_scores);
    let mut observations = Vec::new();
    for score in scores {
        let Some(decile) =
            industry_deciles.get(&industry_key(&score.date, &score.sector, &score.industry))
        else {
            continue;
        };
        observations.extend(stock_forward_observations(
            ENTITY_STOCK_BY_INDUSTRY,
            score,
            *decile,
            histories,
            sector_etfs,
        ));
    }
    observations
}

fn stock_forward_observations(
    entity_type: &'static str,
    score: &StockScore,
    decile: usize,
    histories: &HashMap<String, Vec<DailyPrice>>,
    sector_etfs: &HashMap<&str, &str>,
) -> Vec<Observation> {
    let Some(sector_etf) = sector_etfs.get(score.sector.as_str()) else {
        return Vec::new();
    };

    let mut observations = Vec::new();
    for horizon in scoring::BACKTEST_HORIZONS {
        let Some(entity_forward_return) =
            forward_return(histories, &score.symbol, &score.date, *horizon)
        else {
            continue;
        };
        let Some(sector_return) = forward_return(histories, sector_etf, &score.date, *horizon)
        else {
            continue;
        };
        let Some(spy_return) =
            forward_return(histories, scoring::BENCHMARK_SYMBOL, &score.date, *horizon)
        else {
            continue;
        };
        observations.push(Observation {
            entity_type,
            horizon: *horizon,
            decile,
            forward_return: entity_forward_return,
            relative_return: entity_forward_return - sector_return,
            relative_return_vs_spy: entity_forward_return - spy_return,
            relative_return_vs_sector: Some(entity_forward_return - sector_return),
        });
    }
    observations
}

fn summarize_observations(observations: &[Observation]) -> Vec<BacktestSummaryRow> {
    let mut groups: HashMap<(&str, usize, usize), Vec<&Observation>> = HashMap::new();
    for observation in observations {
        groups
            .entry((
                observation.entity_type,
                observation.horizon,
                observation.decile,
            ))
            .or_default()
            .push(observation);
    }

    let mut summaries: Vec<BacktestSummaryRow> = groups
        .into_iter()
        .map(|((entity_type, horizon, decile), rows)| {
            let forward_returns: Vec<f64> = rows.iter().map(|row| row.forward_return).collect();
            let relative_returns: Vec<f64> = rows.iter().map(|row| row.relative_return).collect();
            let relative_returns_vs_spy: Vec<f64> =
                rows.iter().map(|row| row.relative_return_vs_spy).collect();
            let relative_returns_vs_sector: Vec<f64> = rows
                .iter()
                .filter_map(|row| row.relative_return_vs_sector)
                .collect();
            let wins = relative_returns
                .iter()
                .filter(|relative_return| **relative_return > 0.0)
                .count();
            BacktestSummaryRow {
                entity_type: entity_type.to_string(),
                horizon,
                decile,
                count: rows.len(),
                hit_rate: wins as f64 / rows.len() as f64,
                average_forward_return: average(&forward_returns),
                median_forward_return: median(forward_returns),
                average_relative_return: average(&relative_returns),
                median_relative_return: median(relative_returns),
                average_relative_return_vs_spy: average(&relative_returns_vs_spy),
                median_relative_return_vs_spy: median(relative_returns_vs_spy),
                average_relative_return_vs_sector: optional_average(&relative_returns_vs_sector),
                median_relative_return_vs_sector: optional_median(relative_returns_vs_sector),
            }
        })
        .collect();

    summaries.sort_by(|a, b| {
        a.entity_type
            .cmp(&b.entity_type)
            .then(a.horizon.cmp(&b.horizon))
            .then(b.decile.cmp(&a.decile))
    });
    summaries
}

fn histories_by_symbol(prices: &[DailyPrice]) -> HashMap<String, Vec<DailyPrice>> {
    let mut histories: HashMap<String, Vec<DailyPrice>> = HashMap::new();
    for price in prices {
        histories
            .entry(price.symbol.clone())
            .or_default()
            .push(price.clone());
    }
    for history in histories.values_mut() {
        history.sort_by(|a, b| a.date.cmp(&b.date));
    }
    histories
}

fn sector_scores_by_date(scores: &[SectorScore]) -> HashMap<String, Vec<&SectorScore>> {
    let mut by_date: HashMap<String, Vec<&SectorScore>> = HashMap::new();
    for score in scores {
        by_date.entry(score.date.clone()).or_default().push(score);
    }
    by_date
}

fn stock_scores_by_date(scores: &[StockScore]) -> HashMap<String, Vec<&StockScore>> {
    let mut by_date: HashMap<String, Vec<&StockScore>> = HashMap::new();
    for score in scores {
        by_date.entry(score.date.clone()).or_default().push(score);
    }
    by_date
}

fn industry_scores_by_date(
    scores: &[IndustryScoreSnapshot],
) -> HashMap<String, Vec<&IndustryScoreSnapshot>> {
    let mut by_date: HashMap<String, Vec<&IndustryScoreSnapshot>> = HashMap::new();
    for score in scores {
        by_date.entry(score.date.clone()).or_default().push(score);
    }
    by_date
}

fn industry_deciles_by_date(
    scores: &[IndustryScoreSnapshot],
) -> HashMap<(String, String, String), usize> {
    let mut deciles = HashMap::new();
    for daily_scores in industry_scores_by_date(scores).values() {
        for (score, decile) in ranked_deciles(daily_scores) {
            deciles.insert(
                industry_key(&score.date, &score.sector, &score.industry),
                decile,
            );
        }
    }
    deciles
}

fn industry_key(date: &str, sector: &str, industry: &str) -> (String, String, String) {
    (date.to_string(), sector.to_string(), industry.to_string())
}

fn ranked_deciles<T: ScoreValue>(scores: &[T]) -> Vec<(&T, usize)> {
    let mut ranked: Vec<&T> = scores.iter().collect();
    ranked.sort_by(|a, b| a.score().total_cmp(&b.score()));
    let len = ranked.len();
    ranked
        .into_iter()
        .enumerate()
        .map(|(idx, score)| {
            let decile = (((idx + 1) * scoring::BACKTEST_DECILES).div_ceil(len))
                .clamp(1, scoring::BACKTEST_DECILES);
            (score, decile)
        })
        .collect()
}

fn forward_return(
    histories: &HashMap<String, Vec<DailyPrice>>,
    symbol: &str,
    date: &str,
    horizon: usize,
) -> Option<f64> {
    let history = histories.get(symbol)?;
    let idx = history.iter().position(|price| price.date == date)?;
    let future_idx = idx + horizon;
    if future_idx >= history.len() {
        return None;
    }
    let current = history[idx].adjusted_close;
    let future = history[future_idx].adjusted_close;
    Some((future / current) - 1.0)
}

fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn optional_average(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| average(values))
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn optional_median(values: Vec<f64>) -> Option<f64> {
    (!values.is_empty()).then(|| median(values))
}

trait ScoreValue {
    fn score(&self) -> f64;
}

impl ScoreValue for &SectorScore {
    fn score(&self) -> f64 {
        self.score
    }
}

impl ScoreValue for &StockScore {
    fn score(&self) -> f64 {
        self.score
    }
}

impl ScoreValue for &IndustryScoreSnapshot {
    fn score(&self) -> f64 {
        self.score
    }
}
