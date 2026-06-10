import type { ColumnDef } from "@tanstack/react-table";
import { createElement } from "react";
import { number, percent, signed, toneClass } from "./format";
import type { Industry, IntradaySetup, IntradayTrigger, Sector, Stock, WatchlistRow } from "./types";

export const sectorColumns: ColumnDef<Sector>[] = [
  { header: "Rank", cell: ({ row }) => rankCell(row.original.rank) },
  { header: "Sector", cell: ({ row }) => dataTag(row.original.sector, "accent") },
  { header: "ETF", cell: ({ row }) => dataTag(row.original.sector_etf, "muted") },
  { header: "Score", cell: ({ row }) => metricCell(number(row.original.score)) },
  { header: "5D", cell: ({ row }) => percentCell(row.original.return_5d) },
  { header: "20D", cell: ({ row }) => percentCell(row.original.return_20d) },
  { header: "60D", cell: ({ row }) => percentCell(row.original.return_60d) },
  { header: "Vs SPY", cell: ({ row }) => percentCell(row.original.relative_return_vs_spy) },
  { header: "Rel Vol", cell: ({ row }) => metricCell(number(row.original.relative_volume)) },
  { header: "Rank Chg", cell: ({ row }) => signedCell(row.original.rank_change) }
];

export const industryColumns: ColumnDef<Industry>[] = [
  { header: "Rank", cell: ({ row }) => rankCell(row.original.rank) },
  { header: "Industry", accessorKey: "industry" },
  { header: "Sector", cell: ({ row }) => dataTag(row.original.sector, "accent") },
  { header: "Score", cell: ({ row }) => metricCell(number(row.original.score)) },
  { header: "20D", cell: ({ row }) => percentCell(row.original.return_20d) },
  { header: "Vs Sector", cell: ({ row }) => percentCell(row.original.relative_return_vs_sector) },
  { header: "Rel Vol", cell: ({ row }) => metricCell(number(row.original.relative_volume)) },
  { header: "Members", cell: ({ row }) => metricCell(String(row.original.member_count)) }
];

export const stockColumns: ColumnDef<Stock>[] = [
  { header: "Rank", cell: ({ row }) => rankCell(row.original.rank) },
  { header: "Symbol", cell: ({ row }) => symbolCell(row.original.symbol) },
  { header: "Name", accessorKey: "name" },
  { header: "Sector", cell: ({ row }) => dataTag(row.original.sector, "accent") },
  { header: "Industry", cell: ({ row }) => dataTag(row.original.industry, "muted") },
  { header: "Score", cell: ({ row }) => metricCell(number(row.original.score)) },
  { header: "Vs Sector", cell: ({ row }) => percentCell(row.original.relative_return_vs_sector) },
  { header: "Vs SPY", cell: ({ row }) => percentCell(row.original.relative_return_vs_spy) },
  { header: "Rel Vol", cell: ({ row }) => metricCell(number(row.original.relative_volume)) },
  { header: "Actionability", cell: ({ row }) => actionabilityCell(row.original.primary_actionability) },
  { header: "20D MA", cell: ({ row }) => percentCell(row.original.distance_from_20d_ma_pct) },
  { header: "Catalyst", cell: ({ row }) => statusTag(row.original.catalyst_status) }
];

export const actionabilityQueueColumns: ColumnDef<Stock>[] = [
  { header: "Bucket", cell: ({ row }) => actionabilityCell(row.original.primary_actionability) },
  { header: "Rank", cell: ({ row }) => rankCell(row.original.rank) },
  { header: "Symbol", cell: ({ row }) => symbolCell(row.original.symbol) },
  { header: "Sector", cell: ({ row }) => dataTag(row.original.sector, "accent") },
  { header: "Industry", cell: ({ row }) => dataTag(row.original.industry, "muted") },
  { header: "Score", cell: ({ row }) => metricCell(number(row.original.score)) },
  { header: "5D", cell: ({ row }) => percentCell(row.original.return_5d) },
  { header: "20D", cell: ({ row }) => percentCell(row.original.return_20d) },
  { header: "Vs Sector", cell: ({ row }) => percentCell(row.original.relative_return_vs_sector) },
  { header: "Rel Vol", cell: ({ row }) => metricCell(number(row.original.relative_volume)) },
  { header: "20D MA", cell: ({ row }) => percentCell(row.original.distance_from_20d_ma_pct) },
  { header: "Catalyst", cell: ({ row }) => statusTag(row.original.catalyst_status) }
];

export const watchlistColumns: ColumnDef<WatchlistRow>[] = [
  { header: "Rank", cell: ({ row }) => rankCell(row.original.rank) },
  { header: "Symbol", cell: ({ row }) => symbolCell(row.original.symbol) },
  { header: "Name", accessorKey: "name" },
  { header: "Sector", cell: ({ row }) => dataTag(row.original.sector, "accent") },
  { header: "Industry", cell: ({ row }) => dataTag(row.original.industry, "muted") },
  { header: "Score", cell: ({ row }) => metricCell(number(row.original.score)) },
  { header: "Actionability", cell: ({ row }) => actionabilityCell(row.original.primary_actionability) },
  { header: "20D MA", cell: ({ row }) => percentCell(row.original.distance_from_20d_ma_pct) },
  { header: "Classification", cell: ({ row }) => classificationCell(row.original.classifications) },
  { header: "Catalyst", cell: ({ row }) => statusTag(row.original.catalyst_status) }
];

