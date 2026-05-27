# Data Quality And Reproducibility Spec

Version: 1.0  
Date: 2026-05-27  
Status: Implemented

## Purpose

This document records the PDB-6 pre-dashboard stability check.

The dashboard must not sit on top of unclear or incomplete stored data. Before Phase 4 dashboard work, Merryl needs a direct way to reveal whether the local SQLite database contains enough core market-map data to support the existing workflow:

```text
Market regime -> sector rotation -> industry/theme strength -> stock leadership -> watchlist
```

## Alignment With The Main Spec

The main market rotation spec requires a top-down market map, not a loose stock screener.

This check supports that by validating the stored foundation before any dashboard reads it:

- Required broad, macro-proxy, and sector ETF symbols are present.
- Required sector ETF map entries are present.
- Required ETF daily prices have enough bars for the current longest scoring window.
- Historical score dates are present.
- The latest score date is aligned with the latest benchmark price date.
- The latest score date has regime, sector, industry, stock, and watchlist rows.
- Replacement-style writes are idempotent, so repeated daily/report generation does not duplicate rows.

## Implemented Checks

The `doctor` workflow now checks the database when it exists.

Command:

```text
cargo run -- doctor
```

The data quality checks are:

| Check | Requirement |
|---|---|
| Required market symbols | SPY, QQQ, IWM, DIA, TLT, GLD, USO, and all 11 sector ETFs exist in `symbols`. |
| Sector maps | All configured sector-to-ETF mappings exist in `sector_map`. |
| Required ETF price coverage | Each required ETF has at least 61 daily bars, because 60D returns require 60 prior bars plus the current bar. |
| Required ETF latest date alignment | Each required ETF has prices through the latest SPY price date. |
| Historical score coverage | `sector_scores` contains at least 60 distinct score dates. |
| Latest score date | Latest `sector_scores` date matches latest SPY price date. |
| Latest row coverage | Latest score date has 1 regime row, 11 sector rows, at least 1 industry row, 50 stock-score rows, and 25 watchlist rows. |
| Idempotent writes | Tests verify replacement writes do not duplicate market regime, sector, industry, stock, watchlist, migration, or recent-news rows. |

## Design Decision

No new CLI command was added.

The existing small command surface remains:

```text
merryl run daily --date latest
merryl run backtest --from YYYY-MM-DD --to YYYY-MM-DD
merryl status
merryl doctor
```

`doctor` is the right surface because this is an environment/data health check, not a new user workflow.

## What This Does Not Add

This pass does not add:

- dashboard UI
- dashboard API
- intraday/live data
- new paid data providers
- full macro data
- options flow
- dark-pool flow
- portfolio simulation
- trade profitability claims
- report checksum or reproducible artifact hashing

Those are separate phases or explicitly deferred items.

## Acceptance Criteria

PDB-6 is complete when:

- `doctor` can reveal missing required core data before dashboard use.
- Tests verify missing database content is reported.
- Tests verify a complete fixture passes the core data checks.
- Tests verify replacement writes are idempotent.
- Full Rust verification passes.

## Current Decision

PDB-6 is implemented.

Latest local `doctor` result:

```text
required market symbols present (18/18)
required sector map entries present (11/11)
required ETF price coverage >= 61 bars through 2026-05-27 (18/18)
historical score coverage 228 dates
latest score date matches benchmark price date (2026-05-27)
latest score rows 2026-05-27: regime 1/1, sectors 11/11, industries 127, stocks 50/50, watchlist 25/25
```

The next project step can be Phase 4 planning, provided we keep the dashboard scope narrow:

```text
Market regime
Sector rotation
Industry/theme strength
Stock leadership
Watchlist
Historical score/backtest review
```

No alerts, portfolio simulation, intraday execution, options flow, or advanced data-provider expansion should enter the first dashboard slice.
