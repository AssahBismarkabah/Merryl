import type { DashboardSnapshot } from "./types";

export async function fetchLatestDashboard(): Promise<DashboardSnapshot> {
  const response = await fetch("/api/dashboard/latest");
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { message?: string } | null;
    throw new Error(body?.message ?? `Dashboard request failed with ${response.status}`);
  }

  return response.json() as Promise<DashboardSnapshot>;
}
