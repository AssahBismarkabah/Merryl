use std::fs;

use anyhow::Result;

use merryl::config::macro_data;
use merryl::domain::models::{MacroObservation, MarketRegimeScore, SectorScore};
use merryl::output::write_macro_regime_validation_outputs;
use merryl::validation::{MacroRegimeValidationInput, run_macro_regime_validation};

#[test]
fn macro_regime_validation_uses_as_of_context_and_flags_disagreements() -> Result<()> {
    let metrics = run_macro_regime_validation(stress_then_calm_input())?;

    assert_eq!(metrics.score_date_count, 2);
    assert_eq!(metrics.complete_macro_snapshot_count, 2);
    assert_eq!(metrics.missing_macro_snapshot_count, 0);
    assert_eq!(metrics.risk_on_with_stress_count, 1);
    assert_eq!(metrics.defensive_or_mixed_with_improving_count, 1);

    let volatility = flag(&metrics.flag_summaries, "volatility_stress");
    assert_eq!(volatility.active_dates, 1);
    assert_eq!(volatility.risk_on_dates_when_active, 1);

    let vix_freshness = metrics
        .series_freshness
        .iter()
        .find(|row| row.series == "VIXCLS")
        .expect("VIX freshness row");
    assert_eq!(vix_freshness.score_dates_covered, 2);
    assert_eq!(vix_freshness.missing_score_dates, 0);

    let technology_under_volatility = metrics
        .sector_leadership
        .iter()
        .find(|row| row.flag == "volatility_stress" && row.sector == "Technology")
        .expect("technology leadership under volatility stress");
    assert_eq!(technology_under_volatility.top_rank_count, 1);

    assert!(
        metrics
            .disagreement_examples
            .iter()
            .any(|example| example.reason.contains("risk-on"))
    );
    assert!(
        metrics
            .disagreement_examples
            .iter()
            .any(|example| example.reason.contains("defensive or mixed"))
    );

    Ok(())
}

#[test]
fn macro_regime_validation_does_not_use_future_macro_observations() -> Result<()> {
    let mut input = calm_input_for_date("2026-01-02");
    input.macro_observations.extend(observations_for_date(
        "2026-01-03",
        &[
            ("VIXCLS", 80.0),
            ("DGS10", 5.0),
            ("DGS2", 4.0),
            ("T10Y2Y", -1.0),
            ("DFF", 4.0),
            ("CPIAUCSL", 400.0),
            ("UNRATE", 8.0),
            ("PAYEMS", 90.0),
            ("BAMLC0A0CM", 5.0),
            ("DTWEXBGS", 120.0),
            ("WALCL", 6000.0),
        ],
    ));

    let metrics = run_macro_regime_validation(input)?;

    assert_eq!(
        flag(&metrics.flag_summaries, "volatility_stress").active_dates,
        0
    );
    assert_eq!(metrics.risk_on_with_stress_count, 0);

    Ok(())
}

#[test]
fn macro_regime_validation_writes_markdown_and_csv_outputs() -> Result<()> {
    let metrics = run_macro_regime_validation(stress_then_calm_input())?;
    let outputs = write_macro_regime_validation_outputs(&metrics)?;

    assert!(outputs.report.exists());
    assert!(outputs.summary_export.exists());
    let report = fs::read_to_string(&outputs.report)?;
    assert!(report.contains("Macro Regime Validation"));
    assert!(report.contains("Date alignment rule"));
    assert!(report.contains("Revision limitation"));
    assert!(report.contains("Macro Flag Summary"));
    assert!(report.contains("not a trading recommendation"));

    let _ = fs::remove_file(outputs.report);
    let _ = fs::remove_file(outputs.summary_export);

    Ok(())
}

