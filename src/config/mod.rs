use std::path::PathBuf;
use std::time::Duration;

pub const APP_NAME: &str = "Merryl";
pub const CLI_NAME: &str = "merryl";
pub const USER_AGENT: &str = "Merryl/0.1";
pub const DAILY_RUN_COMMAND: &str = "merryl run daily --date latest";

pub mod dashboard {
    pub const DEFAULT_PORT: u16 = 8787;
    pub const HOST: &str = "127.0.0.1";
    pub const FRONTEND_DIST_DIR: &str = "dashboard/dist";
}

pub mod paths {
    use super::PathBuf;

    pub const DB_PATH: &str = "data/market.db";
    pub const REPORTS_DIR: &str = "reports";
    pub const EXPORTS_DIR: &str = "exports";
    pub const BACKTEST_REPORTS_DIR: &str = "reports/backtests";
    pub const BACKTEST_EXPORTS_DIR: &str = "exports/backtests";
    pub const INTRADAY_REPORTS_DIR: &str = "reports/intraday";
    pub const INTRADAY_EXPORTS_DIR: &str = "exports/intraday";
    pub const VALIDATION_REPORTS_DIR: &str = "reports/validations";
    pub const VALIDATION_EXPORTS_DIR: &str = "exports/validations";
    pub const DAILY_WORKFLOW_CONFIG: &str = "config/workflows/daily.toml";

    pub const REQUIRED_DOCS: &[&str] = &[
        "docs/market_rotation_system_spec.md",
        "docs/phase_0_decisions_spec.md",
        "docs/mvp_technical_plan_spec.md",
    ];

    pub fn db_path() -> PathBuf {
        PathBuf::from(DB_PATH)
    }

    pub fn report_path(date: &str) -> PathBuf {
        PathBuf::from(format!("{REPORTS_DIR}/{date}_market_report.md"))
    }

    pub fn sector_scores_export_path(date: &str) -> PathBuf {
        PathBuf::from(format!("{EXPORTS_DIR}/{date}_sector_scores.csv"))
    }

    pub fn stock_watchlist_export_path(date: &str) -> PathBuf {
        PathBuf::from(format!("{EXPORTS_DIR}/{date}_stock_watchlist.csv"))
    }

    pub fn backtest_report_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{BACKTEST_REPORTS_DIR}/{from_date}_{to_date}_backtest_report.md"
        ))
    }

    pub fn backtest_summary_export_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{BACKTEST_EXPORTS_DIR}/{from_date}_{to_date}_backtest_summary.csv"
        ))
    }

    pub fn macro_regime_validation_report_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{VALIDATION_REPORTS_DIR}/{from_date}_{to_date}_macro_regime_validation.md"
        ))
    }

    pub fn macro_regime_validation_export_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{VALIDATION_EXPORTS_DIR}/{from_date}_{to_date}_macro_regime_validation.csv"
        ))
    }

    pub fn event_context_validation_report_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{VALIDATION_REPORTS_DIR}/{from_date}_{to_date}_event_context_validation.md"
        ))
    }

    pub fn event_context_validation_export_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{VALIDATION_EXPORTS_DIR}/{from_date}_{to_date}_event_context_validation.csv"
        ))
    }

    pub fn actionability_validation_report_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{VALIDATION_REPORTS_DIR}/{from_date}_{to_date}_actionability_validation.md"
        ))
    }

    pub fn actionability_validation_export_path(from_date: &str, to_date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{VALIDATION_EXPORTS_DIR}/{from_date}_{to_date}_actionability_validation.csv"
        ))
    }

    pub fn intraday_readiness_report_path(date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{INTRADAY_REPORTS_DIR}/{date}_intraday_execution_readiness.md"
        ))
    }

    pub fn intraday_readiness_export_path(date: &str) -> PathBuf {
        PathBuf::from(format!(
            "{INTRADAY_EXPORTS_DIR}/{date}_intraday_execution_readiness.csv"
        ))
    }
}

pub mod market_data {
    use super::Duration;

    pub const ALPACA_API_KEY_ID_ENV: &str = "ALPACA_API_KEY_ID";
    pub const ALPACA_API_SECRET_KEY_ENV: &str = "ALPACA_API_SECRET_KEY";
    pub const ALPACA_FEED_ENV: &str = "ALPACA_FEED";
    pub const ALPACA_DATA_URL_ENV: &str = "ALPACA_DATA_URL";
    pub const LOOKBACK_DAYS_ENV: &str = "MERRYL_LOOKBACK_DAYS";

