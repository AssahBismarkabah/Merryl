# Phase 5C Structured Catalyst Source Spec

Version: 0.3
Date: 2026-05-28
Status: First implementation complete; live daily verification and event-context validation passed

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_5_data_source_expansion_spec.md`
- `docs/phase_5c_source_coverage_review_spec.md`
- `docs/phase_5c_event_context_validation_spec.md`
- `docs/phase_5_readiness_gate_spec.md`
- `docs/catalyst_earnings_source_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

Phase 5C improves Merryl's catalyst layer without changing Merryl's job.

Merryl remains:

```text
Market regime
  -> Sector rotation
    -> Industry/theme strength
      -> Stock leadership
        -> Watchlist for chart review elsewhere
```

The specific problem is:

```text
Recent news is connected, but structured earnings dates and filing events are not.
```

The goal is to improve the "why might this be moving?" context for current leaders and watchlist names.

This phase must not turn catalysts into automatic trade signals.

## 2. Hard Constraint

Use free sources only.

Decision:

```text
No paid catalyst, earnings, filings, options, or fund-flow source in Phase 5C.
```

Allowed:

- Free API keys.
- Public official APIs.
- Already connected sources.

Not allowed:

- Paid Finnhub plans.
- Paid Polygon/Massive datasets.
- Paid ETF Global fund flows.
- Paid Cboe DataShop datasets.
- Unlicensed scraping.
- Fake or mock catalyst data in production workflows.

## 3. Source Verification Snapshot

Checked on 2026-05-28.

| Source | Free status | Useful for | Phase 5C decision |
|---|---|---|---|
| Alpaca News | Already connected with existing Alpaca key | Recent headlines for watchlist names | Keep as the existing `recent_news:N` source. Do not replace it. |
| Alpha Vantage Earnings Calendar | Official docs expose `EARNINGS_CALENDAR` with a free API key. The endpoint can return the full scheduled earnings list for the next 3, 6, or 12 months. Free usage is limited to 25 requests/day. | Upcoming earnings dates and earnings-risk flags | Use as the first structured earnings calendar source if the user provides a free Alpha Vantage key. Fetch once per daily run and cache/store results. |
| Alpha Vantage Earnings History | Official docs expose `EARNINGS`, including quarterly EPS, analyst estimates, and surprise metrics. | Last earnings and historical surprise context | Defer from the daily workflow because it is per-symbol and the free limit is too low for broad S&P 500 daily backfills. Consider later for watchlist-only enrichment. |
| SEC EDGAR APIs | Official SEC APIs expose company submissions and XBRL company facts through public endpoints. SEC documents realtime updates for submissions and sub-minute typical XBRL processing delays, subject to peak-time delays and access policy compliance. | Recent 8-K, 10-Q, 10-K, and other filing events; slower official fundamental context | Use as the first filing-event source. Fetch recent submissions for current watchlist/top scored names, not the full universe. |
| Finnhub Earnings Calendar | Finnhub has a free tier, but pricing currently shows limited earnings-calendar access on the free plan and deeper history on paid plans. | Alternative earnings calendar | Do not use first. Revisit only if Alpha Vantage is unsuitable and free access is confirmed by a real key. |

Primary source URLs:

- Alpha Vantage support/free key and limits: `https://www.alphavantage.co/support/`
- Alpha Vantage earnings calendar docs: `https://www.alphavantage.co/documentation/`
- SEC EDGAR API docs: `https://www.sec.gov/search-filings/edgar-application-programming-interfaces`
- Finnhub pricing: `https://api.finnhub.io/pricing`

## 4. Decision

Implement Phase 5C with a free-source stack:

```text
Alpaca News
  -> already connected recent-news context

Alpha Vantage Earnings Calendar
  -> upcoming earnings date context

SEC EDGAR Submissions
  -> recent filing event context
```

Do not use Finnhub in the first implementation.

Reason:

- Alpha Vantage can return a full upcoming earnings calendar in one request, which fits the free daily limit.
- SEC EDGAR is official and public for filing metadata.
- Alpaca News is already working and should remain the recent-news layer.
- This combination improves catalyst context without paid dependencies.

## 5. What This Phase Should Show

The watchlist/report/dashboard should distinguish catalyst types clearly:

```text
news:N
earnings:YYYY-MM-DD
filing:8-K
unknown
```

Examples:

```text
recent_news:4 | earnings:2026-06-12
recent_news:1 | filing:8-K
earnings:2026-06-03
pending_source
```

The report should not say:

```text
This catalyst is bullish.
This stock should be bought before earnings.
This filing explains the move.
```

It should say:

```text
This stock has recent news, an upcoming earnings date, or a recent filing.
Review the chart and event risk manually.
```

## 6. Implementation Scope

### 6.1 Source Configuration

Add:

