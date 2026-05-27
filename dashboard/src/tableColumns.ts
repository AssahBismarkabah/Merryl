import type { ColumnDef } from "@tanstack/react-table";
import { number, percent, signed } from "./format";
import type { Industry, Sector, Stock, WatchlistRow } from "./types";

export const sectorColumns: ColumnDef<Sector>[] = [
  { header: "Rank", accessorKey: "rank" },
  { header: "Sector", accessorKey: "sector" },
  { header: "ETF", accessorKey: "sector_etf" },
  { header: "Score", cell: ({ row }) => number(row.original.score) },
  { header: "5D", cell: ({ row }) => percent(row.original.return_5d) },
  { header: "20D", cell: ({ row }) => percent(row.original.return_20d) },
  { header: "60D", cell: ({ row }) => percent(row.original.return_60d) },
  { header: "Vs SPY", cell: ({ row }) => percent(row.original.relative_return_vs_spy) },
  { header: "Rel Vol", cell: ({ row }) => number(row.original.relative_volume) },
  { header: "Rank Chg", cell: ({ row }) => signed(row.original.rank_change) }
];

export const industryColumns: ColumnDef<Industry>[] = [
  { header: "Rank", accessorKey: "rank" },
  { header: "Industry", accessorKey: "industry" },
  { header: "Sector", accessorKey: "sector" },
  { header: "Score", cell: ({ row }) => number(row.original.score) },
  { header: "20D", cell: ({ row }) => percent(row.original.return_20d) },
  { header: "Vs Sector", cell: ({ row }) => percent(row.original.relative_return_vs_sector) },
  { header: "Rel Vol", cell: ({ row }) => number(row.original.relative_volume) },
  { header: "Members", cell: ({ row }) => String(row.original.member_count) }
];

export const stockColumns: ColumnDef<Stock>[] = [
  { header: "Rank", accessorKey: "rank" },
  { header: "Symbol", accessorKey: "symbol" },
  { header: "Name", accessorKey: "name" },
  { header: "Sector", accessorKey: "sector" },
  { header: "Industry", accessorKey: "industry" },
  { header: "Score", cell: ({ row }) => number(row.original.score) },
  { header: "Vs Sector", cell: ({ row }) => percent(row.original.relative_return_vs_sector) },
  { header: "Vs SPY", cell: ({ row }) => percent(row.original.relative_return_vs_spy) },
  { header: "Rel Vol", cell: ({ row }) => number(row.original.relative_volume) },
  { header: "Catalyst", accessorKey: "catalyst_status" }
];

export const watchlistColumns: ColumnDef<WatchlistRow>[] = [
  { header: "Rank", accessorKey: "rank" },
  { header: "Symbol", accessorKey: "symbol" },
  { header: "Name", accessorKey: "name" },
  { header: "Sector", accessorKey: "sector" },
  { header: "Industry", accessorKey: "industry" },
  { header: "Score", cell: ({ row }) => number(row.original.score) },
  { header: "Catalyst", accessorKey: "catalyst_status" },
  { header: "Reason", accessorKey: "reason" }
];
