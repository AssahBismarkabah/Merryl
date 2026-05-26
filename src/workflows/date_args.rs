use anyhow::{Context, Result};
use chrono::NaiveDate;

pub fn parse_date_arg(date_arg: &str) -> Result<Option<NaiveDate>> {
    if date_arg == "latest" {
        return Ok(None);
    }

    NaiveDate::parse_from_str(date_arg, "%Y-%m-%d")
        .with_context(|| format!("invalid date `{date_arg}`; use YYYY-MM-DD or latest"))
        .map(Some)
}