export const intradaySetupColumns: ColumnDef<IntradaySetup>[] = [
  { header: "Label", cell: ({ row }) => readinessCell(row.original.primary_label) },
  { header: "Symbol", cell: ({ row }) => symbolCell(row.original.symbol) },
  { header: "Name", accessorKey: "name" },
  { header: "Sector", cell: ({ row }) => dataTag(row.original.sector, "accent") },
  { header: "Industry", cell: ({ row }) => dataTag(row.original.industry, "muted") },
  { header: "ADR", cell: ({ row }) => percentCell(row.original.adr_pct) },
  { header: "rVOL", cell: ({ row }) => metricCell(number(row.original.rvol_ratio)) },
  { header: "RS SPY", cell: ({ row }) => metricCell(row.original.mansfield_rs_spy.toFixed(3)) },
  { header: "Price", cell: ({ row }) => metricCell(row.original.latest_price.toFixed(2)) },
  { header: "EMA 10", cell: ({ row }) => metricCell(row.original.ema_10.toFixed(2)) },
  { header: "EMA 20", cell: ({ row }) => metricCell(row.original.ema_20.toFixed(2)) },
  { header: "Confluence", cell: ({ row }) => metricCell(String(row.original.confluence_count)) },
  { header: "Triggers", cell: ({ row }) => metricCell(String(row.original.trigger_count)) }
];

export const intradayTriggerColumns: ColumnDef<IntradayTrigger>[] = [
  { header: "Symbol", cell: ({ row }) => symbolCell(row.original.symbol) },
  { header: "Time", cell: ({ row }) => metricCell(row.original.ts) },
  { header: "Trigger", cell: ({ row }) => dataTag(shortLabel(row.original.trigger_type), "accent") },
  { header: "Frame", cell: ({ row }) => dataTag(row.original.timeframe, "muted") },
  { header: "Price", cell: ({ row }) => metricCell(row.original.trigger_price.toFixed(2)) },
  { header: "Reference", cell: ({ row }) => metricCell(row.original.reference_level.toFixed(2)) },
  { header: "Vol Spike", cell: ({ row }) => metricCell(number(row.original.volume_spike)) },
  { header: "Action", accessorKey: "price_action" }
];

type TagTone = "accent" | "muted";

function rankCell(value: number) {
  return createElement("span", { className: "rankCell" }, String(value));
}

function symbolCell(value: string) {
  return createElement("strong", { className: "symbolCell" }, value);
}

function metricCell(value: string) {
  return createElement("span", { className: "metricCell" }, value);
}

function percentCell(value: number) {
  return createElement("span", { className: toneClass(value) }, percent(value));
}

function signedCell(value: number) {
  return createElement("span", { className: toneClass(value) }, signed(value));
}

function statusTag(value: string) {
  const label = value.replaceAll("_", " ");
  const tone = value === "pending_source" ? "muted" : "accent";
  return dataTag(label, tone);
}

function dataTag(value: string, tone: TagTone) {
  return createElement("span", { className: `dataTag ${tone}` }, value);
}

function classificationCell(values?: string[]) {
  const labels = values && values.length > 0 ? values.map(shortLabel) : ["unclassified"];
  return createElement("span", { className: "classificationCell" }, labels.join(", "));
}

function actionabilityCell(value: string) {
  return dataTag(shortLabel(value || "unclassified_leader"), value === "extended_leader" ? "muted" : "accent");
}

function readinessCell(value: string) {
  return dataTag(
    shortLabel(value || "high_momentum_universe"),
    value === "intraday_execution_ready" ? "accent" : "muted"
  );
}

function shortLabel(value: string) {
  switch (value) {
    case "sector_leader":
      return "sector";
    case "industry_leader":
      return "industry";
    case "relative_strength_leader":
      return "strength";
    case "volume_confirmed":
      return "volume";
    case "new_leader":
      return "new";
    case "event_context":
      return "event";
    case "event_risk":
      return "risk";
    case "macro_conflict_context":
      return "macro";
    case "extended_leader":
      return "extended";
    case "actionable_leader":
      return "actionable";
    case "early_rotation_candidate":
      return "early";
    case "pullback_leader":
      return "pullback";
    case "base_compression_candidate":
      return "base";
    case "event_watch_unconfirmed":
      return "event watch";
    case "unclassified_leader":
      return "unclassified";
    case "high_momentum_universe":
      return "stage 1";
    case "structural_pullback_setup":
      return "stage 2";
    case "intraday_execution_ready":
      return "ready";
    case "orb_breakout":
      return "ORB";
    case "hod_break":
      return "HOD";
    case "volume_dryup_confirmation":
      return "dry-up";
    case "micro_cluster_break":
      return "cluster";
    default:
      return value.replaceAll("_", " ");
  }
}
