import { RefreshCcw } from "lucide-react";
import { type ReactNode, useMemo, useState } from "react";
import { DataTable } from "./DataTable";
import { DashboardSidebar } from "./DashboardSidebar";
import { MarketOverview } from "./MarketOverview";
import { MarketStrip } from "./MarketStrip";
import { number, percent } from "../format";
import {
  industryColumns,
  sectorColumns,
  stockColumns,
  watchlistColumns
} from "../tableColumns";
import type { DashboardSnapshot } from "../types";
import type { DashboardView } from "../view";

export function DashboardPage({
  data,
  dates,
  loading,
  selectedDate,
  onDateChange,
  onRefresh
}: {
  data: DashboardSnapshot;
  dates: string[];
  loading: boolean;
  selectedDate: string;
  onDateChange: (date: string) => void;
  onRefresh: () => void;
}) {
  const [activeView, setActiveView] = useState<DashboardView>("overview");
  const title = useMemo(() => viewTitle(activeView), [activeView]);

  return (
    <div className="dashboardShell">
      <DashboardSidebar
        activeView={activeView}
        onViewChange={setActiveView}
      />

      <section className="workspace">
        <header className="workspaceHeader">
          <div>
            <p className="eyebrow">{title.eyebrow}</p>
            <h1>{title.heading}</h1>
          </div>
          <div className="headerActions">
            <HeaderChips data={data} />
            <DateSelector
              dates={dates}
              disabled={loading}
              selectedDate={selectedDate}
              onChange={onDateChange}
            />
            <button className="iconButton" type="button" onClick={onRefresh}>
              <RefreshCcw size={17} />
              <span>{loading ? "Loading" : "Refresh"}</span>
            </button>
          </div>
        </header>

        {renderView(activeView, data)}
      </section>
    </div>
  );
}

function HeaderChips({ data }: { data: DashboardSnapshot }) {
  const topSector = data.sectors[0];
  const topStock = data.stocks[0];
  const chips = [
    ["Regime", data.regime ? `${data.regime.label} ${number(data.regime.score)}` : "Missing"],
    ["Sector", topSector ? `${topSector.sector} ${number(topSector.score)}` : "Missing"],
    ["Stock", topStock ? `${topStock.symbol} ${number(topStock.score)}` : "Missing"]
  ];

  return (
    <div className="headerChips" aria-label="Market summary">
      {chips.map(([label, value]) => (
        <div className="headerChip" key={label}>
          <span>{label}</span>
          <strong>{value}</strong>
        </div>
      ))}
    </div>
  );
}

function DateSelector({
  dates,
  disabled,
  selectedDate,
  onChange
}: {
  dates: string[];
  disabled: boolean;
  selectedDate: string;
  onChange: (date: string) => void;
}) {
  return (
    <label className="dateControl">
      <span>Market date</span>
      <select
        aria-label="Market date"
        disabled={disabled || dates.length === 0}
        value={selectedDate}
        onChange={(event) => onChange(event.target.value)}
      >
        {dates.map((date) => (
          <option key={date} value={date}>
            {date}
          </option>
        ))}
      </select>
    </label>
  );
}

