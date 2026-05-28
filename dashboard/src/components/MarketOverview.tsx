import { SectorScoreChart } from "./SectorScoreChart";
import { number, percent, toneClass } from "../format";
import type { DashboardSnapshot } from "../types";

export function MarketOverview({ data }: { data: DashboardSnapshot }) {
  const topSector = data.sectors[0];
  const topIndustry = data.industries[0];
  const topStock = data.stocks[0];
  const topWatchlist = data.watchlist[0];
  const focusSymbols = data.watchlist
    .slice(0, 5)
    .map((row) => row.symbol)
    .join(", ");

  return (
    <section className="marketOverview">
      <div className="overviewHeader">
        <h2>Market Map</h2>
      </div>

      <SectorScoreChart sectors={data.sectors} />

      <div className="tableWrap compactTable overviewTable">
        <table>
          <thead>
            <tr>
              <th>Layer</th>
              <th>Lead</th>
              <th>Score</th>
              <th>Signal</th>
              <th>Context</th>
            </tr>
          </thead>
          <tbody>
            {data.regime ? (
              <tr>
                <td>
                  <DataTag tone="accent">Regime</DataTag>
                </td>
                <td className="overviewLeadCell">
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

            {topSector ? (
              <tr>
                <td>
                  <DataTag tone="accent">Sector</DataTag>
                </td>
                <td className="overviewLeadCell">
                  <strong className="symbolCell">{topSector.sector}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(topSector.score)}</strong>
                </td>
                <td className={toneClass(topSector.return_20d)}>{percent(topSector.return_20d)} 20D</td>
                <td>
                  {topSector.sector_etf} / {percent(topSector.relative_return_vs_spy)} vs SPY
                </td>
              </tr>
            ) : null}

            {topIndustry ? (
              <tr>
                <td>
                  <DataTag tone="muted">Industry</DataTag>
                </td>
                <td className="overviewLeadCell">
                  <strong className="symbolCell">{topIndustry.industry}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(topIndustry.score)}</strong>
                </td>
                <td className={toneClass(topIndustry.return_20d)}>
                  {percent(topIndustry.return_20d)} 20D
                </td>
                <td>{topIndustry.sector}</td>
              </tr>
            ) : null}

            {topStock ? (
              <tr>
                <td>
                  <DataTag tone="muted">Stock</DataTag>
                </td>
                <td className="overviewLeadCell">
                  <strong className="symbolCell">{topStock.symbol}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(topStock.score)}</strong>
                </td>
                <td className={toneClass(topStock.relative_return_vs_sector)}>
                  {percent(topStock.relative_return_vs_sector)} vs sector
                </td>
                <td>{topStock.sector} / {topStock.industry}</td>
              </tr>
            ) : null}

            {topWatchlist ? (
              <tr>
                <td>
                  <DataTag tone="muted">Watchlist</DataTag>
                </td>
                <td className="overviewLeadCell">
                  <strong className="symbolCell">{focusSymbols}</strong>
                </td>
                <td>
                  <strong className="metricCell">{number(topWatchlist.score)}</strong>
                </td>
                <td>{topWatchlist.catalyst_status.replaceAll("_", " ")}</td>
                <td>Chart review queue</td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function DataTag({ children, tone }: { children: string; tone: "accent" | "muted" }) {
  return <span className={`dataTag ${tone}`}>{children}</span>;
}
