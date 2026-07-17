use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::dashboard as dashboard_config;
use crate::data::{new_client, run_screener};
use crate::domain::models::ScreenerResultRow;
use crate::storage::{Database, default_db_path};

use super::models::{ApiErrorDto, DatesDto, ScreenerResponseDto, ScreenerResultDto};
use super::repository::{
    is_missing_dashboard_data_error, load_all_screener_results, load_dashboard_for_date,
    load_health, load_latest_dashboard, load_screener_results, load_scored_dates,
    screener_has_any_results, screener_has_results,
};

/// How often the background task re-fetches all sectors from Finviz.
const REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60); // 15 minutes

/// All sector keys for the background refresh.
/// "All Sectors" is computed by merging these individually-fetched results.
const SCREENER_SECTORS: &[&str] = &[
    "Basic Materials",
    "Communication Services",
    "Consumer Cyclical",
    "Consumer Defensive",
    "Energy",
    "Financial",
    "Healthcare",
    "Industrials",
    "Real Estate",
    "Technology",
    "Utilities",
];

#[derive(Debug, Clone)]
pub struct DashboardServerConfig {
    pub port: u16,
    pub db_path: PathBuf,
    pub frontend_dist_dir: PathBuf,
}

impl DashboardServerConfig {
    pub fn local(port: u16) -> Self {
        Self {
            port,
            db_path: default_db_path(),
            frontend_dist_dir: PathBuf::from(dashboard_config::FRONTEND_DIST_DIR),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DashboardState {
    db_path: PathBuf,
}

pub async fn run_dashboard(config: DashboardServerConfig) -> Result<()> {
    spawn_screener_refresh(config.db_path.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let app = router(config.db_path.clone(), &config.frontend_dist_dir);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind dashboard server to {addr}"))?;
    let local_addr = listener
        .local_addr()
        .context("failed to read dashboard server address")?;

    println!("Merryl dashboard:");
    println!("url: http://{local_addr}");
    println!("database: {}", config.db_path.display());
    println!("screener cache: refreshing every {:?}", REFRESH_INTERVAL);
    if !config.frontend_dist_dir.exists() {
        println!(
            "frontend: missing {}; run `npm --prefix dashboard run build`",
            config.frontend_dist_dir.display()
        );
    }

    axum::serve(listener, app)
        .await
        .context("dashboard server failed")
}

/// Spawn a background task that refreshes screener results into SQLite.
fn spawn_screener_refresh(db_path: PathBuf) {
    tokio::spawn(async move {
        // Initial warm-up: wait for the server to start, then refresh once.
        tokio::time::sleep(Duration::from_secs(3)).await;
        let mut interval = tokio::time::interval(REFRESH_INTERVAL);
        interval.tick().await; // consume the immediate tick

        loop {
            let db_path = db_path.clone();
            let result = tokio::task::spawn_blocking(move || {
                let client = new_client()?;
                // Open a DB connection for this batch of writes
                let mut db = Database::open(&db_path)
                    .context("failed to open DB for screener refresh")?;
                db.migrate()?;

                for name in SCREENER_SECTORS {
                    match run_screener(&client, name) {
                        Ok(results) => {
                            let rows: Vec<ScreenerResultRow> = results
                                .into_iter()
                                .map(|r| ScreenerResultRow {
                                    sector: name.to_string(),
                                    ticker: r.ticker,
                                    company: r.company,
                                    industry: r.industry,
                                    market_cap: r.market_cap,
                                    pe_ratio: r.pe_ratio,
                                    price: r.price,
                                    change: r.change,
                                    volume: r.volume,
                                })
                                .collect();

                            if let Err(e) = db.replace_screener_results(name, &rows) {
                                eprintln!("screener cache: DB write error for {name}: {e}");
                            } else {
                                println!("screener cache: refreshed {name} ({} results)", rows.len());
                            }
                        }
                        Err(e) => {
                            eprintln!("screener cache: error fetching {name}: {e}");
                        }
                    }
                }
                anyhow::Ok(())
            });

            if let Err(e) = result.await {
                eprintln!("screener cache background task: {e}");
            }

            interval.tick().await;
        }
    });
}

#[derive(Debug, Deserialize)]
pub struct ScreenerQuery {
    pub sector: Option<String>,
    /// Set to "true" to force a live fetch from Finviz instead of using the cache.
    pub refresh: Option<bool>,
}

pub fn router(db_path: PathBuf, frontend_dist_dir: &Path) -> Router {
    let state = DashboardState { db_path };
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/dates", get(dates))
        .route("/api/dashboard/latest", get(latest_dashboard))
        .route("/api/dashboard/{date}", get(dashboard_for_date))
        .route("/api/screener", get(screener))
        .with_state(state);

    if frontend_dist_dir.exists() {
        let index_path = frontend_dist_dir.join("index.html");
        api.fallback_service(ServeDir::new(frontend_dist_dir).fallback(ServeFile::new(index_path)))
    } else {
        api.fallback(missing_frontend)
    }
}

async fn health(State(state): State<DashboardState>) -> impl IntoResponse {
    api_response(load_health(&state.db_path))
}

async fn dates(State(state): State<DashboardState>) -> impl IntoResponse {
    api_response(load_scored_dates(&state.db_path).map(|dates| DatesDto { dates }))
}

async fn latest_dashboard(State(state): State<DashboardState>) -> impl IntoResponse {
    api_response(load_latest_dashboard(&state.db_path))
}

async fn dashboard_for_date(
    State(state): State<DashboardState>,
    AxumPath(date): AxumPath<String>,
) -> impl IntoResponse {
    api_response(load_dashboard_for_date(&state.db_path, &date))
}

async fn screener(
    State(state): State<DashboardState>,
    Query(query): Query<ScreenerQuery>,
) -> impl IntoResponse {
    let db_path = state.db_path.clone();
    let sector = query.sector.clone();
    let refresh = query.refresh.unwrap_or(false);

    let result = tokio::task::spawn_blocking(move || {
        let is_all = sector.as_deref().unwrap_or("").is_empty();

        if refresh {
            // Force live fetch from Finviz
            let client = new_client()?;
            let mut db = Database::open(&db_path)?;
            db.migrate()?;

            if is_all {
                // Fetch all 11 sectors and merge
                let mut all_rows = Vec::new();
                let mut seen = std::collections::HashSet::new();
                for name in SCREENER_SECTORS {
                    match run_screener(&client, name) {
                        Ok(results) => {
                            let rows: Vec<ScreenerResultRow> = results
                                .into_iter()
                                .filter(|r| seen.insert(r.ticker.clone()))
                                .map(|r| ScreenerResultRow {
                                    sector: name.to_string(),
                                    ticker: r.ticker,
                                    company: r.company,
                                    industry: r.industry,
                                    market_cap: r.market_cap,
                                    pe_ratio: r.pe_ratio,
                                    price: r.price,
                                    change: r.change,
                                    volume: r.volume,
                                })
                                .collect();
                            if let Err(e) = db.replace_screener_results(name, &rows) {
                                eprintln!("screener: DB write error for {name}: {e}");
                            }
                            all_rows.extend(rows);
                        }
                        Err(e) => eprintln!("screener: error fetching {name}: {e}"),
                    }
                }
                Ok(all_rows)
            } else {
                // Fetch single sector
                let name = sector.as_deref().unwrap();
                let results = run_screener(&client, name)?;
                let rows: Vec<ScreenerResultRow> = results
                    .into_iter()
                    .map(|r| ScreenerResultRow {
                        sector: name.to_string(),
                        ticker: r.ticker,
                        company: r.company,
                        industry: r.industry,
                        market_cap: r.market_cap,
                        pe_ratio: r.pe_ratio,
                        price: r.price,
                        change: r.change,
                        volume: r.volume,
                    })
                    .collect();
                db.replace_screener_results(name, &rows)?;
                Ok(rows)
            }
        } else {
            // Read from DB cache
            let rows = if is_all {
                if screener_has_any_results(&db_path) {
                    load_all_screener_results(&db_path)?
                } else {
                    // Cache empty: fetch all sectors
                    let client = new_client()?;
                    let mut db = Database::open(&db_path)?;
                    db.migrate()?;
                    let mut all_rows = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    for name in SCREENER_SECTORS {
                        match run_screener(&client, name) {
                            Ok(results) => {
                                let rows: Vec<ScreenerResultRow> = results
                                    .into_iter()
                                    .filter(|r| seen.insert(r.ticker.clone()))
                                    .map(|r| ScreenerResultRow {
                                        sector: name.to_string(),
                                        ticker: r.ticker,
                                        company: r.company,
                                        industry: r.industry,
                                        market_cap: r.market_cap,
                                        pe_ratio: r.pe_ratio,
                                        price: r.price,
                                        change: r.change,
                                        volume: r.volume,
                                    })
                                    .collect();
                                if let Err(e) = db.replace_screener_results(name, &rows) {
                                    eprintln!("screener: DB write error for {name}: {e}");
                                }
                                all_rows.extend(rows);
                            }
                            Err(e) => eprintln!("screener: error fetching {name}: {e}"),
                        }
                    }
                    all_rows
                }
            } else {
                let sector_key = sector.as_deref().unwrap();
                if screener_has_results(&db_path, sector_key) {
                    load_screener_results(&db_path, sector_key)?
                } else {
                    // Cache miss: fetch live
                    let client = new_client()?;
                    let results = run_screener(&client, sector_key)?;
                    let mut db = Database::open(&db_path)?;
                    db.migrate()?;
                    let rows: Vec<ScreenerResultRow> = results
                        .into_iter()
                        .map(|r| ScreenerResultRow {
                            sector: sector_key.to_string(),
                            ticker: r.ticker,
                            company: r.company,
                            industry: r.industry,
                            market_cap: r.market_cap,
                            pe_ratio: r.pe_ratio,
                            price: r.price,
                            change: r.change,
                            volume: r.volume,
                        })
                        .collect();
                    db.replace_screener_results(sector_key, &rows)?;
                    rows
                }
            };
            Ok(rows)
        }
        .map(|results| {
            let count = results.len();
            ScreenerResponseDto {
                sector: sector.clone(),
                results: results
                    .into_iter()
                    .map(|r| ScreenerResultDto {
                        ticker: r.ticker,
                        company: r.company,
                        sector: r.sector,
                        industry: r.industry,
                        market_cap: r.market_cap,
                        pe_ratio: r.pe_ratio,
                        price: r.price,
                        change: r.change,
                        volume: r.volume,
                    })
                    .collect(),
                count,
            }
        })
    })
    .await
    .expect("screener blocking task panicked");

    api_response(result)
}

async fn missing_frontend() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Merryl Dashboard</title></head>
<body>
<h1>Merryl dashboard frontend is not built</h1>
<p>Run <code>npm --prefix dashboard run build</code>, then start <code>merryl dashboard</code> again.</p>
</body>
</html>"#,
    )
}

fn api_response<T: serde::Serialize>(result: Result<T>) -> axum::response::Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(err) => {
            let message = err.to_string();
            let status = if is_missing_dashboard_data_error(&message) {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(ApiErrorDto { message })).into_response()
        }
    }
}
