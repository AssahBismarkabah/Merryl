import { useEffect, useState } from "react";
import { fetchLatestDashboard } from "./api";
import { DashboardPage } from "./components/DashboardPage";
import { ErrorState } from "./components/ErrorState";
import type { DashboardSnapshot } from "./types";

export function App() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function loadDashboard() {
    setLoading(true);
    setError(null);
    try {
      setSnapshot(await fetchLatestDashboard());
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load dashboard");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let ignore = false;

    async function load() {
      setLoading(true);
      setError(null);
      try {
        const data = await fetchLatestDashboard();
        if (!ignore) {
          setSnapshot(data);
        }
      } catch (err) {
        if (!ignore) {
          setError(err instanceof Error ? err.message : "Failed to load dashboard");
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
  }, []);

  return (
    <main className="appFrame">
      {error ? <ErrorState message={error} /> : null}
      {loading && !snapshot ? <div className="loading">Loading dashboard data...</div> : null}
      {snapshot ? (
        <DashboardPage data={snapshot} loading={loading} onRefresh={loadDashboard} />
      ) : null}
    </main>
  );
}
