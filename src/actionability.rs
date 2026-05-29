use serde_json::{Value, json};

use crate::config::{actionability as actionability_config, scoring};
use crate::domain::models::StockScore;

#[derive(Debug, Clone, PartialEq)]
pub struct ActionabilityClassification {
    pub primary: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ActionabilityMetrics {
    pub ma_20d: f64,
    pub ma_50d: f64,
    pub distance_from_20d_ma_pct: f64,
    pub distance_from_50d_ma_pct: f64,
    pub atr_14d: f64,
    pub atr_14d_pct: f64,
    pub atr_extension_from_20d_ma: f64,
    pub atr_extension_from_50d_ma: f64,
    pub high_20d: f64,
    pub high_60d: f64,
    pub distance_from_20d_high_pct: f64,
    pub distance_from_60d_high_pct: f64,
    pub range_10d_pct: f64,
    pub gap_pct: f64,
    pub true_range_pct: f64,
    pub complete: bool,
}

#[derive(Debug, Clone)]
pub struct ActionabilityInput<'a> {
    pub score: f64,
    pub sector_score: f64,
    pub return_1d: f64,
    pub return_5d: f64,
    pub relative_return_vs_sector: f64,
    pub relative_return_vs_spy: f64,
    pub relative_volume: f64,
    pub trend_state: &'a str,
    pub catalyst_status: &'a str,
}

impl<'a> ActionabilityInput<'a> {
    pub fn from_stock(stock: &'a StockScore) -> Self {
        Self {
            score: stock.score,
            sector_score: stock.sector_score,
            return_1d: stock.return_1d,
            return_5d: stock.return_5d,
            relative_return_vs_sector: stock.relative_return_vs_sector,
            relative_return_vs_spy: stock.relative_return_vs_spy,
            relative_volume: stock.relative_volume,
            trend_state: &stock.trend_state,
            catalyst_status: &stock.catalyst_status,
        }
    }
}

pub fn classify_actionability(
    input: &ActionabilityInput<'_>,
    metrics: &ActionabilityMetrics,
) -> ActionabilityClassification {
    if !metrics.complete {
        return fallback_classification(input.catalyst_status);
    }

    let mut labels = Vec::new();
    let extended = is_extended(input, metrics);
    if extended {
        labels.push(actionability_config::LABEL_EXTENDED_LEADER.to_string());
    }

    let relative_leader =
        input.relative_return_vs_sector > 0.0 || input.relative_return_vs_spy > 0.0;
    let above_trend = input.trend_state == "above_20d_50d" || input.trend_state == "above_20d";
    let near_20d_ma =
        metrics.distance_from_20d_ma_pct.abs() <= actionability_config::NEAR_20D_MA_MAX_DISTANCE;
    let near_50d_ma =
        metrics.distance_from_50d_ma_pct.abs() <= actionability_config::NEAR_50D_MA_MAX_DISTANCE;
    let controlled_pullback = metrics.distance_from_20d_high_pct
        >= actionability_config::PULLBACK_FROM_20D_HIGH_MIN
        && metrics.distance_from_20d_high_pct <= actionability_config::PULLBACK_FROM_20D_HIGH_MAX;

    if !extended
        && relative_leader
        && controlled_pullback
        && above_trend
        && (near_20d_ma || near_50d_ma)
    {
        labels.push(actionability_config::LABEL_PULLBACK_LEADER.to_string());
    }

    if !extended
        && relative_leader
        && metrics.range_10d_pct <= actionability_config::COMPRESSION_10D_RANGE_MAX
        && metrics.distance_from_20d_high_pct >= actionability_config::PULLBACK_FROM_20D_HIGH_MAX
    {
        labels.push(actionability_config::LABEL_BASE_COMPRESSION_CANDIDATE.to_string());
    }

    if !extended
        && relative_leader
        && input.return_5d > 0.0
        && input.return_5d < actionability_config::EARLY_ROTATION_5D_MAX_RETURN
        && (near_20d_ma
            || near_50d_ma
            || (-0.02..=0.08).contains(&metrics.distance_from_20d_ma_pct))
    {
        labels.push(actionability_config::LABEL_EARLY_ROTATION_CANDIDATE.to_string());
    }

    let strong_context = input.score >= actionability_config::STRONG_SCORE_MIN
        || input.sector_score >= actionability_config::STRONG_SCORE_MIN;
    let not_extreme = metrics.distance_from_20d_ma_pct
        < actionability_config::EXTENDED_DISTANCE_20D_MA
        && metrics.distance_from_50d_ma_pct < actionability_config::EXTENDED_DISTANCE_50D_MA
        && metrics.atr_extension_from_20d_ma
            < actionability_config::EXTENDED_ATR_MULTIPLE_FROM_20D_MA;
    if !extended
        && strong_context
        && relative_leader
        && input.trend_state == "above_20d_50d"
        && not_extreme
    {
        labels.push(actionability_config::LABEL_ACTIONABLE_LEADER.to_string());
    }

    if has_event_context(input.catalyst_status) && !has_useful_price_label(&labels) {
        labels.push(actionability_config::LABEL_EVENT_WATCH_UNCONFIRMED.to_string());
    }

    if labels.is_empty() {
        labels.push(actionability_config::LABEL_UNCLASSIFIED_LEADER.to_string());
    }

    labels.dedup();
    let primary = primary_label(&labels);
    ActionabilityClassification { primary, labels }
}

