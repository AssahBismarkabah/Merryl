import type { ColumnDef } from "@tanstack/react-table";
import { createElement } from "react";
import { number, percent, signed, toneClass } from "./format";
import type { Industry, Sector, Stock, WatchlistRow } from "./types";

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
    default:
      return value.replaceAll("_", " ");
  }
}