    pub const ALPACA_DATA_URL: &str = "https://data.alpaca.markets";
    pub const ALPACA_BARS_PATH: &str = "/v2/stocks/bars";
    pub const ALPACA_NEWS_PATH: &str = "/v1beta1/news";
    pub const ALPACA_KEY_HEADER: &str = "APCA-API-KEY-ID";
    pub const ALPACA_SECRET_HEADER: &str = "APCA-API-SECRET-KEY";
    pub const DEFAULT_ALPACA_FEED: &str = "iex";
    pub const DAILY_TIMEFRAME: &str = "1Day";
    pub const PRICE_ADJUSTMENT: &str = "all";
    pub const SORT_ASC: &str = "asc";
    pub const SORT_DESC: &str = "desc";
    pub const ALPACA_PAGE_LIMIT: &str = "10000";
    pub const ALPACA_NEWS_PAGE_LIMIT: &str = "50";
    pub const ALPACA_NEWS_MAX_PAGES: usize = 4;
    pub const DEFAULT_LOOKBACK_DAYS: i64 = 420;
    pub const NEWS_LOOKBACK_DAYS: i64 = 7;
    pub const ALPACA_BATCH_SIZE: usize = 25;
    pub const ALPACA_BATCH_SLEEP_MS: u64 = 350;
    pub const ALPACA_REQUEST_ATTEMPTS: usize = 2;
    pub const ALPACA_REQUEST_RETRY_SLEEP_MS: u64 = 1_000;
    pub const HTTP_TIMEOUT_SECONDS: u64 = 90;
    pub const SOURCE_PREFIX: &str = "alpaca";
    pub const NEWS_EVENT_TYPE: &str = "news";
    pub const NEWS_SOURCE_PREFIX: &str = "alpaca_news";

    pub fn http_timeout() -> Duration {
        Duration::from_secs(HTTP_TIMEOUT_SECONDS)
    }

    pub fn batch_sleep() -> Duration {
        Duration::from_millis(ALPACA_BATCH_SLEEP_MS)
    }

    pub fn retry_sleep() -> Duration {
        Duration::from_millis(ALPACA_REQUEST_RETRY_SLEEP_MS)
    }
}

pub mod intraday {
    pub const ALPACA_REQUESTS_PER_MINUTE_ENV: &str = "MERRYL_ALPACA_REQUESTS_PER_MINUTE";
    pub const PROFILE_TIMEFRAME_ENV: &str = "MERRYL_INTRADAY_PROFILE_TIMEFRAME";
    pub const TRIGGER_TIMEFRAME_ENV: &str = "MERRYL_INTRADAY_TRIGGER_TIMEFRAME";
    pub const CANDIDATE_LIMIT_ENV: &str = "MERRYL_INTRADAY_CANDIDATE_LIMIT";
    pub const OPENING_RANGE_MINUTES_ENV: &str = "MERRYL_INTRADAY_OPENING_RANGE_MINUTES";

    pub const DEFAULT_ALPACA_REQUESTS_PER_MINUTE: usize = 180;
    pub const DEFAULT_PROFILE_TIMEFRAME: &str = "30Min";
    pub const DEFAULT_TRIGGER_TIMEFRAME: &str = "5Min";
    pub const DEFAULT_CANDIDATE_LIMIT: usize = 50;
    pub const DEFAULT_OPENING_RANGE_MINUTES: usize = 30;

    pub const ADR_LOOKBACK: usize = 20;
    pub const ADR_MIN: f64 = 0.04;
    pub const RVOL_LOOKBACK: usize = 20;
    pub const RVOL_MIN: f64 = 1.5;
    pub const EMA_FAST: usize = 10;
    pub const EMA_SLOW: usize = 20;
    pub const MANSFIELD_LOOKBACK: usize = 50;
    pub const RS_TOP_PERCENTILE: f64 = 0.10;
    pub const VALUE_AREA_SHARE: f64 = 0.70;
    pub const VOLUME_PROFILE_MIN_BIN_SIZE: f64 = 0.01;
    pub const VOLUME_PROFILE_BIN_WIDTH_PCT: f64 = 0.0005;
    pub const VOLUME_PROFILE_ATR_LOOKBACK: usize = 20;
    pub const VOLUME_PROFILE_ATR_BIN_WIDTH_PCT: f64 = 0.05;
    pub const VOLUME_PROFILE_MAX_BIN_SIZE_PCT: f64 = 0.002;
    pub const VALUE_AREA_TIE_BREAK_POLICY: &str = "upper_on_equal_volume";
    pub const CONFLUENCE_WINDOW: f64 = 0.0075;
    pub const CONFLUENCE_MIN: usize = 3;
    pub const MATERIAL_EMA20_BREAK: f64 = -0.01;
    pub const VOLUME_SPIKE_MIN: f64 = 1.5;
    pub const DRYUP_RATIO_MAX: f64 = 0.65;
    pub const MICRO_CLUSTER_BAR_COUNT: usize = 3;
    pub const MICRO_CLUSTER_MAX_RANGE: f64 = 0.01;

