use anyhow::Result;

use merryl::data::FredProvider;

#[test]
fn fred_parser_skips_missing_values_and_preserves_provenance() -> Result<()> {
    let observations = FredProvider::parse_observations_json(
        "VIXCLS",
        "CBOE Volatility Index: VIX",
        "Daily",
        "Index",
        r#"{
            "realtime_start": "2026-05-28",
            "realtime_end": "2026-05-28",
            "observation_start": "2026-05-25",
            "observation_end": "2026-05-27",
            "units": "lin",
            "output_type": 1,
            "file_type": "json",
            "order_by": "observation_date",
            "sort_order": "asc",
            "count": 3,
            "offset": 0,
            "limit": 100000,
            "observations": [
                {
                    "realtime_start": "2026-05-28",
                    "realtime_end": "2026-05-28",
                    "date": "2026-05-25",
                    "value": "18.44"
                },
                {
                    "realtime_start": "2026-05-28",
                    "realtime_end": "2026-05-28",
                    "date": "2026-05-26",
                    "value": "."
                },
                {
                    "realtime_start": "2026-05-28",
                    "realtime_end": "2026-05-28",
                    "date": "2026-05-27",
                    "value": "19.15"
                }
            ]
        }"#,
    )?;

    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].series, "VIXCLS");
    assert_eq!(observations[0].series_name, "CBOE Volatility Index: VIX");
    assert_eq!(observations[0].date, "2026-05-25");
    assert_eq!(observations[0].value, 18.44);
    assert_eq!(observations[0].source, "fred:VIXCLS");
    assert_eq!(observations[0].frequency, "Daily");
    assert_eq!(observations[0].units, "Index");
    assert_eq!(observations[0].quality_status, "ok");
    assert!(observations[0].raw_json.contains(r#""series_id":"VIXCLS""#));

    assert_eq!(observations[1].date, "2026-05-27");
    assert_eq!(observations[1].value, 19.15);

    Ok(())
}
