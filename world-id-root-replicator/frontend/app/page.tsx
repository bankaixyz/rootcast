import { AutoRefresh } from "@/components/auto-refresh";
import { ReplicationTopology } from "@/components/replication-topology";
import { getDashboardData } from "@/lib/api";

export default async function HomePage() {
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
        : "The frontend could not reach the backend API.";
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
