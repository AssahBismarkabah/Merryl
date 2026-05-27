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
  { header: "Catalyst", cell: ({ row }) => statusTag(row.original.catalyst_status) }
];

export const watchlistColumns: ColumnDef<WatchlistRow>[] = [
  { header: "Rank", cell: ({ row }) => rankCell(row.original.rank) },
  { header: "Symbol", cell: ({ row }) => symbolCell(row.original.symbol) },
  { header: "Name", accessorKey: "name" },
  { header: "Sector", cell: ({ row }) => dataTag(row.original.sector, "accent") },
  { header: "Industry", cell: ({ row }) => dataTag(row.original.industry, "muted") },
  { header: "Score", cell: ({ row }) => metricCell(number(row.original.score)) },
  { header: "Catalyst", cell: ({ row }) => statusTag(row.original.catalyst_status) },
  { header: "Reason", accessorKey: "reason" }
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
