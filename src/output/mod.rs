mod backtest;
mod csv;
mod formatting;
mod macro_regime_validation;
mod markdown;
mod paths;
mod reports;

pub use backtest::{BacktestOutputPaths, write_backtest_outputs};
pub use macro_regime_validation::{
    MacroRegimeValidationOutputPaths, write_macro_regime_validation_outputs,
};
pub use markdown::{DailyReportInput, daily_report_markdown};
pub use reports::write_daily_outputs;
