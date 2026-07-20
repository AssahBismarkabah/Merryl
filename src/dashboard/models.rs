use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSnapshot {
    pub score_date: String,
    pub limitations: Vec<String>,
    pub regime: Option<RegimeDto>,
    pub sectors: Vec<SectorDto>,
    pub industries: Vec<IndustryDto>,
    pub stocks: Vec<StockDto>,
    pub watchlist: Vec<WatchlistDto>,
    pub intraday_setups: Vec<IntradaySetupDto>,
    pub intraday_triggers: Vec<IntradayTriggerDto>,
    pub latest_backtest: Option<BacktestDto>,
    pub data_health: DataHealthDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegimeDto {
    pub date: String,
    pub label: String,
    pub score: f64,
    pub spy_return_20d: f64,
    pub spy_return_60d: f64,
    pub qqq_relative_return_vs_spy: f64,
    pub iwm_relative_return_vs_spy: f64,
    pub dia_relative_return_vs_spy: f64,
    pub tlt_return_20d: f64,
    pub gld_return_20d: f64,
    pub uso_return_20d: f64,
    pub macro_context: Option<MacroContextDto>,
    pub components: Value,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroContextDto {
    pub date: String,
    pub active_flags: Vec<String>,
    pub stale_series: Vec<String>,
    pub covered_series_count: usize,
    pub required_series_count: usize,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectorDto {
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

#[derive(Debug, Clone, Serialize)]
pub struct IndustryDto {
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
    pub components: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockDto {
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
    pub primary_actionability: String,
    pub actionability_labels: Vec<String>,
    pub distance_from_20d_ma_pct: f64,
    pub distance_from_50d_ma_pct: f64,
    pub atr_extension_from_20d_ma: f64,
    pub distance_from_20d_high_pct: f64,
    pub components: Value,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WatchlistDto {
    pub date: String,
    pub rank: usize,
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub score: f64,
    pub catalyst_status: String,
    pub classifications: Vec<String>,
    pub primary_actionability: String,
    pub actionability_labels: Vec<String>,
    pub distance_from_20d_ma_pct: f64,
    pub distance_from_50d_ma_pct: f64,
    pub atr_extension_from_20d_ma: f64,
    pub distance_from_20d_high_pct: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntradaySetupDto {
    pub date: String,
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub direction: String,
    pub primary_label: String,
    pub stage1_passed: bool,
    pub stage2_passed: bool,
    pub stage3_passed: bool,
    pub adr_pct: f64,
    pub rvol_ratio: f64,
    pub mansfield_rs_spy: f64,
    pub mansfield_rs_sector: f64,
    pub ema_10: f64,
    pub ema_20: f64,
    pub latest_price: f64,
    pub confluence_count: usize,
    pub confluence: Value,
    pub trigger_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntradayTriggerDto {
    pub date: String,
    pub symbol: String,
    pub ts: String,
    pub timeframe: String,
    pub trigger_type: String,
    pub direction: String,
    pub trigger_price: f64,
    pub reference_level: f64,
    pub volume_spike: f64,
    pub price_action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacktestDto {
    pub id: i64,
    pub run_name: String,
    pub from_date: String,
    pub to_date: String,
    pub created_at: String,
    pub metrics: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataHealthDto {
    pub database_path: String,
    pub latest_benchmark_price_date: Option<String>,
    pub latest_score_date: Option<String>,
    pub score_dates: i64,
    pub required_symbol_count: usize,
    pub missing_symbols: Vec<String>,
    pub missing_sector_maps: Vec<String>,
    pub required_price_coverage: Vec<PriceCoverageDto>,
    pub required_macro_coverage: Vec<MacroCoverageDto>,
    pub latest_score_coverage: LatestScoreCoverageDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceCoverageDto {
    pub symbol: String,
    pub bar_count: i64,
    pub first_date: Option<String>,
    pub latest_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MacroCoverageDto {
    pub series: String,
    pub observation_count: i64,
    pub first_date: Option<String>,
    pub latest_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatestScoreCoverageDto {
    pub market_regime_rows: i64,
    pub sector_rows: i64,
    pub industry_rows: i64,
    pub stock_rows: i64,
    pub watchlist_rows: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthDto {
    pub status: String,
    pub database_path: String,
    pub latest_score_date: Option<String>,
    pub score_dates: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatesDto {
    pub dates: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScreenerResultDto {
    pub ticker: String,
    pub company: String,
    pub sector: String,
    pub industry: String,
    pub market_cap: String,
    pub pe_ratio: String,
    pub price: String,
    pub change: String,
    pub volume: String,
    pub dividend: String,
    pub roa: String,
    pub roe: String,
    pub debt_equity: String,
    pub net_profit_margin: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScreenerResponseDto {
    pub results: Vec<ScreenerResultDto>,
    pub sector: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorDto {
    pub message: String,
}
