use anyhow::{Context, Result};

use crate::data::{new_client, run_screener};
use crate::domain::models::ScreenerResultRow;
use crate::storage::{Database, default_db_path};

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

/// Result of a screener cache refresh.
#[derive(Debug)]
pub struct ScreenerCacheResult {
    pub database: std::path::PathBuf,
    pub sectors: Vec<SectorCacheResult>,
}

#[derive(Debug)]
pub struct SectorCacheResult {
    pub name: String,
    pub count: usize,
}

/// Refresh the screener cache for all sectors.
///
/// Fetches each sector from Finviz and stores results in SQLite.
/// This is called from the CLI as `merryl run screener` and is
/// part of the static deployment pipeline.
pub fn run_screener_cache() -> Result<ScreenerCacheResult> {
    let db_path = default_db_path();
    let client = new_client()?;
    let mut db = Database::open(&db_path)?;
    db.migrate()?;

    let mut sectors = Vec::new();

    for name in SCREENER_SECTORS {
        let results = run_screener(&client, name)
            .with_context(|| format!("failed to fetch screener data for {name}"))?;

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
                dividend: r.dividend,
                roa: r.roa,
                roe: r.roe,
                debt_equity: r.debt_equity,
                net_profit_margin: r.net_profit_margin,
            })
            .collect();

        let count = rows.len();
        db.replace_screener_results(name, &rows)
            .with_context(|| format!("failed to write screener results for {name}"))?;

        sectors.push(SectorCacheResult {
            name: name.to_string(),
            count,
        });
    }

    Ok(ScreenerCacheResult {
        database: db_path,
        sectors,
    })
}
