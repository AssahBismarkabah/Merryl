import { RefreshCcw } from "lucide-react";
import { type ReactNode, useMemo, useState } from "react";
import { DataTable } from "./DataTable";
import { DashboardSidebar } from "./DashboardSidebar";
import { MarketOverview } from "./MarketOverview";
import { ScreenerPage } from "./ScreenerPage";
import { number, percent } from "../format";
import {
  actionabilityQueueColumns,
  industryColumns,
  intradaySetupColumns,
  intradayTriggerColumns,
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
  const scoredDates = dates ?? [];

  return (
    <label className="dateControl">
      <span>Market date</span>
      <select
        aria-label="Market date"
        disabled={disabled || scoredDates.length === 0}
        value={selectedDate}
        onChange={(event) => onChange(event.target.value)}
      >
        {scoredDates.map((date) => (
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
        <div className="viewSurface validationStack">
          <section className="detailSection">
            <ViewHeader title="Actionability Buckets" />
            <SimpleTable
              columns={["Bucket", "Count", "Lead", "Context"]}
              rows={actionabilityRows(data)}
            />
          </section>

          <section className="detailSection">
            <ViewHeader title="Actionability Review Queue" />
            <DataTable data={actionabilityQueueStocks(data)} columns={actionabilityQueueColumns} />
          </section>

          <section className="detailSection">
            <ViewHeader title="Watchlist" />
            <DataTable data={data.watchlist} columns={watchlistColumns} />
          </section>
        </div>
      );
    case "execution":
      return (
        <div className="viewSurface validationStack">
          <section className="detailSection">
            <ViewHeader title="Execution Readiness" note="Signal-only intraday layer" />
            <SimpleTable
              columns={["Stage", "Count", "Lead", "Context"]}
              rows={executionRows(data)}
            />
          </section>

          <section className="detailSection">
            <ViewHeader title="Readiness Queue" />
            {(data.intraday_setups ?? []).length > 0 ? (
              <DataTable data={data.intraday_setups ?? []} columns={intradaySetupColumns} />
            ) : (
              <p className="empty">No intraday readiness rows are stored for this date.</p>
            )}
          </section>

          <section className="detailSection">
            <ViewHeader title="Trigger Events" />
            {(data.intraday_triggers ?? []).length > 0 ? (
              <DataTable data={data.intraday_triggers ?? []} columns={intradayTriggerColumns} />
            ) : (
              <p className="empty">No Stage 3 trigger events are stored for this date.</p>
            )}
          </section>
        </div>
      );
    case "screener":
      return <ScreenerPage />;
    case "validation":
      return (
        <div className="viewSurface validationStack">
          <section className="detailSection">
            <ViewHeader title="Validation Summary" />
            <SimpleTable columns={["Layer", "Lead", "Value", "Signal", "Context"]} rows={validationRows(data)} />
          </section>

          <section className="detailSection">
            <ViewHeader title="Data Health" />
            <SimpleTable columns={["Layer", "Lead", "Value", "Signal", "Context"]} rows={dataHealthRows(data)} />
          </section>

          <section className="detailSection">
            <ViewHeader title="Coverage Limits" />
            <SimpleTable
              columns={["Layer", "Lead", "Value", "Signal", "Context"]}
              rows={(data.limitations ?? []).map((item) => limitRow(item))}
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
  const macroCoverage = dashboardHealth(data).required_macro_coverage ?? [];
  return macroCoverage.length > 0 && missingMacroSeries(data).length === 0;
}

function missingMacroSeries(data: DashboardSnapshot) {
  return (dashboardHealth(data).required_macro_coverage ?? []).filter((coverage) => coverage.observation_count === 0);
}

function dataHealthRows(data: DashboardSnapshot): ReactNode[][] {
  const health = dashboardHealth(data);
  const missingSymbols = health.missing_symbols ?? [];
  const missingMaps = health.missing_sector_maps ?? [];
  const macroCoverage = health.required_macro_coverage ?? [];
  const missingMacro = missingMacroSeries(data);
  const latestCoverage = health.latest_score_coverage ?? {
    sector_rows: 0,
    industry_rows: 0,
    stock_rows: 0
  };

  return [
    [
      <DataTag tone="accent">Market</DataTag>,
      "Date",
      data.score_date,
      <DataTag tone="accent">Current</DataTag>,
      "Selected score date"
    ],
    [
      <DataTag tone="accent">Scores</DataTag>,
      "Dates",
      String(health.score_dates ?? 0),
      <DataTag tone="accent">Stored</DataTag>,
      "Historical scoring coverage"
    ],
    [
      <DataTag tone="muted">Universe</DataTag>,
      "Symbols",
      String(health.required_symbol_count ?? 0),
      <DataTag tone="accent">Tracked</DataTag>,
      "Configured market universe"
    ],
    [
      <DataTag tone="muted">Coverage</DataTag>,
      "Missing symbols",
      String(missingSymbols.length),
      <DataTag tone={missingSymbols.length === 0 ? "accent" : "muted"}>
        {missingSymbols.length === 0 ? "OK" : "Review"}
      </DataTag>,
      "Required symbol ingestion"
    ],
    [
      <DataTag tone="muted">Coverage</DataTag>,
      "Missing maps",
      String(missingMaps.length),
      <DataTag tone={missingMaps.length === 0 ? "accent" : "muted"}>
        {missingMaps.length === 0 ? "OK" : "Review"}
      </DataTag>,
      "Sector and industry mapping"
    ],
    [
      <DataTag tone="muted">Macro</DataTag>,
      "FRED series",
      String(macroCoverage.length),
      <DataTag tone={hasMacroCoverage(data) ? "accent" : "muted"}>
        {hasMacroCoverage(data) ? "Stored" : "Review"}
      </DataTag>,
      "Macro context coverage"
    ],
    [
      <DataTag tone="muted">Macro</DataTag>,
      "Missing series",
      String(missingMacro.length),
      <DataTag tone={missingMacro.length === 0 ? "accent" : "muted"}>
        {missingMacro.length === 0 ? "OK" : "Review"}
      </DataTag>,
      "Required FRED observations"
    ],
    [
      <DataTag tone="accent">Latest</DataTag>,
      "Rows",
      `${latestCoverage.sector_rows} sectors / ${latestCoverage.industry_rows} industries / ${latestCoverage.stock_rows} stocks`,
      <DataTag tone="accent">Stored</DataTag>,
      "Dashboard rows from SQLite"
    ],
    [
      <DataTag tone="muted">Storage</DataTag>,
      "System",
      "SQLite",
      <DataTag tone="accent">Connected</DataTag>,
      "Read-only dashboard source"
    ]
  ];
}

function dashboardHealth(data: DashboardSnapshot): Partial<DashboardSnapshot["data_health"]> {
  return data.data_health ?? {};
}

function actionabilityRows(data: DashboardSnapshot): ReactNode[][] {
  const byBucket = new Map<string, typeof data.stocks>();
  for (const stock of data.stocks) {
    const bucket = stock.primary_actionability || "unclassified_leader";
    byBucket.set(bucket, [...(byBucket.get(bucket) ?? []), stock]);
  }

  return actionabilityBucketOrder
    .filter((bucket) => (byBucket.get(bucket)?.length ?? 0) > 0)
    .map((bucket) => {
      const stocks = byBucket.get(bucket) ?? [];
      const lead = stocks[0];
      return [
        <DataTag tone={bucket === "extended_leader" ? "muted" : "accent"}>
          {bucketLabel(bucket)}
        </DataTag>,
        stocks.length,
        lead ? `${lead.symbol} ${number(lead.score)}` : "None",
        actionabilityContext(bucket)
      ];
    });
}

function actionabilityQueueStocks(data: DashboardSnapshot) {
  return [...data.stocks].sort((left, right) => {
    const leftBucket = left.primary_actionability || "unclassified_leader";
    const rightBucket = right.primary_actionability || "unclassified_leader";
    return actionabilityBucketOrderIndex(leftBucket) - actionabilityBucketOrderIndex(rightBucket) || left.rank - right.rank;
  });
}

function executionRows(data: DashboardSnapshot): ReactNode[][] {
  const setups = data.intraday_setups ?? [];
  const stage2 = setups.filter((setup) => setup.stage2_passed);
  const stage3 = setups.filter((setup) => setup.stage3_passed);
  const lead = setups[0];

  return [
    [
      <DataTag tone="accent">Stage 1</DataTag>,
      String(setups.length),
      lead ? `${lead.symbol} ${number(lead.mansfield_rs_spy)}` : "None",
      "High ADR, rVOL, and relative strength"
    ],
    [
      <DataTag tone={stage2.length > 0 ? "accent" : "muted"}>Stage 2</DataTag>,
      String(stage2.length),
      stage2[0] ? `${stage2[0].symbol} ${stage2[0].confluence_count}` : "None",
      "Structural confluence near value/EMA levels"
    ],
    [
      <DataTag tone={stage3.length > 0 ? "accent" : "muted"}>Stage 3</DataTag>,
      String(data.intraday_triggers?.length ?? 0),
      stage3[0] ? `${stage3[0].symbol} ${stage3[0].trigger_count}` : "None",
      "Execution-readiness trigger detected"
    ]
  ];
}

const actionabilityBucketOrder = [
  "early_rotation_candidate",
  "base_compression_candidate",
  "pullback_leader",
  "actionable_leader",
  "extended_leader",
  "event_watch_unconfirmed",
  "unclassified_leader"
];

function actionabilityBucketOrderIndex(bucket: string) {
  const idx = actionabilityBucketOrder.indexOf(bucket);
  return idx === -1 ? actionabilityBucketOrder.length : idx;
}

function bucketLabel(value: string) {
  switch (value) {
    case "early_rotation_candidate":
      return "Early";
    case "base_compression_candidate":
      return "Base";
    case "pullback_leader":
      return "Pullback";
    case "actionable_leader":
      return "Actionable";
    case "extended_leader":
      return "Extended";
    case "event_watch_unconfirmed":
      return "Event watch";
    case "unclassified_leader":
      return "Unclassified";
    default:
      return value.replaceAll("_", " ");
  }
}

function actionabilityContext(value: string) {
  switch (value) {
    case "early_rotation_candidate":
      return "Starting to participate";
    case "base_compression_candidate":
      return "Tight near highs";
    case "pullback_leader":
      return "Leader pulling back";
    case "actionable_leader":
      return "Confirmed, not stretched";
    case "extended_leader":
      return "Already moved";
    case "event_watch_unconfirmed":
      return "Event context only";
    default:
      return "Needs review";
  }
}

function validationRows(data: DashboardSnapshot): ReactNode[][] {
  const latestBacktest = data.latest_backtest;
  const metrics = latestBacktest?.metrics;

  if (!latestBacktest) {
    return [
      [
        <DataTag tone="muted">Backtest</DataTag>,
        "Stored result",
        "Missing",
        <DataTag tone="muted">Run needed</DataTag>,
        "Use `merryl run backtest` after scores exist"
      ]
    ];
  }

  return [
    [
      <DataTag tone="accent">Backtest</DataTag>,
      "Range",
      `${latestBacktest.from_date} to ${latestBacktest.to_date}`,
      <DataTag tone="accent">Stored</DataTag>,
      "Score behavior validation"
    ],
    [
      <DataTag tone="accent">Backtest</DataTag>,
      "Scope",
      validationScope(metrics),
      <DataTag tone="muted">Validation</DataTag>,
      "Not trade profitability"
    ],
    [
      <DataTag tone="muted">Sector</DataTag>,
      "Observations",
      metricCount(metrics, "sector_observation_count"),
      <DataTag tone="accent">Ready</DataTag>,
      "Sector score forward behavior"
    ],
    [
      <DataTag tone="muted">Stock</DataTag>,
      "Observations",
      metricCount(metrics, "stock_observation_count"),
      <DataTag tone="accent">Ready</DataTag>,
      "Stock score forward behavior"
    ],
    [
      <DataTag tone="muted">Industry</DataTag>,
      "Observations",
      metricCount(metrics, "industry_stock_observation_count"),
      <DataTag tone="accent">Ready</DataTag>,
      "Industry-adjusted stock behavior"
    ]
  ];
}

type BacktestMetrics = NonNullable<DashboardSnapshot["latest_backtest"]>["metrics"];

function validationScope(metrics: BacktestMetrics | undefined) {
  const purpose = metrics?.validation_scope?.purpose;
  return purpose ? purpose.replaceAll("_", " ") : "score behavior validation";
}

function metricCount(
  metrics: BacktestMetrics | undefined,
  key: "sector_observation_count" | "stock_observation_count" | "industry_stock_observation_count"
) {
  return String(metrics?.[key] ?? 0);
}

function limitRow(item: string): ReactNode[] {
  const lower = item.toLowerCase();

  if (lower.includes("read-only")) {
    return [
      <DataTag tone="accent">Dashboard</DataTag>,
      "Read-only",
      "Stored SQLite scores",
      <DataTag tone="muted">Phase 4</DataTag>,
      "No scoring or writes in UI"
    ];
  }
  if (lower.includes("watchlist")) {
    return [
      <DataTag tone="accent">Watchlist</DataTag>,
      "Chart review only",
      "Not a trade signal",
      <DataTag tone="muted">Risk plan</DataTag>,
      "Final output remains a review queue"
    ];
  }
  if (lower.includes("sector ranking")) {
    return [
      <DataTag tone="muted">Sector</DataTag>,
      "Map layer",
      "Attention ranking",
      <DataTag tone="muted">Formula review</DataTag>,
      "Not standalone forward signal"
    ];
  }
  if (lower.includes("market regime")) {
    return [
      <DataTag tone="muted">Regime</DataTag>,
      "ETF score",
      "FRED context separate",
      <DataTag tone="muted">Validate</DataTag>,
      "Macro context is not a score input yet"
    ];
  }
  if (lower.includes("earnings")) {
    return [
      <DataTag tone="muted">Catalyst</DataTag>,
      "Events",
      "Source-backed where available",
      <DataTag tone="muted">Data source</DataTag>,
      "Context, not score input"
    ];
  }
  if (lower.includes("backtests")) {
    return [
      <DataTag tone="accent">Backtest</DataTag>,
      "Score behavior",
      "Validation only",
      <DataTag tone="muted">Research</DataTag>,
      "No P&L claim"
    ];
  }

  return [<DataTag tone="muted">General</DataTag>, "Limit", item, <DataTag tone="muted">Review</DataTag>, "Phase control"];
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
    case "execution":
      return { eyebrow: "Intraday layer", heading: "Execution Readiness" };
    case "validation":
      return { eyebrow: "Controls", heading: "Validation" };
    case "screener":
      return { eyebrow: "Fundamental filters", heading: "Finviz Screener" };
  }
}
