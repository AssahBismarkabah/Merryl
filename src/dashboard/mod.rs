mod models;
mod repository;
mod server;

pub use models::{
    ApiErrorDto, BacktestDto, DashboardSnapshot, DataHealthDto, DatesDto, HealthDto, IndustryDto,
    LatestScoreCoverageDto, MacroContextDto, PriceCoverageDto, RegimeDto, ScreenerResponseDto,
    ScreenerResultDto, SectorDto, StockDto, WatchlistDto,
};
pub use repository::{
    StaticDashboardExport, export_static_dashboard, load_dashboard_for_date, load_health,
    load_latest_dashboard, load_scored_dates,
};
pub use server::{DashboardServerConfig, router, run_dashboard};
