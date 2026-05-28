# Phase 4 Dashboard And API Spec

Version: 0.2
Date: 2026-05-27
Status: First read-only Phase 4 dashboard/API slice is implemented

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/implementation_spec.md`
- `docs/pre_dashboard_stability_backlog_spec.md`
- `docs/data_quality_reproducibility_spec.md`
- `docs/phase_4_dashboard_stabilization_spec.md`

## 1. Purpose

Phase 4 adds the first visual product surface for Merryl.

The dashboard must stay aligned with the core product intent:

```text
Show where market participation is concentrating,
map that concentration from market regime to sector to industry/theme to stock,
then produce a small list of liquid, chart-worthy names with explainable reasons.
```

The dashboard is not a trading terminal, portfolio simulator, alert engine, or auto-trading interface.

It should make the already-built market map easier to read:

```text
Market regime -> sector rotation -> industry/theme strength -> stock leadership -> watchlist
```

## 2. Current Starting Point

Phase 4 starts after PDB-1 through PDB-6 are complete.

Available foundation:

- SQLite is the local system of record.
- `run daily` writes historical regime, sector, industry, stock, and watchlist rows.
- `run backtest` writes validation metrics and summary outputs.
- `doctor` catches missing required symbols, price coverage, score-date coverage, latest score row coverage, and idempotent write risk.
- Markdown and CSV outputs exist, but they are not the final product interface.

Implemented first Phase 4 slice:

- Rust `axum` local server.
- `merryl dashboard` as the single dashboard entrypoint.
- SQLite read repositories for dashboard data.
- Dashboard DTOs separate from storage rows.
- Vite React TypeScript frontend under `dashboard/`.
- TanStack Table for data-dense market tables.
- Local static serving from `dashboard/dist`.
- Direct missing-data message: run `merryl run daily --date latest` first.

The implemented slice is intentionally read-only:

```text
SQLite stored results -> dashboard API DTOs -> browser dashboard
```

It does not fetch prices, run scoring, write reports, or mutate score tables.

## 3. Technology Decision

Use a local web dashboard first.

Recommended stack:

```text
Backend/API: Rust + axum + SQLite read repositories
Frontend: Vite + React + TypeScript
Data tables: TanStack Table
Financial charts: TradingView Lightweight Charts where charting is needed
Packaging: Browser/local web first; Tauri later if a packaged desktop app is needed
Avoid for first Phase 4 slice: Electron
```

## 4. Why Local Web First

Merryl is currently a local Rust engine with SQLite storage.

A local web dashboard is the cleanest next layer because it:

- keeps the Rust engine as the source of truth
- keeps SQLite local and inspectable
- avoids desktop packaging complexity before the UI is proven
- allows fast iteration on tables, filters, charts, and review flow
- can later be wrapped by Tauri without discarding the frontend
- avoids turning the CLI into a large command tree

Decision:

```text
Build the dashboard as a local browser app served by a small Rust API.
Do not start with Tauri or Electron.
```

## 5. Tauri Decision

Tauri is a good later packaging option because it aligns with a Rust core and web frontend.

Use Tauri later if Merryl needs:

- a packaged desktop app
- app menu/tray behavior
- native file dialogs
- desktop notifications
- controlled distribution outside a developer terminal

Do not start with Tauri in the first Phase 4 slice.

Reason:

```text
The immediate risk is not packaging.
The immediate risk is whether the dashboard reads and presents the market map correctly.
```

Tauri should be a wrapper after the local web dashboard proves useful.

## 6. Electron Decision

Electron is not the recommended first path.

Electron is strong for cross-platform desktop apps built around a JavaScript/Node runtime, but Merryl already has a Rust core and SQLite engine. Starting with Electron would introduce an additional desktop runtime and would likely move too much app logic toward JavaScript.

Use Electron only if a future requirement clearly needs the Electron ecosystem more than Rust/Tauri alignment.

Current decision:

```text
Do not use Electron for Phase 4.
```

## 7. Backend/API Shape

Add one local API layer.

The API should be read-only in the first Phase 4 slice.

It should:

- read from SQLite
- expose dashboard-ready JSON
- avoid fetching market data
- avoid recalculating scores
- avoid writing reports
- avoid mutating score tables
- bind to localhost only

Implemented public command:

```text
merryl dashboard
```

Behavior:

```text
Start local API/dashboard server.
Print the local URL.
Do not add subcommands for internal dashboard operations.
```

This keeps the CLI small and user-oriented.

## 8. First API Endpoints

First endpoints should match the dashboard views, not internal tables.

Implemented first-slice endpoints:

```text
GET /api/health
GET /api/dashboard/latest
GET /api/dates
GET /api/dashboard/:date
```

The dashboard snapshot endpoint returns the current market map in one DTO:

```text
market regime
sectors
industries/themes
stocks
watchlist
latest backtest result
data health
known limitations
```

Not implemented as separate endpoints in the first slice:

```text
GET /api/regime/latest
GET /api/sectors/latest
GET /api/industries/latest
GET /api/stocks/latest
GET /api/watchlist/latest
GET /api/backtests/latest
```

Those should be added only if the dashboard needs independent refresh, pagination, or filtering per view. For now, the single snapshot avoids a larger API surface and keeps the CLI/API layer small.

Endpoint behavior:

- `latest` endpoints use the latest scored date.
- `:date` endpoints read an already-scored date only.
- Missing data should return a direct message telling the user to run `merryl run daily --date latest`.
- Backtest endpoints read stored `backtest_results`; they do not run a backtest.

## 9. First Dashboard Views

First dashboard views are implemented as sidebar-selected views, not one long page:

```text
Overview
Market Regime
Sector Rotation
Industry/Theme Strength
Stock Leadership
Watchlist
Backtest Review
Data Health
```

The sidebar is the primary view switcher. Only the selected view is shown in the workspace. This prevents the dashboard from turning back into a document-style report.

### 9.1 Overview

Purpose:

```text
Show the current market map in one focused overview screen.
```

Must show:

- market regime label and score
- top sectors
- top industries/themes
- top watchlist names
- recent-news catalyst count where available

Do not repeat market date and latest price date on the same screen. Use one canonical label:

```text
Market date
```

### 9.2 Market Regime

Must show:

- regime label
- regime score
- SPY 20D and 60D return
- QQQ, IWM, DIA relative return vs SPY
- TLT, GLD, USO context values from components JSON
- coverage limits for macro context that is stored but not part of regime scoring yet

Implemented: regime label, score, SPY 20D/60D, QQQ/IWM/DIA relative return vs SPY, TLT/GLD/USO 20D context, and FRED macro coverage in Validation.

### 9.3 Sector Rotation

Must show:

- all 11 sectors
- rank
- score
- 1D, 5D, 20D, 60D return
- relative return vs SPY
- relative volume
- breadth
- rank change
- map-only note from PDB-2/PDB-3.5

### 9.4 Industry/Theme Strength

Must show:

- top industries/themes
- sector
- score
- rank
- component summary where available
- current grouping limitation in the Validation view

### 9.5 Stock Leadership

Must show:

- top scored stocks
- sector and industry
- score
- sector score
- relative strength vs sector
- relative strength vs SPY
- relative volume
- average dollar volume
- trend state
- catalyst/news status

### 9.6 Watchlist

Must show:

- top 25 chart-worthy names
- reason/explanation
- catalyst/news status
- watchlist limitation in the Validation view

### 9.7 Backtest Review

Must show:

- latest stored backtest date range
- validation scope
- sector observation count
- stock observation count
- industry validation observation count
- decile summaries
- hit-rate definition in compact validation metrics where available

### 9.8 Data Health

Must show:

- required symbol coverage
- required ETF price coverage
- score-date coverage
- latest score row coverage
- storage status
- current latest score date

This view can reuse the same checks behind `doctor`.

## 10. Frontend Decision

Use Vite + React + TypeScript for the first dashboard.

Reason:

- Vite supports React TypeScript scaffolding.
- React has broad ecosystem support for data-dense dashboards.
- TanStack Table works well for sortable/filterable market tables.
- TradingView Lightweight Charts is purpose-built for interactive financial charts.
- The frontend can later be wrapped by Tauri if needed.

Avoid heavy dashboard frameworks in the first slice.

Do not add:

- Next.js
- server-side rendering
- app-router complexity
- authentication
- user accounts
- cloud deployment
- drag-and-drop dashboard builders

Phase 4 is local and read-only.

Implemented frontend structure:

```text
dashboard/src/App.tsx                  loading and request state only
dashboard/src/api.ts                   API fetch boundary
dashboard/src/types.ts                 dashboard DTO types
dashboard/src/components/              page sections and reusable UI pieces
dashboard/src/tableColumns.ts          table column definitions
dashboard/src/format.ts                display formatting helpers
dashboard/src/view.ts                  dashboard view ids
dashboard/src/styles.css               visual system and responsive layout
```

This mirrors the server-side separation of concerns. The frontend should not collapse into one large dashboard file as more views are added.

## 11. UX Rules

The dashboard should feel like a market review workstation, not a marketing site.

Rules:

- dense but readable information layout
- no landing page
- no hero section
- no decorative cards around entire sections
- no chart-trading execution UI
- tables should be scannable and sortable
- sector/industry/stock rank changes should be visually clear
- unresolved limitations should be labeled, not hidden
- dark mode should be the default visual mode
- sidebar navigation should reveal one focused view at a time
- watchlist and chart list should not be duplicated; use one canonical `Watchlist`
- operational views should avoid helper or instructional copy; show labels and values
- market date and latest price date should not be shown as separate competing labels when they represent the same current data point
- coverage caveats and limitations belong in the Validation view
- Validation should show compact status rows and short state labels, not long internal run names, filesystem paths, or prose-heavy limits

Design direction checked against Refero:

```text
Primary layout reference: Stocktwits market overview dashboard.
Primary data-treatment reference: Glassnode indicator table/dashboard.
```

Additional visual reference checked:

```text
Good AI List: dark directory-style data surface with compact metric chips, flat tabs,
thin borders, dense tables, badge-based classification, and Chart.js horizontal
stacked bar charts for top-location rankings.
```

Applied constraints:

- left navigation and watchlist context
- central market overview and rotation path
- sidebar-selected views instead of showing every module on the same page
- compact table-first data panels
- flat directory-style tables over card-heavy layouts
- compact top metric chips for current market state
- badge treatments for sector, industry, ETF, and catalyst/status classification
- regime and validation views use the same compact table treatment as the market data views
- validation limits are compressed into area/current-state/next-step rows
- overview uses a Chart.js horizontal sector score chart inspired by the Good AI List city chart treatment
- sector chart uses the Good AI List-style dark chart panel: `#1a1a1a` surface, `#333333` border, compact title text, and dense horizontal bar spacing
- sidebar uses a clean Merryl wordmark until a real logo direction is locked; do not use a placeholder mark as if it were final branding
- dark analytical workspace with low-contrast surfaces
- thin borders and low decoration
- restrained teal/blue accent
- no marketing hero
- no report-like long document layout
- no decorative dashboard chrome
- no duplicate watchlist/chart-list modules
- no explanatory copy competing with operational market data

