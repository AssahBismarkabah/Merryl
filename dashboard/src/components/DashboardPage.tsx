import { RefreshCcw } from "lucide-react";
import { useMemo, useState } from "react";
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
  loading,
  onRefresh
}: {
  data: DashboardSnapshot;
  loading: boolean;
  onRefresh: () => void;
}) {
  const [activeView, setActiveView] = useState<DashboardView>("overview");
  const title = useMemo(() => viewTitle(activeView), [activeView]);

  return (
    <div className="dashboardShell">
      <DashboardSidebar
        data={data}
        activeView={activeView}
        loading={loading}
        onRefresh={onRefresh}
        onViewChange={setActiveView}
      />

      <section className="workspace">
        <header className="workspaceHeader">
          <div>
            <p className="eyebrow">{title.eyebrow}</p>
            <h1>{title.heading}</h1>
          </div>
          <button className="iconButton" type="button" onClick={onRefresh}>
            <RefreshCcw size={17} />
            <span>{loading ? "Loading" : "Refresh"}</span>
          </button>
        </header>

        {renderView(activeView, data)}
      </section>
    </div>
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
          <MarketStrip data={data} />
          <section className="detailSection">
            <ViewHeader title="Market Regime" />
            {data.regime ? (
              <FactList
                facts={[
                  ["Regime", data.regime.label],
                  ["Score", number(data.regime.score)],
                  ["SPY 20D", percent(data.regime.spy_return_20d)],
                  ["SPY 60D", percent(data.regime.spy_return_60d)],
                  ["QQQ vs SPY", percent(data.regime.qqq_relative_return_vs_spy)],
                  ["IWM vs SPY", percent(data.regime.iwm_relative_return_vs_spy)],
                  ["DIA vs SPY", percent(data.regime.dia_relative_return_vs_spy)],
                  ["TLT 20D", percent(data.regime.tlt_return_20d)],
                  ["GLD 20D", percent(data.regime.gld_return_20d)],
                  ["USO 20D", percent(data.regime.uso_return_20d)]
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
          <ViewHeader title="Sector Rotation" />
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
        <div className="viewSurface splitView">
          <section className="detailSection">
            <ViewHeader title="Backtest Review" />
            {data.latest_backtest ? (
              <FactList
                facts={[
                  ["Run", data.latest_backtest.run_name],
                  ["Range", `${data.latest_backtest.from_date} to ${data.latest_backtest.to_date}`],
                  [
                    "Sector observations",
                    String(data.latest_backtest.metrics.sector_observation_count ?? 0)
                  ],
                  [
                    "Stock observations",
                    String(data.latest_backtest.metrics.stock_observation_count ?? 0)
                  ]
                ]}
              />
            ) : (
              <p className="empty">No stored backtest result yet.</p>
            )}
          </section>

          <section className="detailSection">
            <ViewHeader title="Data Health" />
            <FactList
              facts={[
                ["Market date", data.score_date],
                ["Score dates", String(data.data_health.score_dates)],
                ["Required symbols", String(data.data_health.required_symbol_count)],
                ["Missing symbols", String(data.data_health.missing_symbols.length)],
                ["Missing sector maps", String(data.data_health.missing_sector_maps.length)],
                [
                  "Latest score rows",
                  `${data.data_health.latest_score_coverage.sector_rows} sectors / ${data.data_health.latest_score_coverage.stock_rows} stocks`
                ],
                ["Database", data.data_health.database_path]
              ]}
            />
          </section>

          <section className="detailSection fullSpan">
            <ViewHeader title="Known Limits" />
            <ul className="limitsList">
              {data.limitations.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
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

function FactList({ facts }: { facts: Array<[string, string]> }) {
  return (
    <dl className="factList">
      {facts.map(([label, value]) => (
        <div className="factRow" key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
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
