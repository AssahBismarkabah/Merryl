use std::fmt::Display;

use crate::config::{CLI_NAME, DAILY_RUN_COMMAND};
use crate::storage::DbCounts;

pub fn missing_database(path: impl Display) -> String {
    format!("database: missing ({path})\nnext: set Alpaca keys, then run `{DAILY_RUN_COMMAND}`")
}

pub fn database_status(path: impl Display, counts: &DbCounts) -> String {
    format!(
        "database: {path}\nsymbols: {}\ndaily prices: {}\nintraday prices: {}\nmacro series observations: {}\nevents: {}\nmarket regime scores: {}\nscore dates: {}\nsector scores: {}\nindustry scores: {}\nstock scores: {}\nwatchlist rows: {}\nvolume profiles: {}\nintraday setups: {}\nintraday triggers: {}\nbacktest results: {}",
        counts.symbols,
        counts.prices_daily,
        counts.intraday_prices,
        counts.macro_series,
        counts.events,
        counts.market_regime_scores,
        counts.score_dates,
        counts.sector_scores,
        counts.industry_scores,
        counts.stock_scores,
        counts.watchlist_rows,
        counts.volume_profiles,
        counts.intraday_setups,
        counts.intraday_triggers,
        counts.backtest_results,
    )
}

pub fn cli_name_check() -> String {
    ok(format!("CLI command name is {CLI_NAME}"))
}

pub fn ok(label: impl Display) -> String {
    format!("ok: {label}")
}

pub fn missing(label: impl Display) -> String {
    format!("missing: {label}")
}

pub fn not_created_yet(label: impl Display) -> String {
    format!("not created yet: {label}")
}
