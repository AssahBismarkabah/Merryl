import {
  Activity,
  BarChart3,
  Database,
  Layers3,
  LineChart,
  RefreshCcw,
  ShieldCheck,
  Target
} from "lucide-react";
import type { ReactNode } from "react";
import { number, percent } from "../format";
import type { DashboardSnapshot } from "../types";
import type { DashboardView } from "../view";

export function DashboardSidebar({
  data,
  activeView,
  loading,
  onRefresh,
  onViewChange
}: {
  data: DashboardSnapshot;
  activeView: DashboardView;
  loading: boolean;
  onRefresh: () => void;
  onViewChange: (view: DashboardView) => void;
}) {
  const navItems: Array<{
    view: DashboardView;
    label: string;
    icon: ReactNode;
  }> = [
    { view: "overview", label: "Overview", icon: <Activity size={16} /> },
    { view: "regime", label: "Regime", icon: <ShieldCheck size={16} /> },
    { view: "sectors", label: "Sectors", icon: <BarChart3 size={16} /> },
    { view: "industries", label: "Industries", icon: <Layers3 size={16} /> },
    { view: "leadership", label: "Leadership", icon: <LineChart size={16} /> },
    { view: "watchlist", label: "Watchlist", icon: <Target size={16} /> },
    { view: "validation", label: "Validation", icon: <Database size={16} /> }
  ];

  return (
    <aside className="sidebar">
      <div className="brandBlock">
        <div className="brandMark">M</div>
        <div>
          <strong>Merryl</strong>
        </div>
      </div>

      <button className="sideRefresh" type="button" onClick={onRefresh}>
        <RefreshCcw size={16} />
        <span>{loading ? "Loading" : "Refresh data"}</span>
      </button>

      <nav className="sideNav" aria-label="Dashboard views">
        {navItems.map((item) => (
          <button
            className={item.view === activeView ? "active" : undefined}
            key={item.view}
            type="button"
            onClick={() => onViewChange(item.view)}
          >
            {item.icon}
            {item.label}
          </button>
        ))}
      </nav>

      <div className="sideStat">
        <span>Market date</span>
        <strong>{data.score_date}</strong>
      </div>

      {data.regime ? (
        <div className="sideStat">
          <span>Regime</span>
          <strong>{`${data.regime.label} ${number(data.regime.score)}`}</strong>
          <small>SPY 20D {percent(data.regime.spy_return_20d)}</small>
        </div>
      ) : null}

    </aside>
  );
}
