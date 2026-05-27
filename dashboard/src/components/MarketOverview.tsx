import { ArrowRight } from "lucide-react";
import { number, percent, toneClass } from "../format";
import type { DashboardSnapshot, Sector } from "../types";

export function MarketOverview({ data }: { data: DashboardSnapshot }) {
  const topSector = data.sectors[0];
  const topIndustry = data.industries[0];
  const topStock = data.stocks[0];

  return (
    <section className="marketOverview" id="regime">
      <div className="overviewHeader">
        <div>
          <h2>Market Overview</h2>
        </div>
      </div>

      <div className="flowLine" aria-label="Top-down market path">
        <FlowItem label="Regime" value={data.regime?.label ?? "Missing"} score={data.regime?.score} />
        <ArrowRight size={16} />
        <FlowItem label="Sector" value={topSector?.sector ?? "Missing"} score={topSector?.score} />
        <ArrowRight size={16} />
        <FlowItem
          label="Industry"
          value={topIndustry?.industry ?? "Missing"}
          score={topIndustry?.score}
        />
        <ArrowRight size={16} />
        <FlowItem label="Stock" value={topStock?.symbol ?? "Missing"} score={topStock?.score} />
      </div>

      <div className="overviewBody">
        <div className="sectorBars">
          {data.sectors.slice(0, 8).map((sector) => (
            <SectorBar key={sector.sector} sector={sector} />
          ))}
        </div>
        <div className="regimeSummary">
          <span>Regime</span>
          <strong>{data.regime ? `${data.regime.label} ${number(data.regime.score)}` : "Missing"}</strong>
        </div>
      </div>
    </section>
  );
}

function FlowItem({ label, value, score }: { label: string; value: string; score?: number }) {
  return (
    <div className="flowItem">
      <span>{label}</span>
      <strong>{value}</strong>
      {score === undefined ? null : <b>{number(score)}</b>}
    </div>
  );
}

function SectorBar({ sector }: { sector: Sector }) {
  return (
    <div className="sectorBar">
      <div className="sectorBarMeta">
        <strong>{sector.sector}</strong>
        <span className={toneClass(sector.return_20d)}>{percent(sector.return_20d)} 20D</span>
      </div>
      <div className="scoreTrack">
        <div className="scoreFill" style={{ width: `${Math.max(0, Math.min(100, sector.score))}%` }} />
      </div>
    </div>
  );
}