## 12. What Phase 4 Must Not Add

Do not add these in the first dashboard slice:

- alerts
- portfolio simulation
- trade execution
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

These remain later phases.

## 13. Implementation Order

Recommended implementation order:

1. Add read repository methods for dashboard summaries.
2. Add API DTOs that are separate from storage rows.
3. Add local `axum` API routes.
4. Add `merryl dashboard` as the single dashboard command.
5. Add Vite React TypeScript app.
6. Build the Overview and Data Health views first.
7. Add sector, industry, stock, watchlist, and backtest views.
8. Add charting only where the data already supports it.
9. Verify with tests and browser screenshots.

Do not build all views before proving the API boundary.

Current implementation status:

```text
1 through 7 are implemented for the first slice.
8 is partially implemented where existing dashboard data already supports it: the Overview uses a Chart.js sector score ranking chart from stored sector scores. Financial price charting remains deferred until chart data endpoints are explicitly scoped.
9 is complete for this slice through API tests, frontend build, and local Safari desktop/narrow viewport inspection.
```

## 14. Acceptance Criteria

Phase 4 first slice is accepted when:

- `merryl dashboard` starts a local dashboard server.
- The browser dashboard loads from local SQLite without fetching new market data.
- Overview shows latest regime, top sector, top industry, and top stock.
- Watchlist has its own sidebar-selected view.
- Data Health shows the same core checks that `doctor` verifies.
- Dashboard labels known limitations clearly.
- Tests cover API response shape and missing-data behavior.
- The frontend can be built reproducibly.
- No new public CLI subcommands are added beyond the one dashboard entrypoint.

