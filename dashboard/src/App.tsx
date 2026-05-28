import { useEffect, useState } from "react";
import { fetchDashboardForDate, fetchLatestDashboard, fetchScoredDates } from "./api";
import { DashboardPage } from "./components/DashboardPage";
import { ErrorState } from "./components/ErrorState";
import type { DashboardSnapshot } from "./types";

export function App() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [dates, setDates] = useState<string[]>([]);
  const [selectedDate, setSelectedDate] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function loadDashboard(date?: string | null) {
    setLoading(true);
    setError(null);
    try {
      const [data, scoredDates] = await Promise.all([
        date ? fetchDashboardForDate(date) : fetchLatestDashboard(),
        fetchScoredDates()
      ]);
      setSnapshot(data);
      setSelectedDate(data.score_date);
      setDates(scoredDates.length > 0 ? scoredDates : [data.score_date]);
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
        const [data, scoredDates] = await Promise.all([
          fetchLatestDashboard(),
          fetchScoredDates()
        ]);
        if (!ignore) {
          setSnapshot(data);
          setSelectedDate(data.score_date);
          setDates(scoredDates.length > 0 ? scoredDates : [data.score_date]);
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
        <DashboardPage
          data={snapshot}
          dates={dates}
          loading={loading}
          selectedDate={selectedDate ?? snapshot.score_date}
          onDateChange={loadDashboard}
          onRefresh={() => loadDashboard(selectedDate)}
        />
      ) : null}
    </main>
  );
}
