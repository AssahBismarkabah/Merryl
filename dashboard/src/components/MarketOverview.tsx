import { SectorScoreChart } from "./SectorScoreChart";
import { number, percent, toneClass } from "../format";
import type { DashboardSnapshot } from "../types";

export function MarketOverview({ data }: { data: DashboardSnapshot }) {
  return (
    <section className="marketOverview">
      <div className="overviewHeader">
        <h2>Market Overview</h2>
      </div>

      <SectorScoreChart sectors={data.sectors} />

      <div className="tableWrap compactTable overviewTable">
        <table>
          <thead>
            <tr>
              <th>Layer</th>
              <th>Rank</th>
              <th>Name</th>
              <th>Score</th>
              <th>Move</th>
              <th>Context</th>
            </tr>
          </thead>
          <tbody>
            {data.regime ? (
              <tr>
                <td>
                  <DataTag tone="accent">Regime</DataTag>
                </td>
                <td className="rankCell">-</td>
                <td>
                  <strong className="symbolCell">{data.regime.label}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(data.regime.score)}</strong>
                </td>
                <td className={toneClass(data.regime.spy_return_20d)}>
                  {percent(data.regime.spy_return_20d)} SPY 20D
                </td>
                <td>{percent(data.regime.qqq_relative_return_vs_spy)} QQQ vs SPY</td>
              </tr>
            ) : null}

            {data.sectors.slice(0, 5).map((sector) => (
              <tr key={`sector-${sector.sector}`}>
                <td>
                  <DataTag tone="accent">Sector</DataTag>
                </td>
                <td className="rankCell">{sector.rank}</td>
                <td>
                  <strong className="symbolCell">{sector.sector}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(sector.score)}</strong>
                </td>
                <td className={toneClass(sector.return_20d)}>{percent(sector.return_20d)} 20D</td>
                <td>{sector.sector_etf} / {percent(sector.relative_return_vs_spy)} vs SPY</td>
              </tr>
            ))}

            {data.industries.slice(0, 5).map((industry) => (
              <tr key={`industry-${industry.industry}`}>
                <td>
                  <DataTag tone="muted">Industry</DataTag>
                </td>
                <td className="rankCell">{industry.rank}</td>
                <td>
                  <strong className="symbolCell">{industry.industry}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(industry.score)}</strong>
                </td>
                <td className={toneClass(industry.return_20d)}>{percent(industry.return_20d)} 20D</td>
                <td>{industry.sector}</td>
              </tr>
            ))}

            {data.watchlist.slice(0, 5).map((row) => (
              <tr key={`watchlist-${row.symbol}`}>
                <td>
                  <DataTag tone="muted">Watchlist</DataTag>
                </td>
                <td className="rankCell">{row.rank}</td>
                <td>
                  <strong className="symbolCell">{row.symbol}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(row.score)}</strong>
                </td>
                <td>{row.catalyst_status.replaceAll("_", " ")}</td>
                <td>{row.sector} / {row.industry}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function DataTag({ children, tone }: { children: string; tone: "accent" | "muted" }) {
  return <span className={`dataTag ${tone}`}>{children}</span>;
}
