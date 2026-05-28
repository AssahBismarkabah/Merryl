import type { DashboardSnapshot } from "./types";

export async function fetchLatestDashboard(): Promise<DashboardSnapshot> {
  return fetchDashboard("/api/dashboard/latest");
}

export async function fetchDashboardForDate(date: string): Promise<DashboardSnapshot> {
  return fetchDashboard(`/api/dashboard/${encodeURIComponent(date)}`);
}

export async function fetchScoredDates(): Promise<string[]> {
  const response = await fetch("/api/dates");
  if (!response.ok) {
    throw new Error(await errorMessage(response));
  }

  const body = (await response.json()) as { dates: string[] };
  return body.dates;
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
