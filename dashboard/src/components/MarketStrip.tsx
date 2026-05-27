import { percent, toneClass } from "../format";
import type { DashboardSnapshot } from "../types";

export function MarketStrip({ data }: { data: DashboardSnapshot }) {
  if (!data.regime) {
    return null;
  }

  const items = [
    ["SPY 20D", data.regime.spy_return_20d],
    ["SPY 60D", data.regime.spy_return_60d],
    ["QQQ vs SPY", data.regime.qqq_relative_return_vs_spy],
    ["IWM vs SPY", data.regime.iwm_relative_return_vs_spy],
    ["DIA vs SPY", data.regime.dia_relative_return_vs_spy],
    ["TLT 20D", data.regime.tlt_return_20d],
    ["GLD 20D", data.regime.gld_return_20d],
    ["USO 20D", data.regime.uso_return_20d]
  ];

  return (
    <section className="tickerStrip" aria-label="Market regime metrics">
      {items.map(([label, value]) => (
        <div className="tickerItem" key={label}>
          <span>{label}</span>
          <strong className={toneClass(value as number)}>{percent(value as number)}</strong>
        </div>
      ))}
    </section>
  );
}
