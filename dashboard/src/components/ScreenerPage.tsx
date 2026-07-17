import { useEffect, useState } from "react";
import { fetchScreener } from "../api";
import type { ScreenerResult } from "../types";

const SECTORS = [
  { value: "", label: "All Sectors" },
  { value: "Basic Materials", label: "Basic Materials" },
  { value: "Communication Services", label: "Communication Services" },
  { value: "Consumer Cyclical", label: "Consumer Cyclical" },
  { value: "Consumer Defensive", label: "Consumer Defensive" },
  { value: "Energy", label: "Energy" },
  { value: "Financial", label: "Financial" },
  { value: "Healthcare", label: "Healthcare" },
  { value: "Industrials", label: "Industrials" },
  { value: "Real Estate", label: "Real Estate" },
  { value: "Technology", label: "Technology" },
  { value: "Utilities", label: "Utilities" },
];

export function ScreenerPage() {
  const [sector, setSector] = useState("");
  const [results, setResults] = useState<ScreenerResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let ignore = false;

    async function load() {
      setLoading(true);
      setError(null);
      try {
        const data = await fetchScreener(sector || undefined);
        if (!ignore) {
          setResults(data.results);
        }
      } catch (err) {
        if (!ignore) {
          setError(err instanceof Error ? err.message : "Failed to load screener");
        }
      } finally {
        if (!ignore) {
          setLoading(false);
        }
      }
    }

    load();
    return () => {
      ignore = true;
    };
  }, [sector]);

  return (
    <div className="viewSurface">
      <section className="detailSection">
        <div className="screenerControls">
          <label className="screenerSelect">
            <span>Sector</span>
            <select
              value={sector}
              onChange={(e) => setSector(e.target.value)}
              disabled={loading}
            >
              {SECTORS.map((s) => (
                <option key={s.value} value={s.value}>
                  {s.label}
                </option>
              ))}
            </select>
          </label>
          {loading ? <span className="screenerStatus">Loading...</span> : null}
          {!loading && results.length > 0 ? (
            <span className="screenerStatus">{results.length} results</span>
          ) : null}
        </div>
      </section>

      {error ? <p className="empty">{error}</p> : null}

      {!loading && !error && results.length === 0 ? (
        <p className="empty">No results found for the selected sector.</p>
      ) : null}

      {results.length > 0 ? (
        <div className="tableWrap">
          <table className="screenerTable">
            <thead>
              <tr>
                <th>Ticker</th>
                <th>Company</th>
                <th>Sector</th>
                <th>Industry</th>
                <th className="num">Market Cap</th>
                <th className="num">P/E</th>
                <th className="num">Price</th>
                <th className="num">Change</th>
                <th className="num">Volume</th>
              </tr>
            </thead>
            <tbody>
              {results.map((row) => (
                <tr key={row.ticker}>
                  <td>
                    <a
                      href={`https://finviz.com/stock.ashx?t=${row.ticker}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="tickerLink"
                    >
                      {row.ticker}
                    </a>
                  </td>
                  <td className="companyCell">{row.company}</td>
                  <td>{row.sector}</td>
                  <td>{row.industry}</td>
                  <td className="num">{row.market_cap}</td>
                  <td className="num">{row.pe_ratio}</td>
                  <td className="num">{row.price}</td>
                  <td className={`num ${changeClass(row.change)}`}>{row.change}</td>
                  <td className="num">{row.volume}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}
    </div>
  );
}

function changeClass(change: string): string {
  if (change.startsWith("-")) return "negative";
  if (change.startsWith("+")) return "positive";
  return "";
}