function renderView(view: DashboardView, data: DashboardSnapshot) {
  switch (view) {
    case "overview":
      return (
        <div className="viewSurface">
          <MarketStrip data={data} />
          <MarketOverview data={data} />
        </div>
      );
    case "regime":
      return (
        <div className="viewSurface">
          <section className="detailSection">
            <ViewHeader title="Market Regime" />
            {data.regime ? (
              <SimpleTable
                columns={["Area", "Metric", "Value", "Signal"]}
                rows={[
                  [
                    <DataTag tone="accent">Context</DataTag>,
                    "Regime",
                    <strong className="metricCell">{data.regime.label}</strong>,
                    <DataTag tone="accent">Current</DataTag>
                  ],
                  [
                    <DataTag tone="accent">Context</DataTag>,
                    "Score",
                    number(data.regime.score),
                    <ScoreTag score={data.regime.score} />
                  ],
                  [
                    <DataTag tone="muted">Benchmark</DataTag>,
                    "SPY 20D",
                    <ToneValue value={data.regime.spy_return_20d} />,
                    <MoveTag value={data.regime.spy_return_20d} />
                  ],
                  [
                    <DataTag tone="muted">Benchmark</DataTag>,
                    "SPY 60D",
                    <ToneValue value={data.regime.spy_return_60d} />,
                    <MoveTag value={data.regime.spy_return_60d} />
                  ],
                  [
                    <DataTag tone="muted">Relative</DataTag>,
                    "QQQ vs SPY",
                    <ToneValue value={data.regime.qqq_relative_return_vs_spy} />,
                    <MoveTag value={data.regime.qqq_relative_return_vs_spy} />
                  ],
                  [
                    <DataTag tone="muted">Relative</DataTag>,
                    "IWM vs SPY",
                    <ToneValue value={data.regime.iwm_relative_return_vs_spy} />,
                    <MoveTag value={data.regime.iwm_relative_return_vs_spy} />
                  ],
                  [
                    <DataTag tone="muted">Relative</DataTag>,
                    "DIA vs SPY",
                    <ToneValue value={data.regime.dia_relative_return_vs_spy} />,
                    <MoveTag value={data.regime.dia_relative_return_vs_spy} />
                  ],
                  [
                    <DataTag tone="muted">Intermarket</DataTag>,
                    "TLT 20D",
                    <ToneValue value={data.regime.tlt_return_20d} />,
                    <MoveTag value={data.regime.tlt_return_20d} />
                  ],
                  [
                    <DataTag tone="muted">Intermarket</DataTag>,
                    "GLD 20D",
                    <ToneValue value={data.regime.gld_return_20d} />,
                    <MoveTag value={data.regime.gld_return_20d} />
                  ],
                  [
                    <DataTag tone="muted">Intermarket</DataTag>,
                    "USO 20D",
                    <ToneValue value={data.regime.uso_return_20d} />,
                    <MoveTag value={data.regime.uso_return_20d} />
                  ]
                ]}
              />
            ) : (
              <p className="empty">No regime row is stored for this date.</p>
            )}
          </section>
        </div>
      );
    case "sectors":
      return (
        <div className="viewSurface">
          <ViewHeader title="Sector Rotation" note="Map-only rotation layer" />
          <DataTable data={data.sectors} columns={sectorColumns} />
        </div>
      );
    case "industries":
      return (
        <div className="viewSurface">
          <ViewHeader title="Industry Strength" />
          <DataTable data={data.industries} columns={industryColumns} />
        </div>
      );
    case "leadership":
      return (
        <div className="viewSurface">
          <ViewHeader title="Stock Leadership" />
          <DataTable data={data.stocks} columns={stockColumns} />
        </div>
      );
    case "watchlist":
      return (
        <div className="viewSurface">
          <ViewHeader title="Watchlist" />
          <DataTable data={data.watchlist} columns={watchlistColumns} />
        </div>
      );
    case "validation":
      return (
        <div className="viewSurface validationStack">
          <section className="detailSection">
            <ViewHeader title="Backtest Review" />
            {data.latest_backtest ? (
              <SimpleTable
                columns={["Area", "Metric", "Value", "State"]}
                rows={[
                  [
                    <DataTag tone="accent">Backtest</DataTag>,
                    "Range",
                    `${data.latest_backtest.from_date} to ${data.latest_backtest.to_date}`,
                    <DataTag tone="accent">Stored</DataTag>
                  ],
                  [
                    <DataTag tone="accent">Backtest</DataTag>,
                    "Scope",
                    "Score behavior",
                    <DataTag tone="muted">Validation</DataTag>
                  ],
                  [
                    <DataTag tone="muted">Coverage</DataTag>,
                    "Sectors",
                    String(data.latest_backtest.metrics.sector_observation_count ?? 0),
                    <DataTag tone="accent">Ready</DataTag>
                  ],
                  [
                    <DataTag tone="muted">Coverage</DataTag>,
                    "Stocks",
                    String(data.latest_backtest.metrics.stock_observation_count ?? 0),
                    <DataTag tone="accent">Ready</DataTag>
                  ],
                  [
                    <DataTag tone="muted">Coverage</DataTag>,
                    "Industries",
                    String(data.latest_backtest.metrics.industry_stock_observation_count ?? 0),
                    <DataTag tone="accent">Ready</DataTag>
                  ]
                ]}
              />
            ) : (
              <p className="empty">No stored backtest result yet.</p>
            )}
          </section>

          <section className="detailSection">
            <ViewHeader title="Data Health" />
            <SimpleTable
              columns={["Area", "Metric", "Value", "State"]}
              rows={[
                [<DataTag tone="accent">Market</DataTag>, "Date", data.score_date, <DataTag tone="accent">Current</DataTag>],
                [
                  <DataTag tone="accent">Scores</DataTag>,
                  "Dates",
                  String(data.data_health.score_dates),
                  <DataTag tone="accent">Stored</DataTag>
                ],
                [
                  <DataTag tone="muted">Universe</DataTag>,
                  "Symbols",
                  String(data.data_health.required_symbol_count),
                  <DataTag tone="accent">Tracked</DataTag>
                ],
                [
                  <DataTag tone="muted">Coverage</DataTag>,
                  "Missing symbols",
                  String(data.data_health.missing_symbols.length),
                  <DataTag tone={data.data_health.missing_symbols.length === 0 ? "accent" : "muted"}>
                    {data.data_health.missing_symbols.length === 0 ? "OK" : "Review"}
                  </DataTag>
                ],
                [
                  <DataTag tone="muted">Coverage</DataTag>,
                  "Missing maps",
                  String(data.data_health.missing_sector_maps.length),
                  <DataTag tone={data.data_health.missing_sector_maps.length === 0 ? "accent" : "muted"}>
                    {data.data_health.missing_sector_maps.length === 0 ? "OK" : "Review"}
                  </DataTag>
                ],
                [
                  <DataTag tone="muted">Macro</DataTag>,
                  "FRED series",
                  String(data.data_health.required_macro_coverage.length),
                  <DataTag tone={hasMacroCoverage(data) ? "accent" : "muted"}>
                    {hasMacroCoverage(data) ? "Stored" : "Review"}
                  </DataTag>
                ],
                [
                  <DataTag tone="muted">Macro</DataTag>,
                  "Missing series",
                  String(missingMacroSeries(data).length),
                  <DataTag tone={missingMacroSeries(data).length === 0 ? "accent" : "muted"}>
                    {missingMacroSeries(data).length === 0 ? "OK" : "Review"}
                  </DataTag>
                ],
                [
                  <DataTag tone="accent">Latest</DataTag>,
                  "Rows",
                  `${data.data_health.latest_score_coverage.sector_rows} sectors / ${data.data_health.latest_score_coverage.industry_rows} industries / ${data.data_health.latest_score_coverage.stock_rows} stocks`,
                  <DataTag tone="accent">Stored</DataTag>
                ],
                [<DataTag tone="muted">Storage</DataTag>, "System", "SQLite", <DataTag tone="accent">Connected</DataTag>]
              ]}
            />
          </section>

          <section className="detailSection">
            <ViewHeader title="Coverage Limits" />
            <SimpleTable
              columns={["Area", "Current State", "Next"]}
              rows={data.limitations.map((item) => limitRow(item))}
            />
          </section>
        </div>
      );
  }
}

