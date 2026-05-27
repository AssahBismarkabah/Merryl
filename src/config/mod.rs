use std::path::PathBuf;
use std::time::Duration;

pub const APP_NAME: &str = "Merryl";
pub const CLI_NAME: &str = "merryl";
pub const USER_AGENT: &str = "Merryl/0.1";
pub const DAILY_RUN_COMMAND: &str = "merryl run daily --date latest";

pub mod paths {
    use super::PathBuf;

    pub const DB_PATH: &str = "data/market.db";
    pub const REPORTS_DIR: &str = "reports";
    pub const EXPORTS_DIR: &str = "exports";
    pub const BACKTEST_REPORTS_DIR: &str = "reports/backtests";
    pub const BACKTEST_EXPORTS_DIR: &str = "exports/backtests";
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
    pub const ALPACA_KEY_HEADER: &str = "APCA-API-KEY-ID";
    pub const ALPACA_SECRET_HEADER: &str = "APCA-API-SECRET-KEY";
    pub const DEFAULT_ALPACA_FEED: &str = "iex";
    pub const DAILY_TIMEFRAME: &str = "1Day";
    pub const PRICE_ADJUSTMENT: &str = "all";
    pub const SORT_ASC: &str = "asc";
    pub const ALPACA_PAGE_LIMIT: &str = "10000";
    pub const DEFAULT_LOOKBACK_DAYS: i64 = 420;
    pub const ALPACA_BATCH_SIZE: usize = 25;
    pub const ALPACA_BATCH_SLEEP_MS: u64 = 350;
    pub const HTTP_TIMEOUT_SECONDS: u64 = 45;
    pub const SOURCE_PREFIX: &str = "alpaca";

    pub fn http_timeout() -> Duration {
        Duration::from_secs(HTTP_TIMEOUT_SECONDS)
    }

    pub fn batch_sleep() -> Duration {
        Duration::from_millis(ALPACA_BATCH_SLEEP_MS)
    }
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

pub mod output_text {
    pub const DAILY_REPORT_TITLE: &str = "Daily Market Rotation Report";
    pub const REPORT_RULE: &str = "Rule: this is a market rotation watchlist, not an automatic trade signal. Chart structure, invalidation, and risk define any trade.";
    pub const MARKET_REGIME_SECTION: &str = "Market Regime";
    pub const MARKET_REGIME_V1_NOTE: &str = "Market Regime is lightweight V1 context. It uses daily ETF price proxies, not a full macro model or a trading signal.";
    pub const TOP_SECTORS_SECTION: &str = "Top Sectors";
    pub const WEAK_SECTORS_SECTION: &str = "Weak Sectors";
    pub const SECTOR_RANK_CHANGES_SECTION: &str = "Sector Rank Changes";
    pub const SECTOR_MAP_NOTE: &str = "Sector ranking is a market-map and attention layer. PDB-2 validation labels it as map-only, not a proven forward-return signal.";
    pub const TOP_INDUSTRIES_SECTION: &str = "Top Industries Or Themes";
    pub const WATCHLIST_SECTION: &str = "Top Stocks Worth Charting";
    pub const NEW_LEADERS_SECTION: &str = "New Leaders";
    pub const HIGH_RELATIVE_VOLUME_SECTION: &str = "High Relative Volume Names";
    pub const CATALYST_SECTION: &str = "Catalyst / Earnings Flags";
    pub const NOTES_SECTION: &str = "Notes For Chart Review";
    pub const EXPLANATION_SECTION: &str = "Why These Names";
    pub const SECTOR_TABLE_HEADER: &str = "| Rank | Sector | ETF | Score | 1D | 5D | 20D | 60D | Vs SPY | Rel Vol | Breadth 20D | Breadth 50D | Rank Change |";
    pub const SECTOR_TABLE_ALIGNMENT: &str =
        "|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|";
    pub const INDUSTRY_TABLE_HEADER: &str = "| Rank | Industry / Theme | Sector | Score | 5D | 20D | 60D | Vs Sector | Vs SPY | Rel Vol | Breadth 20D | Breadth 50D | 20D Highs | Members |";
    pub const INDUSTRY_TABLE_ALIGNMENT: &str =
        "|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|";
    pub const WATCHLIST_TABLE_HEADER: &str = "| Rank | Symbol | Name | Sector | Industry | Score | 20D | Rel Sector | Rel Vol | Trend | Catalyst |";
    pub const WATCHLIST_TABLE_ALIGNMENT: &str =
        "|---:|---|---|---|---|---:|---:|---:|---:|---|---|";
    pub const HIGH_RELATIVE_VOLUME_TABLE_HEADER: &str =
        "| Rank | Symbol | Sector | Score | Rel Vol | 20D | Rel Sector |";
    pub const HIGH_RELATIVE_VOLUME_TABLE_ALIGNMENT: &str = "|---:|---|---|---:|---:|---:|---:|";
    pub const NO_PRIOR_RANK_HISTORY: &str =
        "No prior dated sector ranking exists yet; rank changes will appear after multiple runs.";
    pub const NO_PRIOR_WATCHLIST_HISTORY: &str =
        "No prior dated watchlist exists yet; this run establishes the leadership baseline.";
    pub const NO_NEW_LEADERS: &str =
        "No new names entered the top watchlist compared with the prior dated run.";
    pub const CATALYST_PENDING_NOTE: &str = "Catalyst and earnings sources are not connected yet; current values remain pending_source.";
    pub const CHART_REVIEW_NOTES: &[&str] = &[
        "Use this report to choose what to chart first, not to enter trades automatically.",
        "Confirm chart structure, invalidation level, liquidity, and earnings risk before any trade.",
        "Treat sector and industry context as the reason to focus attention; price action still decides timing.",
    ];
}