    pub const DIRECTION_LONG: &str = "long";
    pub const LABEL_STAGE1: &str = "high_momentum_universe";
    pub const LABEL_STAGE2: &str = "structural_pullback_setup";
    pub const LABEL_STAGE3: &str = "intraday_execution_ready";
    pub const LABEL_MONITOR: &str = "monitor";

    pub const TRIGGER_ORB_BREAKOUT: &str = "orb_breakout";
    pub const TRIGGER_HOD_BREAK: &str = "hod_break";
    pub const TRIGGER_VOLUME_DRYUP_CONFIRMATION: &str = "volume_dryup_confirmation";
    pub const TRIGGER_MICRO_CLUSTER_BREAK: &str = "micro_cluster_break";
}

pub mod macro_data {
    use super::Duration;

    pub const FRED_API_KEY_ENV: &str = "FRED_API_KEY";
    pub const FRED_API_URL_ENV: &str = "FRED_API_URL";
    pub const MACRO_LOOKBACK_DAYS_ENV: &str = "MERRYL_MACRO_LOOKBACK_DAYS";

    pub const FRED_API_URL: &str = "https://api.stlouisfed.org";
    pub const FRED_SERIES_OBSERVATIONS_PATH: &str = "/fred/series/observations";
    pub const FRED_FILE_TYPE_JSON: &str = "json";
    pub const FRED_SORT_ASC: &str = "asc";
    pub const SOURCE_NAME: &str = "fred";
    pub const QUALITY_OK: &str = "ok";
    pub const DEFAULT_MACRO_LOOKBACK_DAYS: i64 = 900;
    pub const HTTP_TIMEOUT_SECONDS: u64 = 45;

    pub const MACRO_SERIES: &[(&str, &str, &str, &str)] = &[
        ("VIXCLS", "CBOE Volatility Index: VIX", "Daily", "Index"),
        (
            "DGS10",
            "10-Year Treasury Constant Maturity Rate",
            "Daily",
            "Percent",
        ),
        (
            "DGS2",
            "2-Year Treasury Constant Maturity Rate",
            "Daily",
            "Percent",
        ),
        (
            "T10Y2Y",
            "10-Year Treasury Minus 2-Year Treasury",
            "Daily",
            "Percent",
        ),
        ("DFF", "Effective Federal Funds Rate", "Daily", "Percent"),
        (
            "CPIAUCSL",
            "Consumer Price Index for All Urban Consumers",
            "Monthly",
            "Index",
        ),
        ("UNRATE", "Unemployment Rate", "Monthly", "Percent"),
        (
            "PAYEMS",
            "All Employees, Total Nonfarm",
            "Monthly",
            "Thousands",
        ),
        (
            "BAMLC0A0CM",
            "ICE BofA US Corporate Index Option-Adjusted Spread",
            "Daily",
            "Percent",
        ),
        (
            "DTWEXBGS",
            "Nominal Broad U.S. Dollar Index",
            "Daily",
            "Index",
        ),
        (
            "WALCL",
            "Federal Reserve Total Assets",
            "Weekly",
            "Millions of dollars",
        ),
    ];

    pub fn http_timeout() -> Duration {
        Duration::from_secs(HTTP_TIMEOUT_SECONDS)
    }
}

pub mod macro_validation {
    pub const DAILY_MAX_AGE_DAYS: i64 = 7;
    pub const WEEKLY_MAX_AGE_DAYS: i64 = 14;
    pub const MONTHLY_MAX_AGE_DAYS: i64 = 45;
    pub const VIX_STRESS_THRESHOLD: f64 = 20.0;
    pub const YIELD_CURVE_INVERSION_THRESHOLD: f64 = 0.0;
    pub const CPI_YOY_PRESSURE_THRESHOLD: f64 = 0.03;
    pub const TREND_MIN_DELTA: f64 = 0.0;