fn stress_then_calm_input() -> MacroRegimeValidationInput {
    MacroRegimeValidationInput {
        from_date: "2026-01-02".to_string(),
        to_date: "2026-01-03".to_string(),
        regime_scores: vec![
            regime("2026-01-02", "Risk-on", 76.0),
            regime("2026-01-03", "Mixed", 51.0),
        ],
        sector_scores: vec![
            sector_score("2026-01-02", "Technology", 1, 82.0),
            sector_score("2026-01-02", "Utilities", 2, 55.0),
            sector_score("2026-01-03", "Healthcare", 1, 68.0),
            sector_score("2026-01-03", "Technology", 2, 61.0),
        ],
        macro_observations: [
            observations_for_date(
                "2026-01-01",
                &[
                    ("VIXCLS", 15.0),
                    ("DGS10", 4.0),
                    ("DGS2", 3.5),
                    ("T10Y2Y", 0.5),
                    ("DFF", 4.0),
                    ("CPIAUCSL", 300.0),
                    ("UNRATE", 4.0),
                    ("PAYEMS", 100.0),
                    ("BAMLC0A0CM", 1.0),
                    ("DTWEXBGS", 100.0),
                    ("WALCL", 7000.0),
                ],
            ),
            observations_for_date(
                "2026-01-02",
                &[
                    ("VIXCLS", 25.0),
                    ("DGS10", 4.5),
                    ("DGS2", 3.5),
                    ("T10Y2Y", -0.1),
                    ("DFF", 4.0),
                    ("CPIAUCSL", 301.0),
                    ("UNRATE", 4.1),
                    ("PAYEMS", 99.0),
                    ("BAMLC0A0CM", 1.5),
                    ("DTWEXBGS", 101.0),
                    ("WALCL", 6900.0),
                ],
            ),
            observations_for_date(
                "2026-01-03",
                &[
                    ("VIXCLS", 15.0),
                    ("DGS10", 4.0),
                    ("DGS2", 3.4),
                    ("T10Y2Y", 0.2),
                    ("DFF", 4.0),
                    ("CPIAUCSL", 301.0),
                    ("UNRATE", 4.1),
                    ("PAYEMS", 100.0),
                    ("BAMLC0A0CM", 1.2),
                    ("DTWEXBGS", 100.5),
                    ("WALCL", 6950.0),
                ],
            ),
        ]
        .concat(),
    }
}

fn calm_input_for_date(date: &str) -> MacroRegimeValidationInput {
    MacroRegimeValidationInput {
        from_date: date.to_string(),
        to_date: date.to_string(),
        regime_scores: vec![regime(date, "Risk-on", 75.0)],
        sector_scores: vec![sector_score(date, "Technology", 1, 80.0)],
        macro_observations: observations_for_date(
            date,
            &[
                ("VIXCLS", 15.0),
                ("DGS10", 4.0),
                ("DGS2", 3.5),
                ("T10Y2Y", 0.5),
                ("DFF", 4.0),
                ("CPIAUCSL", 300.0),
                ("UNRATE", 4.0),
                ("PAYEMS", 100.0),
                ("BAMLC0A0CM", 1.0),
                ("DTWEXBGS", 100.0),
                ("WALCL", 7000.0),
            ],
        ),
    }
}

fn observations_for_date(date: &str, values: &[(&str, f64)]) -> Vec<MacroObservation> {
    values
        .iter()
        .map(|(series, value)| macro_observation(series, date, *value))
        .collect()
}

fn macro_observation(series: &str, date: &str, value: f64) -> MacroObservation {
    let (_, series_name, frequency, units) = macro_data::MACRO_SERIES
        .iter()
        .find(|(candidate, _, _, _)| *candidate == series)
        .expect("known macro series");
    MacroObservation {
        series: series.to_string(),
        series_name: (*series_name).to_string(),
        date: date.to_string(),
        value,
        source: format!("fred:{series}"),
        frequency: (*frequency).to_string(),
        units: (*units).to_string(),
        realtime_start: date.to_string(),
        realtime_end: date.to_string(),
        raw_json: "{}".to_string(),
        quality_status: "ok".to_string(),
    }
}

fn regime(date: &str, label: &str, score: f64) -> MarketRegimeScore {
    MarketRegimeScore {
        date: date.to_string(),
        label: label.to_string(),
        score,
        spy_return_20d: 0.0,
        spy_return_60d: 0.0,
        qqq_relative_return_vs_spy: 0.0,
        iwm_relative_return_vs_spy: 0.0,
        dia_relative_return_vs_spy: 0.0,
        components_json: "{}".to_string(),
        explanation: "fixture regime".to_string(),
    }
}

fn sector_score(date: &str, sector: &str, rank: usize, score: f64) -> SectorScore {
    SectorScore {
        date: date.to_string(),
        sector: sector.to_string(),
        sector_etf: format!("{sector}ETF"),
        score,
        rank,
        return_1d: 0.0,
        return_5d: 0.0,
        return_20d: 0.0,
        return_60d: 0.0,
        relative_return_vs_spy: 0.0,
        relative_volume: 1.0,
        breadth_20d: 0.5,
        breadth_50d: 0.5,
        rank_change: 0.0,
        explanation: "fixture sector".to_string(),
    }
}

fn flag<'a>(
    rows: &'a [merryl::validation::MacroFlagSummary],
    name: &str,
) -> &'a merryl::validation::MacroFlagSummary {
    rows.iter()
        .find(|row| row.flag == name)
        .expect("macro flag summary")
}
