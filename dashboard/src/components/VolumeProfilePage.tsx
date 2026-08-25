import { useEffect, useState } from "react";
import { fetchVolumeProfileClassifications } from "../api";
import type {
  VolumeProfileClassification,
  VolumeProfileClassificationResponse,
} from "../types";

export function VolumeProfilePage() {
  const [data, setData] = useState<VolumeProfileClassificationResponse | null>(null);
  const [showAll, setShowAll] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let ignore = false;
    fetchVolumeProfileClassifications()
      .then((response) => {
        if (!ignore) setData(response);
      })
      .catch((err) => {
        if (!ignore) setError(err instanceof Error ? err.message : "Failed to load classifications");
      })
      .finally(() => {
        if (!ignore) setLoading(false);
      });
    return () => {
      ignore = true;
    };
  }, []);

  if (loading) return <div className="viewSurface"><p className="empty">Classifying daily structures...</p></div>;
  if (error) return <div className="viewSurface"><p className="empty">{error}</p></div>;
  if (!data) return <div className="viewSurface"><p className="empty">No classification data.</p></div>;

  const rows = showAll ? data.results : data.results.filter((row) => row.labels.length > 0);

  return (
    <div className="viewSurface validationStack">
      <section className="detailSection">
        <div className="sectionTitle">
          <h2>Daily Volume-Profile Classification</h2>
          <p>Candidate structures for manual chart review; not trade signals.</p>
        </div>
        <div className="volumeProfileNotice">
          <strong>{data.timeframe} total-volume approximation</strong>
          <span>{data.approximation_note}</span>
        </div>
        <div className="volumeProfileStats">
          <span><strong>{data.evaluated_count}</strong> stocks evaluated</span>
          <span><strong>{data.candidate_count}</strong> candidates</span>
          <span><strong>{data.skipped_count}</strong> insufficient history</span>
          <span>latest bar <strong>{data.date || "unknown"}</strong></span>
          <label className="volumeProfileToggle">
            <input type="checkbox" checked={showAll} onChange={(event) => setShowAll(event.target.checked)} />
            Show all evaluated stocks
          </label>
        </div>
      </section>

      <section className="detailSection">
        {rows.length === 0 ? (
          <p className="empty">No current candidates. This is a classification result, not a forced signal.</p>
        ) : (
          <div className="tableWrap">
            <table className="screenerTable volumeProfileTable">
              <thead>
                <tr>
                  <th>Ticker</th>
                  <th>Classification</th>
                  <th>Direction</th>
                  <th>Anchor</th>
                  <th className="num">POC</th>
                  <th className="num">Node</th>
                  <th className="num">Price</th>
                  <th className="num">Dist.</th>
                  <th>Review</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((row) => <ClassificationRow key={row.symbol} row={row} />)}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}

function ClassificationRow({ row }: { row: VolumeProfileClassification }) {
  return (
    <tr>
      <td>
        <strong>{row.symbol}</strong>
        <small className="volumeProfileSubtext">{row.name}</small>
      </td>
      <td>{row.labels.length > 0 ? row.labels.join(", ") : "no_candidate"}</td>
      <td className={row.direction === "long" ? "positive" : row.direction === "short" ? "negative" : ""}>{row.direction}</td>
      <td>{row.anchor_start} → {row.anchor_end}</td>
      <td className="num">{formatPrice(row.poc)}</td>
      <td className="num">{formatBand(row.node_low, row.node_high)}</td>
      <td className="num">{formatPrice(row.latest_price)}</td>
      <td className="num">{row.distance_to_node_pct == null ? "—" : `${row.distance_to_node_pct.toFixed(2)}%`}</td>
      <td className="volumeProfileReview">{row.review_note}</td>
    </tr>
  );
}

function formatPrice(value: number | null): string {
  return value == null ? "—" : value.toFixed(2);
}

function formatBand(low: number | null, high: number | null): string {
  return low == null || high == null ? "—" : `${low.toFixed(2)}–${high.toFixed(2)}`;
}
