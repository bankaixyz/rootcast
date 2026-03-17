import { AutoRefresh } from "@/components/auto-refresh";
import { ReplicationTopology } from "@/components/replication-topology";
import { getDashboardData } from "@/lib/api";

export default async function DashboardPage() {
  let snapshot: Awaited<ReturnType<typeof getDashboardData>>["snapshot"] = null;
  let chains: Awaited<ReturnType<typeof getDashboardData>>["chains"] = [];
  let errorMessage: string | null = null;

  try {
    const data = await getDashboardData();
    snapshot = data.snapshot;
    chains = data.chains;
  } catch (error) {
    errorMessage =
      error instanceof Error
        ? error.message
        : "The dashboard could not load backend data.";
  }

  return (
    <>
      <AutoRefresh intervalMs={12000} />
      <ReplicationTopology
        chains={chains}
        errorMessage={errorMessage}
        snapshot={snapshot}
      />
    </>
  );
}