    pub const FLAG_VOLATILITY_STRESS: &str = "volatility_stress";
    pub const FLAG_RATE_PRESSURE: &str = "rate_pressure";
    pub const FLAG_YIELD_CURVE_INVERSION: &str = "yield_curve_inversion";
    pub const FLAG_CREDIT_STRESS: &str = "credit_stress";
    pub const FLAG_DOLLAR_PRESSURE: &str = "dollar_pressure";
    pub const FLAG_LIQUIDITY_TIGHTENING: &str = "liquidity_tightening";
    pub const FLAG_INFLATION_PRESSURE: &str = "inflation_pressure";
    pub const FLAG_LABOR_COOLING: &str = "labor_cooling";
}

pub mod event_data {
    use super::Duration;

    pub const ALPHA_VANTAGE_API_KEY_ENV: &str = "ALPHA_VANTAGE_API_KEY";
    pub const ALPHA_VANTAGE_API_URL_ENV: &str = "ALPHA_VANTAGE_API_URL";
    pub const EARNINGS_CALENDAR_HORIZON_ENV: &str = "MERRYL_EARNINGS_CALENDAR_HORIZON";
    pub const SEC_FILINGS_LOOKBACK_DAYS_ENV: &str = "MERRYL_SEC_FILINGS_LOOKBACK_DAYS";
    pub const SEC_USER_AGENT_ENV: &str = "MERRYL_SEC_USER_AGENT";

    pub const ALPHA_VANTAGE_API_URL: &str = "https://www.alphavantage.co";
    pub const ALPHA_VANTAGE_QUERY_PATH: &str = "/query";
    pub const ALPHA_VANTAGE_EARNINGS_CALENDAR_FUNCTION: &str = "EARNINGS_CALENDAR";
    pub const DEFAULT_EARNINGS_CALENDAR_HORIZON: &str = "3month";
    pub const ALPHA_VANTAGE_SOURCE_NAME: &str = "alpha_vantage:earnings_calendar";

    pub const SEC_COMPANY_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";
    pub const SEC_SUBMISSIONS_URL: &str = "https://data.sec.gov/submissions";
    pub const SEC_ARCHIVES_URL: &str = "https://www.sec.gov/Archives/edgar/data";
    pub const SEC_SOURCE_NAME: &str = "sec_edgar:submissions";
    pub const DEFAULT_SEC_FILINGS_LOOKBACK_DAYS: i64 = 14;
    pub const SEC_REQUEST_SLEEP_MS: u64 = 120;

    pub const EVENT_TYPE_EARNINGS: &str = "earnings";
    pub const EVENT_TYPE_FILING: &str = "filing";
    pub const QUALITY_OK: &str = "ok";
    pub const HTTP_TIMEOUT_SECONDS: u64 = 45;
    pub const SEC_TARGET_FORMS: &[&str] = &["8-K", "10-Q", "10-K"];

    pub fn http_timeout() -> Duration {
        Duration::from_secs(HTTP_TIMEOUT_SECONDS)
    }

    pub fn sec_request_sleep() -> Duration {
        Duration::from_millis(SEC_REQUEST_SLEEP_MS)
    }
}

pub mod event_validation {
    pub const GROUP_ALL_WATCHLIST: &str = "all_watchlist";
    pub const GROUP_PENDING_SOURCE: &str = "pending_source";
    pub const GROUP_EVENT_CONTEXT: &str = "event_context";
    pub const GROUP_RECENT_NEWS: &str = "recent_news";
    pub const GROUP_EARNINGS: &str = "earnings";
    pub const GROUP_FILING: &str = "filing";
    pub const GROUP_EVENT_RISK: &str = "event_risk";
    pub const GROUP_MULTIPLE_EVENT_TYPES: &str = "multiple_event_types";
}

pub mod universe {
    pub const SP500_WIKIPEDIA_URL: &str =
        "https://en.wikipedia.org/wiki/List_of_S%26P_500_companies";
    pub const EXCHANGE_US: &str = "US";
    pub const ASSET_STOCK: &str = "stock";
    pub const ASSET_BROAD_ETF: &str = "broad_etf";
    pub const ASSET_MACRO_ETF: &str = "macro_etf";
    pub const ASSET_SECTOR_ETF: &str = "sector_etf";

    pub const BROAD_ETFS: &[(&str, &str)] = &[
        ("SPY", "SPDR S&P 500 ETF Trust"),
        ("QQQ", "Invesco QQQ Trust"),
        ("IWM", "iShares Russell 2000 ETF"),
        ("DIA", "SPDR Dow Jones Industrial Average ETF"),
    ];

    pub const MACRO_ETFS: &[(&str, &str)] = &[
        ("TLT", "iShares 20+ Year Treasury Bond ETF"),
        ("GLD", "SPDR Gold Shares"),
        ("USO", "United States Oil Fund"),
    ];

