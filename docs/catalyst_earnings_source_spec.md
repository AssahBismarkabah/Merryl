# Catalyst And Earnings Source Decision

Version: 0.1
Date: 2026-05-27
Status: PDB-4 complete; recent news connected, earnings calendar not connected

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/spec_completeness_gate_spec.md`

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
Structured earnings calendar data remains not connected.
If needed later, choose a separate provider such as Alpha Vantage, Finnhub, Polygon, or another licensed fundamentals/calendar source.
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

The daily report now shows:

```text
Catalyst / News Flags
```

The section includes source coverage and the latest headline for flagged symbols.

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

Future catalyst/earnings expansion can add:

- structured earnings calendar.
- earnings result and guidance parsing.
- analyst upgrades/downgrades.
- SEC filings.
- FDA/regulatory events.
- mergers/acquisitions.
- headline classification and relevance scoring.

Those should be handled as a later enrichment phase, not mixed into PDB-4.

## 7. Done When

PDB-4 is done when:

- Alpaca News is fetched with the existing Alpaca key. Complete.
- news rows are stored in `events`. Complete.
- current watchlist rows can show `recent_news:N`. Complete.
- the report no longer claims earnings calendar data exists. Complete.
- earnings calendar remains explicitly not connected. Complete.
- catalyst/news does not change scoring formulas. Complete.
