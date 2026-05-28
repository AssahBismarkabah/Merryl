# Catalyst And Earnings Source Decision

Version: 0.1
Date: 2026-05-27
Status: PDB-4 complete; Phase 5C later connected structured earnings-calendar and SEC filing-event context

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/spec_completeness_gate_spec.md`
- `docs/phase_5c_structured_catalyst_source_spec.md`

## 1. Purpose

PDB-4 exists because the original system must keep asking:

```text
Why is this moving?
```

Before this checkpoint, all stock rows carried:

```text
pending_source
```

That preserved the schema but did not connect a real catalyst source.

## 2. Decision

Connect real recent news catalysts now through the existing Alpaca Market Data credentials.

Decision:

```text
Use Alpaca Historical News for current watchlist catalyst context.
Do not add a new key for PDB-4.
Do not connect an earnings calendar yet.
Do not let catalyst/news change score formulas in this pass.
```

Primary source:

```text
Alpaca News endpoint: https://data.alpaca.markets/v1beta1/news
Official docs: https://docs.alpaca.markets/us/docs/streaming-real-time-news
```

Earnings calendar note:

```text
At PDB-4 time, structured earnings calendar data was not connected.
Phase 5C later selected Alpha Vantage as the free first structured earnings-calendar source.
```

## 3. What Changed

The daily workflow now:

1. Scores the historical market window as before.
2. Takes the current top stock watchlist.
3. Fetches recent Alpaca News for those symbols over the configured news lookback window.
4. Stores fetched news rows in `events`.
5. Marks affected stock rows as:

```text
recent_news:N
```

where `N` is the number of recent news events found for that symbol.

At PDB-4 time, the daily report showed:

```text
Catalyst / News Flags
```

After Phase 5C, the report section is `Catalyst / Event Flags`.

## 4. What This Does Not Claim

This is not a full news/NLP catalyst engine.

It does not:

- classify headline sentiment.
- decide whether news is bullish or bearish.
- parse earnings surprise, revenue, EPS, or guidance.
- model analyst estimate revisions.
- change sector, industry, or stock scores.
- predict trades.

It only provides real recent-news context so the report no longer implies catalyst data that does not exist.

## 5. Why This Matches The Source Specs

The main spec requires catalyst awareness but explicitly leaves full NLP news analysis for later.

This implementation preserves the central flow:

```text
Market regime -> sector -> industry/theme -> stock -> chart review
```

and adds a real "why might this be moving?" clue without expanding into a full advanced data layer.

## 6. Future Phase

The Phase 5C implementation document is now:

```text
docs/phase_5c_structured_catalyst_source_spec.md
```

The Phase 5C source decision stayed free-source first:

```text
Alpaca News
  -> existing recent-news context

Alpha Vantage Earnings Calendar
  -> upcoming earnings-date context if a free API key is provided

SEC EDGAR submissions
  -> recent filing-event context
```

Future catalyst/earnings expansion can add:

- earnings result and guidance parsing.
- analyst upgrades/downgrades.
- richer SEC filing interpretation.
- FDA/regulatory events.
- mergers/acquisitions.
- headline classification and relevance scoring.

Those should be handled as later enrichment phases, not mixed into PDB-4 or the first Phase 5C source connection.

## 7. Done When

PDB-4 is done when:

- Alpaca News is fetched with the existing Alpaca key. Complete.
- news rows are stored in `events`. Complete.
- current watchlist rows can show `recent_news:N`. Complete.
- the report no longer claims earnings calendar data exists. Complete.
- earnings calendar remains explicitly not connected. Complete.
- catalyst/news does not change scoring formulas. Complete.

Later Phase 5C update:

- Alpha Vantage Earnings Calendar is connected as upcoming earnings-date context.
- SEC EDGAR submissions are connected as recent filing-event context.
- The report section is now `Catalyst / Event Flags`.
- Catalyst/event context still does not change scoring formulas.