    pub const SECTOR_ETFS: &[(&str, &str)] = &[
        ("Communication Services", "XLC"),
        ("Consumer Discretionary", "XLY"),
        ("Consumer Staples", "XLP"),
        ("Energy", "XLE"),
        ("Financials", "XLF"),
        ("Health Care", "XLV"),
        ("Industrials", "XLI"),
        ("Materials", "XLB"),
        ("Real Estate", "XLRE"),
        ("Technology", "XLK"),
        ("Utilities", "XLU"),
    ];

    pub fn required_market_symbols() -> Vec<&'static str> {
        BROAD_ETFS
            .iter()
            .map(|(symbol, _)| *symbol)
            .chain(MACRO_ETFS.iter().map(|(symbol, _)| *symbol))
            .chain(SECTOR_ETFS.iter().map(|(_, symbol)| *symbol))
            .collect()
    }
}

pub mod scoring {
    pub const BENCHMARK_SYMBOL: &str = "SPY";
    pub const GROWTH_SYMBOL: &str = "QQQ";
    pub const SMALL_CAP_SYMBOL: &str = "IWM";
    pub const INDUSTRIAL_SYMBOL: &str = "DIA";
    pub const LONG_BOND_SYMBOL: &str = "TLT";
    pub const GOLD_SYMBOL: &str = "GLD";
    pub const OIL_SYMBOL: &str = "USO";
    pub const RETURN_1D: usize = 1;
    pub const RETURN_5D: usize = 5;
    pub const RETURN_20D: usize = 20;
    pub const RETURN_50D: usize = 50;
    pub const RETURN_60D: usize = 60;
    pub const SCORE_MIN: f64 = 0.0;
    pub const SCORE_MAX: f64 = 100.0;
    pub const ZERO_PERCENT: f64 = 0.0;
    pub const NEUTRAL_SCORE: f64 = 50.0;
    pub const DEFAULT_RELATIVE_VOLUME: f64 = 1.0;
    pub const MIN_AVG_DOLLAR_VOLUME: f64 = 20_000_000.0;
    pub const INITIAL_RANK: usize = 0;
    pub const FIRST_RANK: usize = 1;
    pub const ZERO_RANK_CHANGE: f64 = 0.0;
    pub const BREADTH_COMPONENT_DIVISOR: f64 = 2.0;
    pub const RELATIVE_RETURN_SCORE_MULTIPLIER: f64 = 500.0;
    pub const TREND_RETURN_SCORE_MULTIPLIER: f64 = 400.0;
    pub const RELATIVE_VOLUME_BASELINE: f64 = 0.5;
    pub const RELATIVE_VOLUME_SCORE_MULTIPLIER: f64 = 100.0;
    pub const PERCENT_SCALE: f64 = 100.0;
    pub const LIQUIDITY_SCORE_SCALE: f64 = 20.0;
    pub const TREND_ABOVE_20D_50D_SCORE: f64 = 90.0;
    pub const TREND_ABOVE_20D_SCORE: f64 = 70.0;
    pub const TREND_BELOW_SCORE: f64 = 35.0;
    pub const STOCK_WATCHLIST_LIMIT: usize = 50;
    pub const REPORT_WATCHLIST_LIMIT: usize = 25;
    pub const EXPLANATION_LIMIT: usize = 10;
    pub const TOP_SECTOR_REPORT_LIMIT: usize = 5;
    pub const WEAK_SECTOR_REPORT_LIMIT: usize = 5;
    pub const TOP_INDUSTRY_REPORT_LIMIT: usize = 10;
    pub const HIGH_RELATIVE_VOLUME_REPORT_LIMIT: usize = 10;
    pub const CATALYST_PENDING_SOURCE: &str = "pending_source";
    pub const CATALYST_RECENT_NEWS_PREFIX: &str = "recent_news";
    pub const CATALYST_EARNINGS_PREFIX: &str = "earnings";
    pub const CATALYST_FILING_PREFIX: &str = "filing";
    pub const CATALYST_SEPARATOR: &str = " | ";
    pub const BACKTEST_HORIZONS: &[usize] = &[1, 5, 10, 20, 60];
    pub const BACKTEST_DECILES: usize = 10;

