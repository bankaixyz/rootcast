import { hasConfirmedBroadcast, RootSnapshot } from "@/lib/api";
import { formatBlock, formatTimestamp, shortHash } from "@/lib/format";
import { sourceTxUrl } from "@/lib/chain-metadata";
import { StatusBadge, toneForDisplayState } from "@/components/status-badge";

export function RecentUpdatesList({ roots }: { roots: RootSnapshot[] }) {
  return (
    <section className="updates-list">
      <div className="updates-list__header">
        <div>
          <div className="eyebrow">Recent updates</div>
          <h3>Recent broadcast history</h3>
        </div>
      </div>

      {roots.length === 0 ? (
        <p className="updates-list__empty">
          No root updates have been persisted yet. The watcher is online and
          waiting for the first Sepolia root submission.
        </p>
      ) : (
        <div className="updates-list__rows">
          {roots.map((root) => (
            <article className="updates-row" key={root.job_id}>
              <div className="updates-row__main">
                <span className="label">Root update</span>
                <span className="updates-row__title">
                  {shortHash(root.root_hex, 14, 10)}
                </span>
                <span className="subtle">{formatTimestamp(root.observed_at)}</span>
              </div>
              <div className="updates-row__stat">
                <span className="label">Source block</span>
                <span>{formatBlock(root.source_block_number)}</span>
              </div>
              <div className="updates-row__stat">
                <span className="label">Status</span>
                <StatusBadge
                  label={root.stage_label}
                  tone={toneForDisplayState(
                    root.job_state === "completed" || hasConfirmedBroadcast(root)
                      ? "confirmed"
                      : root.job_state === "failed"
                        ? "failed"
                        : root.blocked_by
                          ? "blocked"
                          : root.job_state === "submitting"
                            ? "submitting"
                            : "queued",
                  )}
                />
              </div>
              <div className="updates-row__stat">
                <span className="label">Link</span>
                <a
                  className="inline-link"
                  href={sourceTxUrl(root.source_tx_hash)}
                  rel="noreferrer"
                  target="_blank"
                >
                  View L1 tx
                </a>
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
