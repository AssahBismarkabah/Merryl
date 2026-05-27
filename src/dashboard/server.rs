use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use tower_http::services::{ServeDir, ServeFile};

use crate::config::dashboard as dashboard_config;
use crate::storage::default_db_path;

use super::models::{ApiErrorDto, DatesDto};
use super::repository::{
    is_missing_dashboard_data_error, load_dashboard_for_date, load_health, load_latest_dashboard,
    load_scored_dates,
};

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

pub fn router(db_path: PathBuf, frontend_dist_dir: &Path) -> Router {
    let state = DashboardState { db_path };
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/dates", get(dates))
        .route("/api/dashboard/latest", get(latest_dashboard))
        .route("/api/dashboard/{date}", get(dashboard_for_date))
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