    pub const REGIME_RISK_ON_THRESHOLD: f64 = 60.0;
    pub const REGIME_RISK_OFF_THRESHOLD: f64 = 40.0;
    pub const REGIME_SPY_TREND_WEIGHT: f64 = 0.40;
    pub const REGIME_QQQ_RELATIVE_WEIGHT: f64 = 0.25;
    pub const REGIME_IWM_RELATIVE_WEIGHT: f64 = 0.25;
    pub const REGIME_DIA_RELATIVE_WEIGHT: f64 = 0.10;
    pub const REGIME_SPY_60D_WEIGHT: f64 = 0.40;
    pub const REGIME_RELATIVE_SCORE_MULTIPLIER: f64 = 500.0;
    pub const REGIME_TREND_SCORE_MULTIPLIER: f64 = 400.0;
    pub const REGIME_CONTEXT_RETURN_THRESHOLD: f64 = 0.03;

    pub const SECTOR_RELATIVE_RETURN_WEIGHT: f64 = 0.30;
    pub const SECTOR_TREND_WEIGHT: f64 = 0.20;
    pub const SECTOR_RELATIVE_VOLUME_WEIGHT: f64 = 0.20;
    pub const SECTOR_BREADTH_WEIGHT: f64 = 0.20;
    pub const SECTOR_SCORE_WEIGHT_TOTAL: f64 = SECTOR_RELATIVE_RETURN_WEIGHT
        + SECTOR_TREND_WEIGHT
        + SECTOR_RELATIVE_VOLUME_WEIGHT
        + SECTOR_BREADTH_WEIGHT;

    pub const INDUSTRY_RELATIVE_SECTOR_WEIGHT: f64 = 0.30;
    pub const INDUSTRY_RELATIVE_SPY_WEIGHT: f64 = 0.20;
    pub const INDUSTRY_BREADTH_WEIGHT: f64 = 0.20;
    pub const INDUSTRY_RELATIVE_VOLUME_WEIGHT: f64 = 0.15;
    pub const INDUSTRY_HIGH_RATE_WEIGHT: f64 = 0.15;

    pub const STOCK_SECTOR_WEIGHT: f64 = 0.30;
    pub const STOCK_RELATIVE_STRENGTH_WEIGHT: f64 = 0.25;
    pub const STOCK_RELATIVE_VOLUME_WEIGHT: f64 = 0.20;
    pub const STOCK_TREND_WEIGHT: f64 = 0.15;
    pub const STOCK_LIQUIDITY_WEIGHT: f64 = 0.10;
}

pub mod classification {
    pub const LEADING_SECTOR_MAX_RANK: usize = 3;
    pub const LEADING_INDUSTRY_MAX_RANK: usize = 10;
    pub const RELATIVE_STRENGTH_MIN_RETURN: f64 = 0.0;
    pub const VOLUME_CONFIRMED_MIN_RELATIVE_VOLUME: f64 = 1.2;

    pub const LABEL_SECTOR_LEADER: &str = "sector_leader";
    pub const LABEL_INDUSTRY_LEADER: &str = "industry_leader";
    pub const LABEL_RELATIVE_STRENGTH_LEADER: &str = "relative_strength_leader";
    pub const LABEL_VOLUME_CONFIRMED: &str = "volume_confirmed";
    pub const LABEL_NEW_LEADER: &str = "new_leader";
    pub const LABEL_EVENT_CONTEXT: &str = "event_context";
    pub const LABEL_EVENT_RISK: &str = "event_risk";
    pub const LABEL_MACRO_CONFLICT_CONTEXT: &str = "macro_conflict_context";
}

pub mod actionability {
    pub const ATR_LOOKBACK: usize = 14;
    pub const HIGH_20D_LOOKBACK: usize = 20;
    pub const HIGH_60D_LOOKBACK: usize = 60;
    pub const RANGE_10D_LOOKBACK: usize = 10;
    pub const EXTENDED_5D_RETURN: f64 = 0.08;
    pub const EXTENDED_1D_RETURN: f64 = 0.05;
    pub const EXTENDED_GAP: f64 = 0.04;
    pub const EXTENDED_DISTANCE_20D_MA: f64 = 0.12;
    pub const EXTENDED_DISTANCE_50D_MA: f64 = 0.20;
    pub const EXTENDED_ATR_MULTIPLE_FROM_20D_MA: f64 = 2.5;
    pub const PULLBACK_FROM_20D_HIGH_MIN: f64 = -0.12;
    pub const PULLBACK_FROM_20D_HIGH_MAX: f64 = -0.03;
    pub const NEAR_20D_MA_MAX_DISTANCE: f64 = 0.04;
    pub const NEAR_50D_MA_MAX_DISTANCE: f64 = 0.06;
    pub const COMPRESSION_10D_RANGE_MAX: f64 = 0.08;
    pub const STRONG_SCORE_MIN: f64 = 70.0;
    pub const EARLY_ROTATION_5D_MAX_RETURN: f64 = EXTENDED_5D_RETURN;