Current acceptance status:

- `merryl dashboard` is implemented.
- Dashboard API reads from SQLite only.
- Overview shows the market map as one compact row per layer without duplicating the full watchlist or leadership tables.
- Sidebar-selected views show sectors, industries, leadership, watchlist, and validation separately.
- Data Health is surfaced from the same storage checks behind `doctor`.
- Known limitations are visible through compact Validation status rows.
- Tests cover API response shape and missing-data behavior.
- Frontend production build is reproducible with `npm --prefix dashboard run build`.
- No extra public CLI subcommands were added beyond `dashboard`.
- Local browser inspection confirmed the dashboard renders as a dark market-review workstation with sidebar-selected views and horizontally scrollable wide tables.

## 15. Evidence Checked

Primary sources checked during planning:

- Tauri architecture documentation: `https://tauri.app/concept/architecture/`
- Electron introduction: `https://www.electronjs.org/docs/latest`
- axum documentation: `https://docs.rs/axum/latest/axum/`
- Vite guide: `https://vite.dev/guide/`
- TanStack Table documentation: `https://tanstack.com/table/latest`
- TradingView Lightweight Charts documentation: `https://tradingview.github.io/lightweight-charts/`
- Refero design references: Stocktwits market overview dashboard for layout and Glassnode indicator dashboard for metric/table treatment.
- Good AI List: `https://goodailist.com/` for dark directory-style tables, compact chips, and badge-based classification.

## 16. Current Decision

Proceed with:

```text
Rust axum local API + Vite React TypeScript dashboard in the browser.
```

Do not start with:

```text
Tauri
Electron
```

Tauri remains the preferred later desktop-packaging path if the local web dashboard proves useful.