pub fn classification_for_stock(stock: &StockScore) -> ActionabilityClassification {
    classify_actionability(
        &ActionabilityInput::from_stock(stock),
        &metrics_from_components(&stock.components_json),
    )
}

pub fn stored_classification_from_components(raw: &str) -> ActionabilityClassification {
    let value = json_value(raw);
    let labels = value
        .get(actionability_config::COMPONENT_ACTIONABILITY_LABELS)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|labels| !labels.is_empty())
        .unwrap_or_else(|| vec![actionability_config::LABEL_UNCLASSIFIED_LEADER.to_string()]);
    let primary = value
        .get(actionability_config::COMPONENT_PRIMARY_ACTIONABILITY)
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| primary_label(&labels));
    ActionabilityClassification { primary, labels }
}

pub fn refresh_stock_components(stock: &mut StockScore) {
    let metrics = metrics_from_components(&stock.components_json);
    let classification = classify_actionability(&ActionabilityInput::from_stock(stock), &metrics);
    let mut value = json_value(&stock.components_json);
    if !value.is_object() {
        value = json!({});
    }
    if let Some(map) = value.as_object_mut() {
        map.insert(
            actionability_config::COMPONENT_ACTIONABILITY_LABELS.to_string(),
            json!(classification.labels),
        );
        map.insert(
            actionability_config::COMPONENT_PRIMARY_ACTIONABILITY.to_string(),
            json!(classification.primary),
        );
    }
    stock.components_json = value.to_string();
}

pub fn metrics_from_components(raw: &str) -> ActionabilityMetrics {
    let value = json_value(raw);
    let required_keys = [
        actionability_config::COMPONENT_DISTANCE_FROM_20D_MA_PCT,
        actionability_config::COMPONENT_DISTANCE_FROM_50D_MA_PCT,
        actionability_config::COMPONENT_ATR_EXTENSION_FROM_20D_MA,
        actionability_config::COMPONENT_DISTANCE_FROM_20D_HIGH_PCT,
        actionability_config::COMPONENT_RANGE_10D_PCT,
    ];

    ActionabilityMetrics {
        ma_20d: component_f64(&value, actionability_config::COMPONENT_MA_20D),
        ma_50d: component_f64(&value, actionability_config::COMPONENT_MA_50D),
        distance_from_20d_ma_pct: component_f64(
            &value,
            actionability_config::COMPONENT_DISTANCE_FROM_20D_MA_PCT,
        ),
        distance_from_50d_ma_pct: component_f64(
            &value,
            actionability_config::COMPONENT_DISTANCE_FROM_50D_MA_PCT,
        ),
        atr_14d: component_f64(&value, actionability_config::COMPONENT_ATR_14D),
        atr_14d_pct: component_f64(&value, actionability_config::COMPONENT_ATR_14D_PCT),
        atr_extension_from_20d_ma: component_f64(
            &value,
            actionability_config::COMPONENT_ATR_EXTENSION_FROM_20D_MA,
        ),
        atr_extension_from_50d_ma: component_f64(
            &value,
            actionability_config::COMPONENT_ATR_EXTENSION_FROM_50D_MA,
        ),
        high_20d: component_f64(&value, actionability_config::COMPONENT_HIGH_20D),
        high_60d: component_f64(&value, actionability_config::COMPONENT_HIGH_60D),
        distance_from_20d_high_pct: component_f64(
            &value,
            actionability_config::COMPONENT_DISTANCE_FROM_20D_HIGH_PCT,
        ),
        distance_from_60d_high_pct: component_f64(
            &value,
            actionability_config::COMPONENT_DISTANCE_FROM_60D_HIGH_PCT,
        ),
        range_10d_pct: component_f64(&value, actionability_config::COMPONENT_RANGE_10D_PCT),
        gap_pct: component_f64(&value, actionability_config::COMPONENT_GAP_PCT),
        true_range_pct: component_f64(&value, actionability_config::COMPONENT_TRUE_RANGE_PCT),
        complete: required_keys.iter().all(|key| value.get(*key).is_some()),
    }
}