    pub const COMPONENT_MA_20D: &str = "ma_20d";
    pub const COMPONENT_MA_50D: &str = "ma_50d";
    pub const COMPONENT_DISTANCE_FROM_20D_MA_PCT: &str = "distance_from_20d_ma_pct";
    pub const COMPONENT_DISTANCE_FROM_50D_MA_PCT: &str = "distance_from_50d_ma_pct";
    pub const COMPONENT_ATR_14D: &str = "atr_14d";
    pub const COMPONENT_ATR_14D_PCT: &str = "atr_14d_pct";
    pub const COMPONENT_ATR_EXTENSION_FROM_20D_MA: &str = "atr_extension_from_20d_ma";
    pub const COMPONENT_ATR_EXTENSION_FROM_50D_MA: &str = "atr_extension_from_50d_ma";
    pub const COMPONENT_HIGH_20D: &str = "high_20d";
    pub const COMPONENT_HIGH_60D: &str = "high_60d";
    pub const COMPONENT_DISTANCE_FROM_20D_HIGH_PCT: &str = "distance_from_20d_high_pct";
    pub const COMPONENT_DISTANCE_FROM_60D_HIGH_PCT: &str = "distance_from_60d_high_pct";
    pub const COMPONENT_RANGE_10D_PCT: &str = "range_10d_pct";
    pub const COMPONENT_GAP_PCT: &str = "gap_pct";
    pub const COMPONENT_TRUE_RANGE_PCT: &str = "true_range_pct";
    pub const COMPONENT_ACTIONABILITY_LABELS: &str = "actionability_labels";
    pub const COMPONENT_PRIMARY_ACTIONABILITY: &str = "primary_actionability";

    pub const LABEL_ALL_WATCHLIST: &str = "all_watchlist";
    pub const LABEL_EXTENDED_LEADER: &str = "extended_leader";
    pub const LABEL_ACTIONABLE_LEADER: &str = "actionable_leader";
    pub const LABEL_EARLY_ROTATION_CANDIDATE: &str = "early_rotation_candidate";
    pub const LABEL_PULLBACK_LEADER: &str = "pullback_leader";
    pub const LABEL_BASE_COMPRESSION_CANDIDATE: &str = "base_compression_candidate";
    pub const LABEL_EVENT_WATCH_UNCONFIRMED: &str = "event_watch_unconfirmed";
    pub const LABEL_UNCLASSIFIED_LEADER: &str = "unclassified_leader";

    pub const PRIMARY_PRIORITY: &[&str] = &[
        LABEL_EXTENDED_LEADER,
        LABEL_PULLBACK_LEADER,
        LABEL_BASE_COMPRESSION_CANDIDATE,
        LABEL_EARLY_ROTATION_CANDIDATE,
        LABEL_ACTIONABLE_LEADER,
        LABEL_EVENT_WATCH_UNCONFIRMED,
        LABEL_UNCLASSIFIED_LEADER,
    ];

    pub const REVIEW_QUEUE_ORDER: &[&str] = &[
        LABEL_EARLY_ROTATION_CANDIDATE,
        LABEL_BASE_COMPRESSION_CANDIDATE,
        LABEL_PULLBACK_LEADER,
        LABEL_ACTIONABLE_LEADER,
        LABEL_EXTENDED_LEADER,
        LABEL_EVENT_WATCH_UNCONFIRMED,
        LABEL_UNCLASSIFIED_LEADER,
    ];
}

pub mod quality {
    use super::scoring::RETURN_60D;

    pub const MIN_REQUIRED_PRICE_BARS: i64 = RETURN_60D as i64 + 1;
    pub const MIN_REQUIRED_SCORE_DATES: i64 = RETURN_60D as i64;
}

