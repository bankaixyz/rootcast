import { ReplicationTarget } from "@/lib/api";
import { chainLabel, chainTargetLabel, chainTxUrl } from "@/lib/chain-metadata";
import { shortHash } from "@/lib/format";
import { StatusBadge, toneForDisplayState } from "@/components/status-badge";

export function ReplicationTargetCard({
  target,
}: {
  target: ReplicationTarget;
}) {
  const failed = target.display_state === "failed";

  return (
    <article className="target-card">
      <div className="target-card__header">
        <div>
          <div className="label">Broadcast target</div>
          <h3 className="target-card__title">{chainLabel(target.chain_name)}</h3>
        </div>
        <StatusBadge
          label={target.display_state}
          tone={toneForDisplayState(target.display_state)}
        />
      </div>

      <div className="target-card__body">
        <p>
          {target.blocked_reason ??
            target.error_message ??
            "Submission state is flowing normally for this target."}
        </p>
      </div>

      <div className="target-card__footer">
        <span>
          <span className="label">{chainTargetLabel(target.chain_name)}</span>{" "}
          <code className="mono-value">{shortHash(target.registry_address, 10, 8)}</code>
        </span>
        <span>
          <span className="label">Chain id</span> {target.chain_id}
        </span>
        {target.tx_hash ? (
          <a
            className={`inline-link ${failed ? "inline-link--failed" : ""}`}
            href={chainTxUrl(target.chain_name, target.tx_hash)}
            rel="noreferrer"
            target="_blank"
          >
            View destination tx
          </a>
        ) : (
          <span className="subtle">No destination tx yet</span>
        )}
      </div>
    </article>
  );
}
