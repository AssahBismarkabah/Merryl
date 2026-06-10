export interface DashboardSnapshot {
  score_date: string;
  limitations: string[];
  regime: Regime | null;
  sectors: Sector[];
  industries: Industry[];
  stocks: Stock[];
  watchlist: WatchlistRow[];
  intraday_setups: IntradaySetup[];
  intraday_triggers: IntradayTrigger[];
  latest_backtest: Backtest | null;
  data_health: DataHealth;
}

export interface Regime {
  label: string;
  score: number;
  spy_return_20d: number;
  spy_return_60d: number;
  qqq_relative_return_vs_spy: number;
  iwm_relative_return_vs_spy: number;
  dia_relative_return_vs_spy: number;
  tlt_return_20d: number;
  gld_return_20d: number;
  uso_return_20d: number;
  explanation: string;
}

export interface Sector {
  rank: number;
  sector: string;
  sector_etf: string;
  score: number;
  return_1d: number;
  return_5d: number;
  return_20d: number;
  return_60d: number;
  relative_return_vs_spy: number;
  relative_volume: number;
  breadth_20d: number;
  breadth_50d: number;
  rank_change: number;
}

export interface Industry {
  rank: number;
  industry: string;
  sector: string;
  score: number;
  return_5d: number;
  return_20d: number;
  return_60d: number;
  relative_return_vs_sector: number;
  relative_volume: number;
  member_count: number;
}

export interface Stock {
  rank: number;
  symbol: string;
  name: string;
  sector: string;
  industry: string;
  score: number;
  sector_score: number;
  return_1d: number;
  return_5d: number;
  return_20d: number;
  return_60d: number;
  relative_return_vs_sector: number;
  relative_return_vs_spy: number;
  relative_volume: number;
  avg_dollar_volume: number;
  trend_state: string;
  catalyst_status: string;
  primary_actionability: string;
  actionability_labels: string[];
  distance_from_20d_ma_pct: number;
  distance_from_50d_ma_pct: number;
  atr_extension_from_20d_ma: number;
  distance_from_20d_high_pct: number;
}

export interface WatchlistRow {
  rank: number;
  symbol: string;
  name: string;
  sector: string;
  industry: string;
  score: number;
  catalyst_status: string;
  classifications: string[];
  primary_actionability: string;
  actionability_labels: string[];
  distance_from_20d_ma_pct: number;
  distance_from_50d_ma_pct: number;
  atr_extension_from_20d_ma: number;
  distance_from_20d_high_pct: number;
  reason: string;
}

export interface IntradaySetup {
  date: string;
  symbol: string;
  name: string;
  sector: string;
  industry: string;
  direction: string;
  primary_label: string;
  stage1_passed: boolean;
  stage2_passed: boolean;
  stage3_passed: boolean;
  adr_pct: number;
  rvol_ratio: number;
  mansfield_rs_spy: number;
  mansfield_rs_sector: number;
  ema_10: number;
  ema_20: number;
  latest_price: number;
  confluence_count: number;
  confluence: string[];
  trigger_count: number;
}

export interface IntradayTrigger {
  date: string;
  symbol: string;
  ts: string;
  timeframe: string;
  trigger_type: string;
  direction: string;
  trigger_price: number;
  reference_level: number;
  volume_spike: number;
  price_action: string;
}

export interface Backtest {
  id: number;
  run_name: string;
  from_date: string;
  to_date: string;
  created_at: string;
  metrics: {
    validation_scope?: {
      purpose?: string;
      hit_rate_definition?: string;
    };
    sector_observation_count?: number;
    stock_observation_count?: number;
    industry_stock_observation_count?: number;
  };
}

export interface DataHealth {
  database_path: string;
  latest_benchmark_price_date: string | null;
  latest_score_date: string | null;
  score_dates: number;
  required_symbol_count: number;
  missing_symbols: string[];
  missing_sector_maps: string[];
  required_price_coverage: PriceCoverage[];
  required_macro_coverage: MacroCoverage[];
  latest_score_coverage: LatestScoreCoverage;
}

export interface PriceCoverage {
  symbol: string;
  bar_count: number;
  first_date: string | null;
  latest_date: string | null;
}

export interface MacroCoverage {
  series: string;
  observation_count: number;
  first_date: string | null;
  latest_date: string | null;
}

export interface LatestScoreCoverage {
  market_regime_rows: number;
  sector_rows: number;
  industry_rows: number;
  stock_rows: number;
  watchlist_rows: number;
}
