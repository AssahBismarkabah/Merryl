import {
  Activity,
  BarChart3,
  Database,
  Layers3,
  LineChart,
  ShieldCheck,
  Target
} from "lucide-react";
import type { ReactNode } from "react";
import type { DashboardView } from "../view";

export function DashboardSidebar({
  activeView,
  onViewChange
}: {
  activeView: DashboardView;
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
        <strong>Merryl</strong>
      </div>

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
    </aside>
  );
}
