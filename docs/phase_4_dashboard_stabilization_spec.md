# Phase 4.1 Dashboard Stabilization Spec

Version: 0.3
Date: 2026-05-28
Status: In progress; selected-date review, dashboard data-fidelity tests, and daily review ergonomics are implemented

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/phase_4_dashboard_api_spec.md`

## 1. Purpose

Phase 4.1 exists to stabilize the current dashboard before Merryl moves into advanced data layers.

The first Phase 4 slice is implemented:

```text
SQLite stored results -> read-only Rust API -> browser dashboard
```

The next milestone is not to add more market data or trading features. The next milestone is to make the current dashboard reliable enough for the daily review workflow:

```text
Market regime -> sector rotation -> industry/theme strength -> stock leadership -> watchlist
```

The dashboard must help answer:

```text
Where is market participation concentrating now,
which sectors and industries are active,
which stocks deserve chart review,
and what limitations should I keep in mind?
```

## 2. Current Position

Completed foundation:

- PDB-1 through PDB-6 are complete.
- Daily scoring stores historical regime, sector, industry, stock, and watchlist rows.
- Backtesting stores score-behavior validation.
- `doctor` checks required data coverage.
- `merryl dashboard` serves a local read-only dashboard.
- The dashboard has sidebar-selected views for overview, regime, sectors, industries, leadership, watchlist, and validation.
- The dashboard uses stored SQLite results and does not fetch market data or recalculate scores.

Current documented priority:

```text
Phase 4 dashboard stabilization and controlled improvement.
```

## 3. Non-Negotiable Guardrails

Do not add Phase 5 features in this milestone:

- alerts
- trade execution
- portfolio simulation
- position sizing
- transaction costs or slippage modeling
- intraday/live data
- options flow
- dark-pool data
- ETF fund flows
- full macro provider
- custom AI theme engine
- multi-user authentication
- cloud deployment

Do not expand the public CLI.

Do not make the dashboard write to score tables, fetch market data, or run scoring.

Keep the dashboard as a reader over the controlled market-map chain.

## 4. Milestone Definition

Phase 4.1 is complete when the dashboard can be used as the primary daily review surface without needing to open Markdown reports first.

The user should be able to:

1. Open the dashboard.
2. See the current market date and regime.
3. See which sectors are leading.
4. See which industries/themes are leading inside the market.
5. See the strongest stocks and the watchlist.
6. Understand validation limits without reading source docs.
7. Review a stored historical score date if needed.
8. Trust that dashboard values match stored SQLite/report data.

## 5. Workstreams

### 5.1 Data Fidelity

Goal:

```text
Dashboard values must match stored SQLite data and generated reports.
```

Required checks:

- Verify overview top regime, sector, industry, stock, and watchlist values come from the same score date.
- Verify sector, industry, stock, and watchlist tables match API DTO values.
- Verify validation/data health values match `doctor` storage checks where applicable.
- Verify missing-data states tell the user to run `merryl run daily --date latest`.

Done when:

- API tests cover latest dashboard response shape.
- API tests cover selected-date dashboard response shape.
- API tests cover missing-data behavior.
- A dashboard fixture can prove that displayed summary values come from one score date.

Current implementation status:

- Latest dashboard response shape is covered.
- Selected-date dashboard response shape is covered.
- Missing-data behavior is covered.
- A two-date dashboard fixture proves selected dashboard values come from the requested score date.

### 5.2 Historical Date Review

Goal:

```text
Use existing `/api/dates` and `/api/dashboard/:date` support to review stored score dates without adding workflow commands.
```

Required behavior:

- The dashboard should expose a compact scored-date selector or equivalent view control.
- Selecting a date should load stored data for that date.
- The UI should keep one canonical date label: `Market date`.
- If a selected date has no stored dashboard data, show a direct missing-data state.

Done when:

- The selected date changes the dashboard snapshot.
- The URL/API boundary remains read-only.
- No new public CLI command is added.

Current implementation status:

- A compact `Market date` selector is implemented in the dashboard header.
- The selector uses existing `/api/dates` and `/api/dashboard/:date` endpoints.
- No new CLI command was added.

### 5.3 Daily Review Ergonomics

Goal:

```text
The dashboard should feel like an operational market-review workstation, not a report page.
```

Required checks:

- Overview summarizes the market map without duplicating watchlist/leadership concepts.
- Regime view uses compact market context and keeps full macro limits in Validation.
- Sector view clearly labels sector ranking as map-only.
- Industries view stays tied to sector context.
- Leadership and Watchlist views do not duplicate each other.
- Validation view uses status rows, not long explanatory prose.

Done when:

- Every view has a clear role.
- No view is mostly instructional text.
- No major table or chip overflows on narrow viewport.

Current implementation status:

- Overview now summarizes the market map as one row per layer instead of repeating full watchlist or leadership tables.
- Regime uses compact metric/status rows and leaves broader macro coverage limits in Validation.
- Sector Rotation is labeled as a map-only rotation layer.
- Validation uses compact Backtest Review, Data Health, and Coverage Limits tables.
- Narrow browser checks found and fixed Validation table containment so horizontal scroll stays inside table wrappers.

### 5.4 Visual Verification

Goal:

```text
The dashboard must remain readable on desktop and narrow/mobile widths.
```

Required checks:

- Desktop browser screenshot.
- Narrow/mobile browser screenshot.
- Sidebar-selected views render one focused workspace at a time.
- Tables remain horizontally scrollable when needed.
- Chart panels do not overflow or collapse.
- Text does not overlap or clip inside chips, buttons, table cells, or chart labels.

Done when:

- Browser screenshots confirm overview, validation, and at least one dense table view.
- `npm --prefix dashboard run build` passes.

### 5.5 Documentation Alignment

Goal:

```text
Docs must reflect the current phase and prevent feature drift.
```

Required updates:

- `implementation_spec.md` should point to this Phase 4.1 milestone as the current controlled improvement plan.
- `phase_4_dashboard_api_spec.md` should remain the first-slice API/dashboard source.
- This document should record the stabilization acceptance criteria.

Done when:

- No current doc says the first Phase 4 slice is still only a future next step.
- Phase 5 features remain documented as deferred.

## 6. Implementation Order

Recommended order:

1. Add tests for selected-date dashboard loading and dashboard data fidelity. Complete.
2. Add a compact scored-date selector using the existing dates API. Complete.
3. Tighten overview/regime/validation wording and layout only where needed. Complete.
4. Verify desktop and narrow browser layouts. In progress; production build and narrow browser checks for Overview/Regime/Validation are complete.
5. Update status documentation after implementation. Complete for this pass.

Do not start with new charts or new data sources.

## 7. Acceptance Criteria

Phase 4.1 is accepted when:

- Dashboard can load latest stored data.
- Dashboard can load a selected historical score date.
- Overview, regime, sector, industry, leadership, watchlist, and validation views render from the same selected score date.
- Known limitations remain visible.
- Sector ranking is not presented as a proven standalone forward-return signal.
- Backtest is presented as score-behavior validation, not trade profitability.
- No new public CLI command is added.
- API remains read-only.
- `cargo test` passes.
- `cargo clippy -- -D warnings` passes.
- `npm --prefix dashboard run build` passes.
- Browser visual checks pass on desktop and narrow viewport.

## 8. What Comes After

Only after Phase 4.1 is stable should Merryl consider the next milestone.

The next milestone should be chosen deliberately from:

- Phase 4.2: richer dashboard review workflow and financial price charts.
- Phase 5: advanced data source planning.
- Universe expansion planning.
- Regime/macro expansion planning.

Do not assume Phase 5 starts automatically.