```text
ALPHA_VANTAGE_API_KEY
ALPHA_VANTAGE_API_URL=https://www.alphavantage.co
MERRYL_EARNINGS_CALENDAR_HORIZON=3month
MERRYL_SEC_FILINGS_LOOKBACK_DAYS=14
MERRYL_SEC_USER_AGENT=Merryl/0.1 your-email@example.com
```

Do not add a new public CLI command.

The existing workflow remains:

```text
merryl run daily --date latest
```

If Phase 5C is implemented and `ALPHA_VANTAGE_API_KEY` is missing, the workflow should fail directly instead of generating placeholder earnings rows.

SEC EDGAR does not require a user API key, but requests must use Merryl's configured user agent with contact information and respect SEC access policy.

### 6.2 Provider Boundaries

Add provider interfaces without exposing internal steps as CLI commands:

```text
EarningsCalendarProvider
FilingEventProvider
```

Recommended adapters:

```text
src/data/alpha_vantage.rs
src/data/sec_edgar.rs
```

Keep source-specific HTTP paths, query parameters, response parsing, and rate-limit behavior inside provider modules.

### 6.3 Storage

Use `events` as the canonical catalyst table.

Current `events` already stores news rows. Extend it additively instead of creating disconnected catalyst tables.

Additive columns to consider:

```text
event_time
source_event_id
effective_date
processed_at
fetched_at
actual
estimate
surprise
fiscal_period
raw_json
quality_status
```

Add an idempotency key or unique index so daily runs do not duplicate the same event.

Do not make the existing `filings` table a second production path. It can be left alone until a cleanup/migration decision is made, but Phase 5C should write filing events into the canonical `events` table.

### 6.4 Daily Workflow

Keep the public workflow unchanged.

Daily flow after Phase 5C:

```text
load universe
fetch Alpaca prices
fetch FRED macro context
score historical market window
fetch Alpaca recent news for current top watchlist
fetch Alpha Vantage earnings calendar once for configured horizon
filter earnings rows to current universe/watchlist
fetch SEC recent submissions for current top scored names
store events with source/provenance
compose catalyst status for current report date
write report/dashboard-ready data
```

Important scope boundary:

```text
Fetch SEC filings for current top scored names/watchlist first, not all S&P 500 names.
```

Reason:

- The daily report only needs catalyst context for leaders and chart-review candidates.
- SEC access should be respectful and bounded.
- Full-universe filing history can be a later batch/backfill phase if needed.

### 6.5 Catalyst Status Composition

Current:

```text
pending_source
recent_news:N
```

Target:

```text
pending_source
recent_news:N
earnings:YYYY-MM-DD
filing:FORM
recent_news:N | earnings:YYYY-MM-DD
recent_news:N | filing:FORM
recent_news:N | earnings:YYYY-MM-DD | filing:FORM
```

Keep the string compact for report/dashboard scanning.

Store full details in `events`, not in the `catalyst_status` string.

### 6.6 Report And Dashboard

Update the current Catalyst / News section to become:

```text
Catalyst / Event Flags
```

It should show:

- Source coverage: Alpaca News, Alpha Vantage Earnings Calendar, SEC EDGAR filings.
- Latest headline where recent news exists.
- Upcoming earnings date where available.
- Recent filing type/date/link where available.
- Explicit `unknown` or `pending_source` when no source event exists.

Do not add a new dashboard screen in this phase.

Add the structured context into the existing watchlist/validation surfaces.

## 7. What Stays Out

Out of scope for Phase 5C:

- Paid Finnhub.
- Paid Polygon/Massive.
- Earnings call transcripts.
- Analyst upgrade/downgrade feeds.
- NLP sentiment.
- Guidance parsing.
- Options flow.
- ETF fund flows.
- SEC full-text filing parsing.
- Full S&P 500 filing-history backfill.
- Any change to sector, industry, stock, or regime score weights.

Alpha Vantage `EARNINGS` history is also out of the first daily workflow because the free limit is too small for broad per-symbol calls.

It can be revisited later for watchlist-only enrichment if we prove the calendar layer is useful.

## 8. Validation Requirements

Before any catalyst data changes scoring, create a validation checkpoint.

First Phase 5C validation should only answer:

- Does structured event context cover current watchlist names?
- Does upcoming earnings context reduce avoidable event-risk surprises?
- Do recent filing flags help explain why a name is active?
- Is missing event data clearly labeled instead of treated as neutral?
- Does the source context improve review quality without implying an entry signal?

Backtest grouping can later compare:

```text
watchlist names with no event context
watchlist names with recent news
watchlist names with upcoming earnings
watchlist names with recent filings
watchlist names with multiple event types
```

Do not change score weights until that comparison exists.

## 9. Test Plan For Implementation

When implementing Phase 5C, add tests under `/tests`.

Required tests:

- Alpha Vantage earnings calendar CSV parser preserves source rows and skips malformed rows.
- SEC submissions parser extracts recent 8-K, 10-Q, 10-K, and filing URLs.
- `events` migration adds provenance/event columns idempotently.
- Event writes are idempotent across repeated daily runs.
- Catalyst status composition handles:
  - news only
  - earnings only
  - filing only
  - combined news + earnings + filing
  - no event context
