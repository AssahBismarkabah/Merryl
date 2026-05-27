use std::path::Path;

use anyhow::Result;

use crate::config::{paths, quality, scoring, universe};
use crate::data::AlpacaProvider;
use crate::storage::{
    DataQualitySnapshot, Database, RequiredPriceCoverage, RequiredSymbolCoverage, default_db_path,
};

use super::messages;

pub fn doctor() -> Result<Vec<String>> {
    doctor_for_db_path(&default_db_path())
}

pub fn doctor_for_db_path(db_path: &Path) -> Result<Vec<String>> {
    let mut checks = Vec::new();
    checks.push(messages::cli_name_check());

    for path in paths::REQUIRED_DOCS {
        checks.push(path_check(path));
    }

    checks.extend(AlpacaProvider::env_status());
    checks.push(path_check(paths::DAILY_WORKFLOW_CONFIG));
    checks.push(generated_path_check(db_path));
    checks.push(generated_path_check(paths::REPORTS_DIR));
    checks.push(generated_path_check(paths::EXPORTS_DIR));
    checks.extend(data_quality_checks(db_path)?);

    Ok(checks)
}

fn path_check(path: &str) -> String {
    if Path::new(path).exists() {
        messages::ok(path)
    } else {
        messages::missing(path)
    }
}

fn generated_path_check(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    if path.exists() {
        messages::ok(path.display())
    } else {
        messages::not_created_yet(path.display())
    }
}

fn data_quality_checks(db_path: &Path) -> Result<Vec<String>> {
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let db = Database::open(db_path)?;
    db.migrate()?;
    let snapshot = db.data_quality_snapshot(&required_market_symbols(), universe::SECTOR_ETFS)?;

    Ok(data_quality_messages(&snapshot))
}

fn required_market_symbols() -> Vec<&'static str> {
    universe::required_market_symbols()
}

fn data_quality_messages(snapshot: &DataQualitySnapshot) -> Vec<String> {
    vec![
        required_symbol_message(&snapshot.symbol_coverage),
        sector_map_message(snapshot),
        price_coverage_message(snapshot),
        score_date_coverage_message(snapshot),
        latest_score_date_message(snapshot),
        latest_score_rows_message(snapshot),
    ]
}

fn required_symbol_message(coverage: &RequiredSymbolCoverage) -> String {
    let present = coverage.required_count - coverage.missing.len();
    if coverage.missing.is_empty() {
        messages::ok(format!(
            "required market symbols present ({present}/{})",
            coverage.required_count
        ))
    } else {
        messages::missing(format!(
            "required market symbols: {}",
            coverage.missing.join(", ")
        ))
    }
}

fn sector_map_message(snapshot: &DataQualitySnapshot) -> String {
    if snapshot.missing_sector_maps.is_empty() {
        messages::ok(format!(
            "required sector map entries present ({}/{})",
            universe::SECTOR_ETFS.len(),
            universe::SECTOR_ETFS.len()
        ))
    } else {
        messages::missing(format!(
            "required sector map entries: {}",
            snapshot.missing_sector_maps.join(", ")
        ))
    }
}

fn price_coverage_message(snapshot: &DataQualitySnapshot) -> String {
    let insufficient = insufficient_price_coverage(snapshot);
    if insufficient.is_empty() {
        messages::ok(format!(
            "required ETF price coverage >= {} bars through {} ({}/{})",
            quality::MIN_REQUIRED_PRICE_BARS,
            optional_date(snapshot.latest_benchmark_price_date.as_deref()),
            snapshot.price_coverage.len(),
            snapshot.price_coverage.len()
        ))
    } else {
        messages::missing(format!(
            "required ETF price coverage >= {} bars through {}: {}",
            quality::MIN_REQUIRED_PRICE_BARS,
            optional_date(snapshot.latest_benchmark_price_date.as_deref()),
            insufficient.join(", ")
        ))
    }
}

fn insufficient_price_coverage(snapshot: &DataQualitySnapshot) -> Vec<String> {
    snapshot
        .price_coverage
        .iter()
        .filter(|coverage| {
            coverage.bar_count < quality::MIN_REQUIRED_PRICE_BARS
                || coverage.latest_date != snapshot.latest_benchmark_price_date
        })
        .map(price_coverage_summary)
        .collect()
}

fn price_coverage_summary(coverage: &RequiredPriceCoverage) -> String {
    format!(
        "{} bars={}, first={}, latest={}",
        coverage.symbol,
        coverage.bar_count,
        optional_date(coverage.first_date.as_deref()),
        optional_date(coverage.latest_date.as_deref())
    )
}

fn score_date_coverage_message(snapshot: &DataQualitySnapshot) -> String {
    if snapshot.score_dates >= quality::MIN_REQUIRED_SCORE_DATES {
        messages::ok(format!(
            "historical score coverage {} dates",
            snapshot.score_dates
        ))
    } else {
        messages::missing(format!(
            "historical score coverage {} dates, need at least {}",
            snapshot.score_dates,
            quality::MIN_REQUIRED_SCORE_DATES
        ))
    }
}

fn latest_score_date_message(snapshot: &DataQualitySnapshot) -> String {
    match (
        snapshot.latest_score_date.as_deref(),
        snapshot.latest_benchmark_price_date.as_deref(),
    ) {
        (Some(score_date), Some(price_date)) if score_date == price_date => messages::ok(format!(
            "latest score date matches benchmark price date ({score_date})"
        )),
        (Some(score_date), Some(price_date)) => messages::missing(format!(
            "latest score date {score_date} does not match benchmark price date {price_date}"
        )),
        (None, Some(price_date)) => messages::missing(format!(
            "latest score date missing; benchmark price date is {price_date}"
        )),
        (Some(score_date), None) => messages::missing(format!(
            "benchmark price date missing; latest score date is {score_date}"
        )),
        (None, None) => messages::missing("latest score date and benchmark price date"),
    }
}

fn latest_score_rows_message(snapshot: &DataQualitySnapshot) -> String {
    let expected_sector_rows = universe::SECTOR_ETFS.len() as i64;
    let expected_stock_rows = scoring::STOCK_WATCHLIST_LIMIT as i64;
    let expected_watchlist_rows = scoring::REPORT_WATCHLIST_LIMIT as i64;
    let coverage = &snapshot.latest_score_coverage;
    let has_expected_rows = coverage.market_regime_rows == 1
        && coverage.sector_rows == expected_sector_rows
        && coverage.industry_rows > 0
        && coverage.stock_rows == expected_stock_rows
        && coverage.watchlist_rows == expected_watchlist_rows;
    let Some(date) = snapshot.latest_score_date.as_deref() else {
        return messages::missing("latest score rows: no score date");
    };

    let summary = format!(
        "{date}: regime {}/1, sectors {}/{}, industries {}, stocks {}/{}, watchlist {}/{}",
        coverage.market_regime_rows,
        coverage.sector_rows,
        expected_sector_rows,
        coverage.industry_rows,
        coverage.stock_rows,
        expected_stock_rows,
        coverage.watchlist_rows,
        expected_watchlist_rows
    );

    if has_expected_rows {
        messages::ok(format!("latest score rows {summary}"))
    } else {
        messages::missing(format!("latest score rows {summary}"))
    }
}

fn optional_date(value: Option<&str>) -> &str {
    value.unwrap_or("none")
}
