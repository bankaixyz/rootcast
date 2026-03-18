import { ReplicationTarget } from "@/lib/api";
import { ReplicationTargetCard } from "@/components/replication-target-card";

export function ReplicationTargetGrid({
  targets,
}: {
  targets: ReplicationTarget[];
}) {
  return (
    <section className="target-grid">
      {targets.map((target) => (
        <ReplicationTargetCard key={target.chain_name} target={target} />
      ))}
    </section>
  );
}
