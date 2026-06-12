import type { DashboardSnapshot } from "./types";

const STATIC_MODE = import.meta.env.VITE_MERRYL_STATIC_DASHBOARD === "true";
const STATIC_DATA_BASE = `${import.meta.env.BASE_URL}static-data`;

export async function fetchLatestDashboard(): Promise<DashboardSnapshot> {
  if (STATIC_MODE) {
    return fetchDashboard(`${STATIC_DATA_BASE}/latest.json`);
  }

  return fetchDashboard("/api/dashboard/latest");
}

export async function fetchDashboardForDate(date: string): Promise<DashboardSnapshot> {
  if (STATIC_MODE) {
    return fetchDashboard(`${STATIC_DATA_BASE}/dashboard/${encodeURIComponent(date)}.json`);
  }

  return fetchDashboard(`/api/dashboard/${encodeURIComponent(date)}`);
}

export async function fetchScoredDates(): Promise<string[]> {
  if (STATIC_MODE) {
    const response = await fetch(`${STATIC_DATA_BASE}/dates.json`);
    if (!response.ok) {
      throw new Error(await errorMessage(response));
    }

    const body = (await response.json()) as { dates: string[] };
    return body.dates ?? [];
  }

  const response = await fetch("/api/dates");
  if (!response.ok) {
    throw new Error(await errorMessage(response));
  }

  const body = (await response.json()) as { dates: string[] };
  return body.dates ?? [];
}

async function fetchDashboard(url: string): Promise<DashboardSnapshot> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(await errorMessage(response));
  }

  return response.json() as Promise<DashboardSnapshot>;
}

async function errorMessage(response: Response): Promise<string> {
  const body = (await response.json().catch(() => null)) as { message?: string } | null;
  return body?.message ?? `Dashboard request failed with ${response.status}`;
}
