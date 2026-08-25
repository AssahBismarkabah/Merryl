import { useEffect, useState } from "react";
import { fetchVolumeProfileClassifications } from "../api";
import type {
  VolumeProfileClassification,
  VolumeProfileClassificationResponse,
} from "../types";

export function VolumeProfilePage() {
  const [data, setData] = useState<VolumeProfileClassificationResponse | null>(null);
  const [showAll, setShowAll] = useState(false);
  const [expandedSymbol, setExpandedSymbol] = useState<string | null>(null);
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
                {rows.map((row) => (
                  <ClassificationRow
                    key={`${row.symbol}-${row.anchor_start}-${row.structure_kind}`}
                    row={row}
                    expanded={expandedSymbol === row.symbol}
                    onToggle={() => setExpandedSymbol(expandedSymbol === row.symbol ? null : row.symbol)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}

function ClassificationRow({
  row,
  expanded,
  onToggle,
}: {
  row: VolumeProfileClassification;
  expanded: boolean;
  onToggle: () => void;
}) {
  return (
    <>
    <tr className="volumeProfileClickable" onClick={onToggle}>
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
    {expanded ? (
      <tr>
        <td colSpan={9}>
          <StructurePreview row={row} />
        </td>
      </tr>
    ) : null}
    </>
  );
}

function StructurePreview({ row }: { row: VolumeProfileClassification }) {
  const width = 720;
  const height = 230;
  const chartWidth = 500;
  const profileWidth = 180;
  const prices = row.chart_bars.flatMap((bar) => [bar.high, bar.low]);
  const profilePrices = row.profile_rows.flatMap((profile) => [profile.price_low, profile.price_high]);
  const min = Math.min(...prices, ...profilePrices);
  const max = Math.max(...prices, ...profilePrices);
  const span = Math.max(max - min, 0.000001);
  const xStep = chartWidth / Math.max(row.chart_bars.length - 1, 1);
  const y = (price: number) => height - 18 - ((price - min) / span) * (height - 36);
  const maxVolume = Math.max(...row.profile_rows.map((profile) => profile.volume_pct), 0.000001);

  return (
    <div className="volumeProfilePreview">
      <div>
        <strong>{row.structure_kind} structure preview</strong>
        <span>Click the row again to collapse. Daily approximation; review the selected anchor on the real chart.</span>
      </div>
      <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`${row.symbol} structure and volume profile`}>
        <line x1={0} y1={height - 18} x2={chartWidth} y2={height - 18} className="previewAxis" />
        {row.chart_bars.map((bar, index) => {
          const x = index * xStep;
          const color = bar.close >= bar.open ? "var(--positive)" : "var(--negative)";
          return (
            <g key={bar.date}>
              <line x1={x} y1={y(bar.high)} x2={x} y2={y(bar.low)} stroke={color} strokeWidth="1" />
              <rect x={x - 2.5} y={Math.min(y(bar.open), y(bar.close))} width="5" height={Math.max(Math.abs(y(bar.close) - y(bar.open)), 1)} fill={color} />
            </g>
          );
        })}
        {row.node_low != null && row.node_high != null ? (
          <rect x={0} y={y(row.node_high)} width={chartWidth} height={Math.max(y(row.node_low) - y(row.node_high), 1)} className="previewNode" />
        ) : null}
        {row.poc != null ? <line x1={0} y1={y(row.poc)} x2={chartWidth} y2={y(row.poc)} className="previewPoc" /> : null}
        <g transform={`translate(${chartWidth + 15}, 0)`}>
          {row.profile_rows.map((profile) => {
            const barWidth = (profile.volume_pct / maxVolume) * profileWidth;
            return <rect key={`${profile.price_low}-${profile.price_high}`} x={profileWidth - barWidth} y={y(profile.price_high)} width={barWidth} height={Math.max(y(profile.price_low) - y(profile.price_high), 1)} className="previewVolume" />;
          })}
        </g>
        <text x={8} y={16} className="previewLabel">Anchor: {row.anchor_start} → {row.anchor_end}</text>
        <text x={chartWidth + 18} y={16} className="previewLabel">Volume by price</text>
      </svg>
    </div>
  );
}

function formatPrice(value: number | null): string {
  return value == null ? "—" : value.toFixed(2);
}

function formatBand(low: number | null, high: number | null): string {
  return low == null || high == null ? "—" : `${low.toFixed(2)}–${high.toFixed(2)}`;
}
