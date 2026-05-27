use serde_json::json;

use crate::config::scoring as scoring_config;
use crate::domain::models::MarketRegimeScore;

use super::indicators::{PriceHistories, clamp_score, pct_return};

pub fn score_market_regime(date: &str, histories: &PriceHistories) -> MarketRegimeScore {
    let spy_20d = pct_return(
        histories,
        scoring_config::BENCHMARK_SYMBOL,
        date,
        scoring_config::RETURN_20D,
    )
    .unwrap_or_default();
    let spy_60d = pct_return(
        histories,
        scoring_config::BENCHMARK_SYMBOL,
        date,
        scoring_config::RETURN_60D,
    )
    .unwrap_or_default();
    let qqq_relative = relative_to_spy(histories, scoring_config::GROWTH_SYMBOL, date, spy_20d);
    let iwm_relative = relative_to_spy(histories, scoring_config::SMALL_CAP_SYMBOL, date, spy_20d);
    let dia_relative = relative_to_spy(histories, scoring_config::INDUSTRIAL_SYMBOL, date, spy_20d);

    let spy_trend_component = clamp_score(
        scoring_config::NEUTRAL_SCORE
            + spy_20d * scoring_config::REGIME_TREND_SCORE_MULTIPLIER
            + spy_60d
                * scoring_config::REGIME_TREND_SCORE_MULTIPLIER
                * scoring_config::REGIME_SPY_60D_WEIGHT,
    );
    let qqq_component = relative_component(qqq_relative);
    let iwm_component = relative_component(iwm_relative);
    let dia_component = relative_component(dia_relative);

    let score = scoring_config::REGIME_SPY_TREND_WEIGHT * spy_trend_component
        + scoring_config::REGIME_QQQ_RELATIVE_WEIGHT * qqq_component
        + scoring_config::REGIME_IWM_RELATIVE_WEIGHT * iwm_component
        + scoring_config::REGIME_DIA_RELATIVE_WEIGHT * dia_component;
    let label = regime_label(score, spy_20d, spy_60d, qqq_relative, iwm_relative);
    let explanation = format!(
        "{label}: SPY 20D {}, SPY 60D {}, QQQ vs SPY {}, IWM vs SPY {}, DIA vs SPY {}.",
        pct(spy_20d),
        pct(spy_60d),
        pct(qqq_relative),
        pct(iwm_relative),
        pct(dia_relative)
    );

    MarketRegimeScore {
        date: date.to_string(),
        label,
        score,
        spy_return_20d: spy_20d,
        spy_return_60d: spy_60d,
        qqq_relative_return_vs_spy: qqq_relative,
        iwm_relative_return_vs_spy: iwm_relative,
        dia_relative_return_vs_spy: dia_relative,
        components_json: json!({
            "spy_trend_component": spy_trend_component,
            "qqq_relative_component": qqq_component,
            "iwm_relative_component": iwm_component,
            "dia_relative_component": dia_component,
            "source_note": "V1 uses broad ETF price proxies only; VIX, TLT, DXY, macro surprises, and rates can be added when sources are connected."
        })
        .to_string(),
        explanation,
    }
}

fn relative_to_spy(
    histories: &PriceHistories,
    symbol: &str,
    date: &str,
    spy_return_20d: f64,
) -> f64 {
    pct_return(histories, symbol, date, scoring_config::RETURN_20D)
        .map(|symbol_return| symbol_return - spy_return_20d)
        .unwrap_or_default()
}

fn relative_component(relative_return: f64) -> f64 {
    clamp_score(
        scoring_config::NEUTRAL_SCORE
            + relative_return * scoring_config::REGIME_RELATIVE_SCORE_MULTIPLIER,
    )
}

fn regime_label(
    score: f64,
    spy_20d: f64,
    spy_60d: f64,
    qqq_relative: f64,
    iwm_relative: f64,
) -> String {
    if score >= scoring_config::REGIME_RISK_ON_THRESHOLD
        && spy_20d >= 0.0
        && (qqq_relative >= 0.0 || iwm_relative >= 0.0)
    {
        "Risk-on".to_string()
    } else if score <= scoring_config::REGIME_RISK_OFF_THRESHOLD && spy_20d < 0.0 && spy_60d < 0.0 {
        "Risk-off".to_string()
    } else if spy_20d >= 0.0 && qqq_relative < 0.0 && iwm_relative < 0.0 {
        "Defensive".to_string()
    } else {
        "Mixed".to_string()
    }
}

fn pct(value: f64) -> String {
    format!("{:.2}%", value * scoring_config::PERCENT_SCALE)
}
