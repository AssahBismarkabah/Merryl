use anyhow::Result;

use merryl::data::{AlphaVantageProvider, SecEdgarProvider};
use merryl::domain::models::{MarketEvent, MarketEventMetadata};
use merryl::scoring::catalyst_status_for_symbol;
use merryl::workflows::event_source_warning;

#[test]
fn catalyst_status_composes_news_earnings_filing_and_pending() {
    let events = vec![
        event("MSFT", "news", "2026-05-26", "Microsoft news"),
        event(
            "MSFT",
            "earnings",
            "2026-06-12",
            "Expected earnings for Microsoft Corporation",
        ),
        event(
            "MSFT",
            "filing",
            "2026-05-25",
            "8-K filed by Microsoft Corporation",
        ),
        event("AMD", "earnings", "2026-06-08", "Expected earnings for AMD"),
        event(
            "NVDA",
            "filing",
            "2026-05-24",
            "10-Q filed by NVIDIA Corporation",
        ),
    ];

    assert_eq!(
        catalyst_status_for_symbol("MSFT", &events),
        "recent_news:1 | earnings:2026-06-12 | filing:8-K"
    );
    assert_eq!(
        catalyst_status_for_symbol("AMD", &events),
        "earnings:2026-06-08"
    );
    assert_eq!(catalyst_status_for_symbol("NVDA", &events), "filing:10-Q");
    assert_eq!(
        catalyst_status_for_symbol("AAPL", &events),
        "pending_source"
    );
}

#[test]
fn alpha_vantage_parser_filters_requested_symbols_and_skips_malformed_rows() -> Result<()> {
    let symbols = vec!["MSFT".to_string(), "AMD".to_string()];
    let events = AlphaVantageProvider::parse_earnings_calendar_csv(
        &symbols,
        "symbol,name,reportDate,fiscalDateEnding,estimate,currency\n\
         MSFT,Microsoft Corporation,2026-06-12,2026-03-31,2.18,USD\n\
         AAPL,Apple Inc,2026-06-10,2026-03-31,1.50,USD\n\
         AMD,Advanced Micro Devices,not-a-date,2026-03-31,0.88,USD\n",
    )?;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].symbol, "MSFT");
    assert_eq!(events[0].event_type, "earnings");
    assert_eq!(events[0].event_date, "2026-06-12");
    assert_eq!(events[0].metadata.estimate, Some(2.18));
    assert_eq!(
        events[0].metadata.source_event_id.as_deref(),
        Some("alpha_vantage:earnings_calendar:MSFT:2026-06-12")
    );
    assert!(
        events[0]
            .metadata
            .raw_json
            .as_deref()
            .unwrap()
            .contains("MSFT")
    );

    Ok(())
}

#[test]
fn sec_submissions_parser_extracts_recent_target_filings_and_urls() -> Result<()> {
    let end_date = chrono::NaiveDate::from_ymd_opt(2026, 5, 28).expect("fixture date");
    let events = SecEdgarProvider::parse_submissions_json(
        "MSFT",
        789019,
        r#"{
            "name": "MICROSOFT CORP",
            "filings": {
                "recent": {
                    "accessionNumber": ["0000789019-26-000101", "0000789019-26-000100", "0000789019-26-000099"],
                    "filingDate": ["2026-05-27", "2026-05-20", "2026-04-01"],
                    "reportDate": ["2026-05-27", "2026-03-31", "2026-03-01"],
                    "acceptanceDateTime": ["20260527160100", "20260520160000", "20260401160000"],
                    "form": ["8-K", "10-Q", "4"],
                    "primaryDocument": ["msft-20260527.htm", "msft-20260331x10q.htm", "xslF345X05/doc4.xml"]
                }
            }
        }"#,
        end_date,
        14,
    )?;

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, "filing");
    assert_eq!(events[0].event_date, "2026-05-27");
    assert_eq!(
        events[0].metadata.source_event_id.as_deref(),
        Some("sec_edgar:submissions:0000789019:0000789019-26-000101")
    );
    assert_eq!(
        events[0].metadata.effective_date.as_deref(),
        Some("2026-05-27")
    );
    assert!(
        events[0]
            .url
            .as_deref()
            .unwrap()
            .contains("000078901926000101/msft-20260527.htm")
    );
    assert!(
        events
            .iter()
            .any(|event| event.headline.starts_with("10-Q"))
    );

    Ok(())
}

#[test]
fn event_source_warning_keeps_context_failure_nonfatal_and_visible() {
    let error = anyhow::anyhow!("operation timed out").context("failed to read response");

    assert_eq!(
        event_source_warning("Alpha Vantage earnings calendar", &error),
        "Alpha Vantage earnings calendar event source failed; continuing without Alpha Vantage earnings calendar context (failed to read response: operation timed out)"
    );
}

fn event(symbol: &str, event_type: &str, date: &str, headline: &str) -> MarketEvent {
    MarketEvent {
        symbol: symbol.to_string(),
        sector: Some("Technology".to_string()),
        event_date: date.to_string(),
        event_type: event_type.to_string(),
        headline: headline.to_string(),
        source: "test".to_string(),
        url: None,
        metadata: MarketEventMetadata::default(),
    }
}
