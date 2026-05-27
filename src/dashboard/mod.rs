mod models;
mod repository;
mod server;

pub use models::{
    ApiErrorDto, BacktestDto, DashboardSnapshot, DataHealthDto, DatesDto, HealthDto, IndustryDto,
    LatestScoreCoverageDto, PriceCoverageDto, RegimeDto, SectorDto, StockDto, WatchlistDto,
};
pub use repository::{
    load_dashboard_for_date, load_health, load_latest_dashboard, load_scored_dates,
};
pub use server::{DashboardServerConfig, router, run_dashboard};
