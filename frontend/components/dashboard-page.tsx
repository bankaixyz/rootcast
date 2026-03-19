import type { RootSnapshot, ChainStatus } from "@/lib/api";
import { PipelineIndicator } from "@/components/pipeline-indicator";
import { ReplicationTopology } from "@/components/replication-topology";
import { ReplicationHistoryTable } from "@/components/replication-history-table";
import { Navbar } from "@/components/navbar";
import { SiteFooter } from "@/components/site-footer";

type DashboardPageProps = {
  snapshot: RootSnapshot | null;
  chains: ChainStatus[];
  roots: RootSnapshot[];
  errorMessage: string | null;
};

export function DashboardPage({
  snapshot,
  chains,
  roots,
  errorMessage,
}: DashboardPageProps) {
  return (
    <div className="dashboard">
      <Navbar currentPage="dashboard" />

      {errorMessage && (
        <div className="dash-error">
          <p>{errorMessage}</p>
        </div>
      )}

      <div className="dash-topology">
        <ReplicationTopology
          snapshot={snapshot}
          chains={chains}
          errorMessage={errorMessage}
          hideHeader
          headerContent={
            <div className="topology-card__header">
              <div className="topology-card__title-block">
                <span className="topology-card__eyebrow">Live status</span>
                <h2 className="topology-card__title">Current Broadcast</h2>
              </div>
              <PipelineIndicator
                snapshot={snapshot}
                errorMessage={errorMessage}
              />
            </div>
          }
        />
      </div>

      <div className="dash-history-container">
        <ReplicationHistoryTable roots={roots} maxItems={10} />
      </div>

      <SiteFooter />
    </div>
  );
}