pub mod output_text {
    pub const DAILY_REPORT_TITLE: &str = "Daily Market Rotation Report";
    pub const REPORT_RULE: &str = "Rule: this is a market rotation watchlist, not an automatic trade signal. Chart structure, invalidation, and risk define any trade.";
    pub const MARKET_REGIME_SECTION: &str = "Market Regime";
    pub const MARKET_REGIME_V1_NOTE: &str = "Market regime score: daily ETF price proxies SPY, QQQ, IWM, DIA, TLT, GLD, and USO. FRED macro context is stored separately and is not part of scoring yet.";
    pub const MACRO_CONTEXT_SECTION: &str = "Macro Context Coverage";
    pub const MACRO_CONTEXT_NOTE: &str = "FRED macro observations are stored as context/provenance only; they are not scoring inputs yet.";
    pub const TOP_SECTORS_SECTION: &str = "Top Sectors";
    pub const WEAK_SECTORS_SECTION: &str = "Weak Sectors";
    pub const SECTOR_RANK_CHANGES_SECTION: &str = "Sector Rank Changes";
    pub const SECTOR_MAP_NOTE: &str = "Sector ranking is a market-map and attention layer. PDB-2 validation labels it as map-only, not a proven forward-return signal.";
    pub const TOP_INDUSTRIES_SECTION: &str = "Top Industries Or Themes";
    pub const ACTIONABILITY_SECTION: &str = "Actionability Review Queue";
    pub const ACTIONABILITY_NOTE: &str = "Actionability groups are a chart-review queue, not trade signals. Core score and rank remain unchanged.";
    pub const WATCHLIST_SECTION: &str = "Top Stocks Worth Charting";
    pub const NEW_LEADERS_SECTION: &str = "New Leaders";
    pub const HIGH_RELATIVE_VOLUME_SECTION: &str = "High Relative Volume Names";
    pub const CATALYST_SECTION: &str = "Catalyst / Event Flags";
    pub const NOTES_SECTION: &str = "Notes For Chart Review";
    pub const EXPLANATION_SECTION: &str = "Why These Names";
    pub const SECTOR_TABLE_HEADER: &str = "| Rank | Sector | ETF | Score | 1D | 5D | 20D | 60D | Vs SPY | Rel Vol | Breadth 20D | Breadth 50D | Rank Change |";
    pub const SECTOR_TABLE_ALIGNMENT: &str =
        "|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|";
    pub const INDUSTRY_TABLE_HEADER: &str = "| Rank | Industry / Theme | Sector | Score | 5D | 20D | 60D | Vs Sector | Vs SPY | Rel Vol | Breadth 20D | Breadth 50D | 20D Highs | Members |";
    pub const INDUSTRY_TABLE_ALIGNMENT: &str =
        "|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|";
    pub const ACTIONABILITY_TABLE_HEADER: &str = "| Bucket | Rank | Symbol | Score | 5D | 20D | Rel Sector | Rel Vol | 20D MA Dist | 20D High Dist | ATR Ext | Catalyst |";
    pub const ACTIONABILITY_TABLE_ALIGNMENT: &str =
        "|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|";
    pub const WATCHLIST_TABLE_HEADER: &str = "| Rank | Symbol | Name | Sector | Industry | Score | 20D | Rel Sector | Rel Vol | Trend | Primary Actionability | Actionability | Classification | Catalyst |";
    pub const WATCHLIST_TABLE_ALIGNMENT: &str =
        "|---:|---|---|---|---|---:|---:|---:|---:|---|---|---|---|---|";
    pub const MACRO_TABLE_HEADER: &str =
        "| Series | Name | Frequency | Latest | Observations | Status |";
    pub const MACRO_TABLE_ALIGNMENT: &str = "|---|---|---|---:|---:|---|";
    pub const HIGH_RELATIVE_VOLUME_TABLE_HEADER: &str =
        "| Rank | Symbol | Sector | Score | Rel Vol | 20D | Rel Sector |";
    pub const HIGH_RELATIVE_VOLUME_TABLE_ALIGNMENT: &str = "|---:|---|---|---:|---:|---:|---:|";
    pub const NO_PRIOR_RANK_HISTORY: &str =
        "No prior dated sector ranking exists yet; rank changes will appear after multiple runs.";
    pub const NO_PRIOR_WATCHLIST_HISTORY: &str =
        "No prior dated watchlist exists yet; this run establishes the leadership baseline.";
    pub const NO_NEW_LEADERS: &str =
        "No new names entered the top watchlist compared with the prior dated run.";
    pub const CATALYST_SOURCE_NOTE: &str = "Event sources: Alpaca News, Alpha Vantage Earnings Calendar, and SEC EDGAR submissions. These are context flags, not trade signals.";
    pub const CATALYST_PENDING_NOTE: &str = "No news, earnings calendar, or recent SEC filing event found for the current top watchlist.";
    pub const CHART_REVIEW_NOTES: &[&str] = &[
        "Use this report to choose what to chart first, not to enter trades automatically.",
        "Confirm chart structure, invalidation level, liquidity, and earnings risk before any trade.",
        "Treat sector and industry context as the reason to focus attention; price action still decides timing.",
    ];
}
