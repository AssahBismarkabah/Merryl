use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub exchange: String,
    pub market_cap: Option<f64>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct DailyPrice {
    pub symbol: String,
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adjusted_close: f64,
    pub volume: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct MacroObservation {
    pub series: String,
    pub series_name: String,
    pub date: String,
    pub value: f64,
    pub source: String,
    pub frequency: String,
    pub units: String,
    pub realtime_start: String,
    pub realtime_end: String,
    pub raw_json: String,
    pub quality_status: String,
}

#[derive(Debug, Clone)]
pub struct MarketEvent {
    pub symbol: String,
    pub sector: Option<String>,
    pub event_date: String,
    pub event_type: String,
    pub headline: String,
    pub source: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SectorMap {
    pub sector: String,
    pub sector_etf: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct IndustryMap {
    pub industry: String,
    pub sector: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct MarketRegimeScore {
    pub date: String,
    pub label: String,
    pub score: f64,
    pub spy_return_20d: f64,
    pub spy_return_60d: f64,
    pub qqq_relative_return_vs_spy: f64,
    pub iwm_relative_return_vs_spy: f64,
    pub dia_relative_return_vs_spy: f64,
    pub components_json: String,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct WatchlistRow {
    pub date: String,
    pub rank: usize,
    pub symbol: String,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct BacktestResultRow {
    pub id: i64,
    pub run_name: String,
    pub from_date: String,
    pub to_date: String,
    pub config_json: String,
    pub metrics_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectorScore {
    pub date: String,
    pub sector: String,
    pub sector_etf: String,
    pub score: f64,
    pub rank: usize,
    pub return_1d: f64,
    pub return_5d: f64,
    pub return_20d: f64,
    pub return_60d: f64,
    pub relative_return_vs_spy: f64,
    pub relative_volume: f64,
    pub breadth_20d: f64,
    pub breadth_50d: f64,
    pub rank_change: f64,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct IndustryScore {
    pub date: String,
    pub industry: String,
    pub sector: String,
    pub score: f64,
    pub rank: usize,
    pub return_5d: f64,
    pub return_20d: f64,
    pub return_60d: f64,
    pub relative_return_vs_sector: f64,
    pub relative_return_vs_spy: f64,
    pub relative_volume: f64,
    pub breadth_20d: f64,
    pub breadth_50d: f64,
    pub high_20d_rate: f64,
    pub member_count: usize,
    pub components_json: String,
}

#[derive(Debug, Clone)]
pub struct IndustryScoreSnapshot {
    pub date: String,
    pub industry: String,
    pub sector: String,
    pub score: f64,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockScore {
    pub date: String,
    pub rank: usize,
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub score: f64,
    pub sector_score: f64,
    pub return_1d: f64,
    pub return_5d: f64,
    pub return_20d: f64,
    pub return_60d: f64,
    pub relative_return_vs_sector: f64,
    pub relative_return_vs_spy: f64,
    pub relative_volume: f64,
    pub avg_dollar_volume: f64,
    pub trend_state: String,
    pub catalyst_status: String,
    #[serde(skip_serializing)]
    pub components_json: String,
    pub explanation: String,
}
