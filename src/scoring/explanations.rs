use crate::config::scoring as scoring_config;

pub fn sector_explanation(
    sector: &str,
    score: f64,
    return_20d: f64,
    relative_return: f64,
    relative_volume: f64,
    breadth_component: f64,
) -> String {
    format!(
        "{sector} scored {:.1}: 20D return {}, relative return vs SPY {}, relative volume {:.2}x, breadth {:.0}%.",
        score,
        pct(return_20d),
        pct(relative_return),
        relative_volume,
        breadth_component
    )
}

pub fn stock_explanation(
    symbol: &str,
    score: f64,
    sector: &str,
    relative_return_vs_sector: f64,
    relative_volume: f64,
) -> String {
    format!(
        "{symbol} scored {:.1}: sector {sector}, 20D relative return vs sector {}, relative volume {:.2}x.",
        score,
        pct(relative_return_vs_sector),
        relative_volume
    )
}

fn pct(value: f64) -> String {
    format!("{:.2}%", value * scoring_config::PERCENT_SCALE)
}
