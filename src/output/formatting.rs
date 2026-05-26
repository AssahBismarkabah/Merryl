use crate::config::scoring;

pub fn pct(value: f64) -> String {
    format!("{:.2}%", value * scoring::PERCENT_SCALE)
}

pub fn score(value: f64) -> String {
    format!("{value:.1}")
}

pub fn multiple(value: f64) -> String {
    format!("{value:.2}x")
}