function ViewHeader({ title, note }: { title: string; note?: string }) {
  return (
    <div className="sectionTitle">
      <h2>{title}</h2>
      {note ? <p>{note}</p> : null}
    </div>
  );
}

function SimpleTable({ columns, rows }: { columns: string[]; rows: ReactNode[][] }) {
  return (
    <div className="tableWrap compactTable">
      <table>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column}>{column}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, idx) => (
            <tr key={idx}>
              {row.map((cell, cellIdx) => (
                <td key={cellIdx}>{cell}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function DataTag({ children, tone }: { children: ReactNode; tone: "accent" | "muted" }) {
  return <span className={`dataTag ${tone}`}>{children}</span>;
}

function ToneValue({ value }: { value: number }) {
  const className = value > 0 ? "positive" : value < 0 ? "negative" : "neutral";
  return <span className={className}>{percent(value)}</span>;
}

function ScoreTag({ score }: { score: number }) {
  if (score > 50) {
    return <DataTag tone="accent">Above 50</DataTag>;
  }
  if (score < 50) {
    return <DataTag tone="muted">Below 50</DataTag>;
  }
  return <DataTag tone="muted">Neutral</DataTag>;
}

function MoveTag({ value }: { value: number }) {
  if (value > 0) {
    return <DataTag tone="accent">Up</DataTag>;
  }
  if (value < 0) {
    return <DataTag tone="muted">Down</DataTag>;
  }
  return <DataTag tone="muted">Flat</DataTag>;
}

function hasMacroCoverage(data: DashboardSnapshot) {
  return data.data_health.required_macro_coverage.length > 0 && missingMacroSeries(data).length === 0;
}

function missingMacroSeries(data: DashboardSnapshot) {
  return data.data_health.required_macro_coverage.filter((coverage) => coverage.observation_count === 0);
}

function limitRow(item: string): ReactNode[] {
  const lower = item.toLowerCase();

  if (lower.includes("read-only")) {
    return [<DataTag tone="accent">Dashboard</DataTag>, "Read-only", <DataTag tone="muted">Phase 4</DataTag>];
  }
  if (lower.includes("watchlist")) {
    return [<DataTag tone="accent">Watchlist</DataTag>, "Chart review only", <DataTag tone="muted">Risk plan</DataTag>];
  }
  if (lower.includes("sector ranking")) {
    return [<DataTag tone="muted">Sector</DataTag>, "Map layer", <DataTag tone="muted">Formula review</DataTag>];
  }
  if (lower.includes("market regime")) {
    return [<DataTag tone="muted">Regime</DataTag>, "ETF score, FRED context", <DataTag tone="muted">Validate</DataTag>];
  }
  if (lower.includes("earnings")) {
    return [<DataTag tone="muted">Catalyst</DataTag>, "Earnings pending", <DataTag tone="muted">Data source</DataTag>];
  }
  if (lower.includes("backtests")) {
    return [<DataTag tone="accent">Backtest</DataTag>, "Score behavior", <DataTag tone="muted">Validation only</DataTag>];
  }

  return [<DataTag tone="muted">General</DataTag>, item, <DataTag tone="muted">Review</DataTag>];
}

function viewTitle(view: DashboardView) {
  switch (view) {
    case "overview":
      return { eyebrow: "Market overview", heading: "Daily Rotation Map" };
    case "regime":
      return { eyebrow: "Market context", heading: "Regime" };
    case "sectors":
      return { eyebrow: "Rotation layer", heading: "Sectors" };
    case "industries":
      return { eyebrow: "Theme layer", heading: "Industries" };
    case "leadership":
      return { eyebrow: "Stock layer", heading: "Leadership" };
    case "watchlist":
      return { eyebrow: "Chart review", heading: "Watchlist" };
    case "validation":
      return { eyebrow: "Controls", heading: "Validation" };
  }
}
