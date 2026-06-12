# Static Dashboard Deployment Spec

Version: 0.1
Date: 2026-06-12
Status: Implemented deployment path; repository secrets and GitHub Pages settings must be configured before first hosted run

Related documents:

- `docs/market_rotation_system_spec.md`
- `docs/mvp_technical_plan_spec.md`
- `docs/phase_0_decisions_spec.md`
- `docs/phase_4_dashboard_api_spec.md`
- `docs/application_state_remaining_work_spec.md`
- `docs/implementation_spec.md`

## 1. Purpose

Merryl needs a zero-server deployment path that does not require a paid host, persistent cloud disk, or a live Rust web server.

The selected path is:

```text
GitHub Actions scheduled workflow
  -> run Merryl daily/intraday workflows
  -> build the dashboard in static mode
  -> export dashboard JSON snapshots from SQLite
  -> publish dashboard/dist to GitHub Pages
```

This keeps the core product boundary intact:

```text
Market regime
  -> sector rotation
    -> industry/theme strength
      -> stock leadership
        -> classified watchlist
          -> signal-only intraday readiness
            -> static read-only dashboard snapshot
```

## 2. Boundary

The static dashboard deployment is not a stateful cloud Merryl server.

It does not:

- host SQLite as a live database
- run the Rust `merryl dashboard` API server on GitHub Pages
- fetch provider data from the browser
- recalculate scores in the browser
- place trades, create alerts, size positions, or become a charting platform

It does:

- run the existing Rust workflows inside GitHub Actions
- use repository Secrets for provider credentials
- generate the same dashboard DTOs used by the local dashboard
- publish static JSON and frontend assets to GitHub Pages

## 3. Implementation

Implemented code path:

```text
merryl dashboard --export-static dashboard/dist/static-data
```

The export writes:

```text
dashboard/dist/static-data/dates.json
dashboard/dist/static-data/latest.json
dashboard/dist/static-data/dashboard/YYYY-MM-DD.json
```

The React dashboard keeps the normal local API mode by default.

Static mode is enabled at build time:

```text
VITE_MERRYL_STATIC_DASHBOARD=true
```

Vite receives the GitHub Pages base path through:

```text
base=./
```

That relative base is the default in `dashboard/vite.config.ts`, so the same static build can work under a GitHub Pages repository path or a local preview path. `MERRYL_DASHBOARD_BASE` remains available only if a future host needs an absolute base path.

## 4. GitHub Workflow

Implemented workflow:

```text
.github/workflows/static-dashboard.yml
```

The workflow runs on:

- manual dispatch
- weekday schedule at `23:30 UTC`

It performs:

```text
npm --prefix dashboard ci
cargo build
cargo run -- run daily --date latest
cargo run -- run intraday --date latest
cargo run -- doctor
cargo run -- status
VITE_MERRYL_STATIC_DASHBOARD=true npm --prefix dashboard run build
cargo run -- dashboard --export-static dashboard/dist/static-data
deploy dashboard/dist to GitHub Pages
```

## 5. Required Repository Settings

GitHub Pages must be configured to deploy from GitHub Actions.

Required repository Secrets:

```text
ALPACA_API_KEY_ID
ALPACA_API_SECRET_KEY
FRED_API_KEY
ALPHA_VANTAGE_API_KEY
MERRYL_SEC_USER_AGENT
```

Optional repository Variables:

```text
ALPACA_FEED
MERRYL_LOOKBACK_DAYS
MERRYL_ALPACA_REQUESTS_PER_MINUTE
MERRYL_INTRADAY_PROFILE_TIMEFRAME
MERRYL_INTRADAY_TRIGGER_TIMEFRAME
MERRYL_INTRADAY_CANDIDATE_LIMIT
MERRYL_INTRADAY_OPENING_RANGE_MINUTES
```

## 6. Tradeoffs

This path is acceptable because Merryl is currently a daily/snapshot decision-support tool, not a live trading terminal.

Tradeoffs:

- The hosted dashboard updates only when the scheduled/manual workflow completes.
- GitHub Actions storage is ephemeral; each run rebuilds the SQLite state from source data.
- GitHub scheduled workflows can be delayed, so this is not a real-time market system.
- If the repository's GitHub Pages site is public, the generated dashboard data is public.
- Large historical expansion may eventually need artifact/state caching or a real hosted database.

## 7. Acceptance

The static deployment path is accepted when:

- `npm --prefix dashboard run build` still works for local API mode.
- `VITE_MERRYL_STATIC_DASHBOARD=true npm --prefix dashboard run build` builds static mode.
- `cargo run -- dashboard --export-static dashboard/dist/static-data` writes all snapshot files from the existing SQLite database.
- The GitHub workflow publishes `dashboard/dist` to Pages after provider secrets are configured.

Reference docs:

- GitHub Pages with Actions: `https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages`
- GitHub Actions schedule syntax: `https://docs.github.com/en/actions/writing-workflows/workflow-syntax-for-github-actions#onschedule`
- Vite static deployment: `https://vite.dev/guide/static-deploy`
