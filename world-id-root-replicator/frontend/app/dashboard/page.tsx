import { AutoRefresh } from "@/components/auto-refresh";
import { DashboardPage } from "@/components/dashboard-page";
import { getDashboardData } from "@/lib/api";

export default async function DashboardRoute() {
  let snapshot: Awaited<ReturnType<typeof getDashboardData>>["snapshot"] = null;
  let chains: Awaited<ReturnType<typeof getDashboardData>>["chains"] = [];
  let roots: Awaited<ReturnType<typeof getDashboardData>>["roots"] = [];
  let errorMessage: string | null = null;

  try {
    const data = await getDashboardData();
    snapshot = data.snapshot;
    chains = data.chains;
    roots = data.roots;
  } catch (error) {
    errorMessage =
      error instanceof Error
        ? error.message
        : "The dashboard could not load backend data.";
  }

  return (
    <>
      <AutoRefresh intervalMs={12000} />
      <DashboardPage
        snapshot={snapshot}
        chains={chains}
        roots={roots}
        errorMessage={errorMessage}
      />
    </>
  );
}
