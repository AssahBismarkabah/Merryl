# CLI Reference

Merryl is a single binary with a small command surface. All commands are run via `cargo run -- <args>` during development, or directly with the compiled binary.

```text
merryl <COMMAND>
merryl run <WORKFLOW> [OPTIONS]
merryl db <SUBCOMMAND>
```

## Global Options

All environment variables are loaded from `.env` at runtime. Use `set -a; source .env; set +a` to load them before running commands.

## Commands

### `merryl run daily --date <DATE>`

Run the full daily market rotation workflow: fetch data, score the market, generate reports and exports.

| Option | Default | Description |
|---|---|---|
| `--date` | `latest` | Trading date to process. Use `latest` for the most recent trading day, or `YYYY-MM-DD` for a specific date. |

**Outputs:**

```text
data/market.db                          SQLite database (system of record)
reports/YYYY-MM-DD_market_report.md     Daily market rotation report
exports/YYYY-MM-DD_sector_scores.csv    Sector scores export
exports/YYYY-MM-DD_stock_watchlist.csv  Stock watchlist export
```

### `merryl run backtest --from <DATE> --to <DATE>`

Run backtest validation over a date range. Reads scored data from SQLite only -- does not fetch new prices or change formulas.

| Option | Required | Description |
|---|---|---|
| `--from` | Yes | Start date (`YYYY-MM-DD`) |
| `--to` | Yes | End date (`YYYY-MM-DD`) |

**Outputs:**

```text
reports/backtests/FROM_TO_backtest_report.md           Backtest report
exports/backtests/FROM_TO_backtest_summary.csv         Backtest summary
reports/validations/FROM_TO_macro_regime_validation.md Macro regime validation report
exports/validations/FROM_TO_macro_regime_validation.csv
reports/validations/FROM_TO_event_context_validation.md Event context validation report
exports/validations/FROM_TO_event_context_validation.csv
reports/validations/FROM_TO_actionability_validation.md Actionability validation report
exports/validations/FROM_TO_actionability_validation.csv
```

### `merryl run intraday --date <DATE>`

Run Phase 6 intraday execution readiness analysis. Evaluates watchlist candidates for intraday setup and trigger conditions.

| Option | Default | Description |
|---|---|---|
| `--date` | `latest` | Score date to evaluate. Use `latest` or `YYYY-MM-DD`. |

**Outputs:**

```text
reports/intraday/YYYY-MM-DD_intraday_execution_readiness.md   Intraday readiness report
exports/intraday/YYYY-MM-DD_intraday_execution_readiness.csv  Intraday readiness export
```

### `merryl status`

Print a summary of the current database state: date range, observation counts, and coverage statistics.

### `merryl doctor`

Run diagnostic checks on the local setup and database. Reports missing data, stale observations, configuration issues, and warnings.

### `merryl dashboard [--port <PORT>] [--export-static <DIR>]`

Start the local read-only dashboard server or export static dashboard data.

| Option | Default | Description |
|---|---|---|
| `--port` | `8787` | HTTP port for the dashboard server |
| `--export-static` | (none) | Export dashboard JSON snapshots to a directory (for static hosting) |

The dashboard is a reader only. It does not fetch new market data or recalculate scores.

**Static export example:**

```bash
npm --prefix dashboard run build
cargo run -- dashboard --export-static dashboard/dist/static-data
```

### `merryl db migrate`

Create or migrate the local SQLite database schema. Creates `data/market.db` if it does not exist.

## Workflow Configuration

Workflow-specific settings are in `config/workflows/`:

| File | Purpose |
|---|---|
| `config/workflows/daily.toml` | Daily workflow configuration |
| `config/workflows/backtest.toml` | Backtest workflow configuration |
| `config/workflows/weekly.toml` | Weekly workflow configuration |

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | Runtime error (check stderr for details) |
