import { DashboardPage } from "@/components/dashboard-page";
import { getDashboardData } from "@/lib/api";
import { usePolling } from "@/lib/use-polling";

export function Dashboard() {
  const { data, error } = usePolling(getDashboardData, 12_000);

  return (
    <DashboardPage
      snapshot={data?.snapshot ?? null}
      chains={data?.chains ?? []}
      roots={data?.roots ?? []}
      errorMessage={error}
    />
  );
}