- Daily report includes structured catalyst source coverage.
- Doctor reports configured source status without exposing API keys.

Live verification after implementation:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
npm --prefix dashboard run build
set -a; source .env; set +a; cargo run -- run daily --date latest
set -a; source .env; set +a; cargo run -- doctor
cargo run -- status
```

## 10. Implementation Order

Recommended order:

1. Add source config and `.env.example` entries for Alpha Vantage.
2. Add additive `events` provenance migration and idempotent event key.
3. Add Alpha Vantage earnings calendar provider and parser.
4. Add SEC CIK mapping and submissions provider.
5. Extend daily workflow internally; keep public CLI unchanged.
6. Compose structured catalyst status from stored events.
7. Update report/dashboard labels from news-only to event-aware.
8. Add doctor/status coverage for event sources.
9. Run full local and live verification.
10. Update `docs/implementation_spec.md` and this document with actual run results.

## 11. Implemented Boundary

Implemented on 2026-05-28:

- Added Alpha Vantage source configuration to `.env.example`.
- Added SEC filing source configuration to `.env.example`, including a required SEC user-agent contact string.
- Added `EarningsCalendarProvider` and `FilingEventProvider` provider boundaries.
- Added `src/data/alpha_vantage.rs` for the free Alpha Vantage `EARNINGS_CALENDAR` CSV endpoint.
- Added `src/data/sec_edgar.rs` for SEC company ticker mapping and submissions metadata.
- Extended `events` with provenance/event-context columns through additive SQLite migration.
- Added a unique source-event id index for idempotent structured event writes.
- Kept Alpaca News as the existing recent-news source.
- Kept the public workflow unchanged: `merryl run daily --date latest`.
- Added structured catalyst status composition:
  - `recent_news:N`
  - `earnings:YYYY-MM-DD`
  - `filing:FORM`
  - combined labels
  - `pending_source`
- Updated the daily report section to `Catalyst / Event Flags`.
- Updated dashboard limitation text so catalyst/event context is presented as source-backed context, not a score input.
- Added doctor source-status checks for Alpha Vantage and SEC EDGAR.
- Added status event row counts.

What remained unchanged:

- No new public CLI command.
- No paid data source.
- No fake production catalyst data.
- No sector, industry, stock, or regime scoring formula change.
- No dashboard charting/trading/portfolio feature.
- No full SEC text parsing or full-universe filing backfill.

## 12. Verification

Local verification completed on 2026-05-28:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
npm --prefix dashboard run build
set -a; source .env; set +a; cargo run -- doctor
cargo run -- status
```

Passing test coverage includes:

- Alpha Vantage earnings calendar CSV parser.
- SEC submissions parser for 8-K and 10-Q events.
- Event provenance migration idempotency.
- Structured event write idempotency.
- Catalyst status composition for news, earnings, filing, combined, and pending states.
- Daily report source coverage for structured catalyst/event flags.
- Existing dashboard, doctor, historical scoring, backtest, and storage tests.

Live daily verification completed on 2026-05-28:

```text
set -a; source .env; set +a; cargo run -- run daily --date latest
  -> date: 2026-05-28
  -> historical score dates: 228
  -> macro observations: 4847
  -> news events: 273
  -> earnings events: 49
  -> filing events: 40

set -a; source .env; set +a; cargo run -- doctor
  -> ok: Alpha Vantage API key is set
  -> ok: SEC EDGAR user agent is set (no API key required)
  -> ok: latest score date matches benchmark price date (2026-05-28)

cargo run -- status
  -> events: 362
```

`doctor` now reports the source status directly and does not expose API-key values.

Post-implementation coverage review:

```text
docs/phase_5c_source_coverage_review_spec.md
```

The coverage review accepted Phase 5C as a source-backed event-context layer for the current scored stock surface. It did not approve any catalyst/event scoring-weight change.

Follow-up validation checkpoint:

```text
docs/phase_5c_event_context_validation_spec.md
```

This checkpoint now validates stored catalyst/event labels against watchlist forward behavior before any catalyst/event scoring-weight change is considered.

Live result:

```text
Rows with event context: 25
Event-context forward observations: 0
```

Interpretation:

```text
Structured event labels are connected, but they do not yet have enough future bars for forward validation.
Do not change catalyst/event score weights yet.
```

## 13. Acceptance Criteria

Phase 5C is accepted when:

- No paid source is required.
- No fake production catalyst data exists.
- `merryl run daily --date latest` stores real news, earnings-calendar, and filing events where available.
- Watchlist catalyst labels distinguish news, earnings, filing, and unknown.
- Report/dashboard explain event context without implying trade signals.
- Source coverage and missing data are visible.
- Existing scoring formulas remain unchanged.
- Tests pass.