pub fn components_with_actionability(
    base: Value,
    input: &ActionabilityInput<'_>,
    metrics: &ActionabilityMetrics,
) -> Value {
    let classification = classify_actionability(input, metrics);
    let mut value = if base.is_object() { base } else { json!({}) };
    if let Some(map) = value.as_object_mut() {
        if metrics.complete {
            map.insert(
                actionability_config::COMPONENT_MA_20D.to_string(),
                json!(metrics.ma_20d),
            );
            map.insert(
                actionability_config::COMPONENT_MA_50D.to_string(),
                json!(metrics.ma_50d),
            );
            map.insert(
                actionability_config::COMPONENT_DISTANCE_FROM_20D_MA_PCT.to_string(),
                json!(metrics.distance_from_20d_ma_pct),
            );
            map.insert(
                actionability_config::COMPONENT_DISTANCE_FROM_50D_MA_PCT.to_string(),
                json!(metrics.distance_from_50d_ma_pct),
            );
            map.insert(
                actionability_config::COMPONENT_ATR_14D.to_string(),
                json!(metrics.atr_14d),
            );
            map.insert(
                actionability_config::COMPONENT_ATR_14D_PCT.to_string(),
                json!(metrics.atr_14d_pct),
            );
            map.insert(
                actionability_config::COMPONENT_ATR_EXTENSION_FROM_20D_MA.to_string(),
                json!(metrics.atr_extension_from_20d_ma),
            );
            map.insert(
                actionability_config::COMPONENT_ATR_EXTENSION_FROM_50D_MA.to_string(),
                json!(metrics.atr_extension_from_50d_ma),
            );
            map.insert(
                actionability_config::COMPONENT_HIGH_20D.to_string(),
                json!(metrics.high_20d),
            );
            map.insert(
                actionability_config::COMPONENT_HIGH_60D.to_string(),
                json!(metrics.high_60d),
            );
            map.insert(
                actionability_config::COMPONENT_DISTANCE_FROM_20D_HIGH_PCT.to_string(),
                json!(metrics.distance_from_20d_high_pct),
            );
            map.insert(
                actionability_config::COMPONENT_DISTANCE_FROM_60D_HIGH_PCT.to_string(),
                json!(metrics.distance_from_60d_high_pct),
            );
            map.insert(
                actionability_config::COMPONENT_RANGE_10D_PCT.to_string(),
                json!(metrics.range_10d_pct),
            );
            map.insert(
                actionability_config::COMPONENT_GAP_PCT.to_string(),
                json!(metrics.gap_pct),
            );
            map.insert(
                actionability_config::COMPONENT_TRUE_RANGE_PCT.to_string(),
                json!(metrics.true_range_pct),
            );
        }
        map.insert(
            actionability_config::COMPONENT_ACTIONABILITY_LABELS.to_string(),
            json!(classification.labels),
        );
        map.insert(
            actionability_config::COMPONENT_PRIMARY_ACTIONABILITY.to_string(),
            json!(classification.primary),
        );
    }
    value
}

fn is_extended(input: &ActionabilityInput<'_>, metrics: &ActionabilityMetrics) -> bool {
    input.return_5d >= actionability_config::EXTENDED_5D_RETURN
        || input.return_1d >= actionability_config::EXTENDED_1D_RETURN
        || metrics.gap_pct >= actionability_config::EXTENDED_GAP
        || metrics.distance_from_20d_ma_pct >= actionability_config::EXTENDED_DISTANCE_20D_MA
        || metrics.distance_from_50d_ma_pct >= actionability_config::EXTENDED_DISTANCE_50D_MA
        || metrics.atr_extension_from_20d_ma
            >= actionability_config::EXTENDED_ATR_MULTIPLE_FROM_20D_MA
}

fn has_event_context(catalyst_status: &str) -> bool {
    catalyst_status != scoring::CATALYST_PENDING_SOURCE
}

fn has_useful_price_label(labels: &[String]) -> bool {
    labels.iter().any(|label| {
        label == actionability_config::LABEL_PULLBACK_LEADER
            || label == actionability_config::LABEL_BASE_COMPRESSION_CANDIDATE
            || label == actionability_config::LABEL_EARLY_ROTATION_CANDIDATE
            || label == actionability_config::LABEL_ACTIONABLE_LEADER
    })
}

fn fallback_classification(catalyst_status: &str) -> ActionabilityClassification {
    let labels = if has_event_context(catalyst_status) {
        vec![actionability_config::LABEL_EVENT_WATCH_UNCONFIRMED.to_string()]
    } else {
        vec![actionability_config::LABEL_UNCLASSIFIED_LEADER.to_string()]
    };
    let primary = primary_label(&labels);
    ActionabilityClassification { primary, labels }
}

fn primary_label(labels: &[String]) -> String {
    actionability_config::PRIMARY_PRIORITY
        .iter()
        .find(|priority| labels.iter().any(|label| label == **priority))
        .unwrap_or(&actionability_config::LABEL_UNCLASSIFIED_LEADER)
        .to_string()
}

fn json_value(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or(Value::Null)
}

fn component_f64(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or_default()
}
