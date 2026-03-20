import { hasConfirmedBroadcast, RootSnapshot } from "@/lib/api";
import { formatBlock, formatTimestamp, shortHash } from "@/lib/format";
import { sourceTxUrl } from "@/lib/chain-metadata";
import { StatusBadge, toneForDisplayState } from "@/components/status-badge";

type RootUpdateCardProps = {
  snapshot: RootSnapshot;
  title: string;
  summary: string;
  hero?: boolean;
};

function stageTone(snapshot: RootSnapshot) {
  if (snapshot.job_state === "completed" || hasConfirmedBroadcast(snapshot)) {
    return "confirmed";
  }

  if (snapshot.job_state === "failed") {
    return "failed";
  }

  if (snapshot.blocked_by) {
    return "blocked";
  }

  if (snapshot.job_state === "submitting") {
    return "submitting";
  }

  return "queued";
}

export function RootUpdateCard({
  snapshot,
  title,
  summary,
  hero = false,
}: RootUpdateCardProps) {
  const pendingCount =
    snapshot.targets.length -
    snapshot.confirmed_target_count -
    snapshot.failed_target_count;

  return (
    <section className={`panel root-card ${hero ? "panel--hero" : ""}`}>
      <div className="root-card__header">
        <div className="root-card__headline">
          <span className="eyebrow">{title}</span>
          <h2>{summary}</h2>
          <p>{snapshot.stage_description}</p>
        </div>
        <StatusBadge
          label={snapshot.stage_label}
          tone={toneForDisplayState(stageTone(snapshot))}
        />
      </div>

      <div className="root-card__headline">
        <span className="label">World ID root</span>
        <code className="root-card__rootValue mono-value">
          {shortHash(snapshot.root_hex, hero ? 20 : 14, 12)}
        </code>
      </div>

      <div className="root-card__meta">
        <div className="stat">
          <span className="label">Source block</span>
          <span className="stat__value">{formatBlock(snapshot.source_block_number)}</span>
        </div>
        <div className="stat">
          <span className="label">Confirmed targets</span>
          <span className="stat__value">{snapshot.confirmed_target_count}</span>
        </div>
        <div className="stat">
          <span className="label">Pending targets</span>
          <span className="stat__value">{pendingCount}</span>
        </div>
        <div className="stat">
          <span className="label">Failed targets</span>
          <span className="stat__value">{snapshot.failed_target_count}</span>
        </div>
      </div>

      <div className="root-card__footer">
        <span>Observed {formatTimestamp(snapshot.observed_at)}</span>
        <span>
          Finality{" "}
          {snapshot.bankai_finalized_at
            ? `cleared ${formatTimestamp(snapshot.bankai_finalized_at)}`
            : "still pending"}
        </span>
        <a
          className="inline-link"
          href={sourceTxUrl(snapshot.source_tx_hash)}
          rel="noreferrer"
          target="_blank"
        >
          View L1 transaction
        </a>
      </div>
    </section>
  );
}
