import type { ReplicationTarget, RootSnapshot } from "@/lib/api";
import {
  chainLabel,
  chainOrder,
  chainTxUrl,
  sourceTxUrl,
} from "@/lib/chain-metadata";
import { formatBlock, formatTimestamp, shortHash } from "@/lib/format";

type Props = {
  roots: RootSnapshot[];
  maxItems?: number;
};

export function ReplicationHistoryTable({ roots, maxItems }: Props) {
  const completedRoots = roots.filter((root) =>
    root.targets.some((t) => t.tx_hash),
  );
  const visibleRoots =
    maxItems == null ? completedRoots : completedRoots.slice(0, maxItems);

  return (
    <section className="history">
      <div className="history__header">
        <span className="history__eyebrow">Broadcast log</span>
        <h2 className="history__title">Past broadcasts</h2>
      </div>

      {visibleRoots.length === 0 ? (
        <p className="history__empty">
          No completed broadcasts yet. Rows will appear here once the first
          root has been relayed to at least one destination chain.
        </p>
      ) : (
        <div className="history__list">
          {visibleRoots.map((root) => (
            <ReplicationCard key={root.job_id} root={root} />
          ))}
        </div>
      )}
    </section>
  );
}

export function ReplicationCard({ root }: { root: RootSnapshot }) {
  const sortedTargets = [...root.targets]
    .filter((t) => t.tx_hash)
    .sort((a, b) => chainOrder(a.chain_name) - chainOrder(b.chain_name));

  const confirmed = sortedTargets.filter(
    (t) => t.display_state === "confirmed",
  ).length;
  const submitting = sortedTargets.filter(
    (t) => t.display_state === "submitting",
  ).length;
  const failed = sortedTargets.filter(
    (t) => t.display_state === "failed",
  ).length;

  return (
    <article className="history-card">
      <div className="history-card__top">
        <div className="history-card__source">
          <span className="history-card__label">Tx:</span>
          <a
            className="history-card__tx"
            href={sourceTxUrl(root.source_tx_hash)}
            rel="noreferrer"
            target="_blank"
          >
            {shortHash(root.source_tx_hash, 8, 6)}
          </a>
          <span className="history-card__sep">·</span>
          <span className="history-card__block">
            Block {formatBlock(root.source_block_number)}
          </span>
          <span className="history-card__sep">·</span>
          <span className="history-card__time">
            {formatTimestamp(root.updated_at)}
          </span>
        </div>
        <span className="history-card__summary">
          {confirmed > 0 && (
            <span className="history-card__count history-card__count--ok">
              {confirmed} confirmed
            </span>
          )}
          {submitting > 0 && (
            <span className="history-card__count history-card__count--active">
              {submitting} submitting
            </span>
          )}
          {failed > 0 && (
            <span className="history-card__count history-card__count--fail">
              {failed} failed
            </span>
          )}
        </span>
      </div>

      <div className="history-card__grid">
        {sortedTargets.map((target) => (
          <TargetChip key={target.chain_name} target={target} />
        ))}
      </div>
    </article>
  );
}

function TargetChip({ target }: { target: ReplicationTarget }) {
  const fail = target.display_state === "failed";
  const dotClass =
    target.display_state === "failed"
      ? "history-chip__dot--fail"
      : target.display_state === "confirmed"
        ? "history-chip__dot--confirmed"
        : "history-chip__dot--submitting";

  return (
    <a
      className={`history-chip ${fail ? "history-chip--fail" : ""}`}
      href={chainTxUrl(target.chain_name, target.tx_hash!)}
      rel="noreferrer"
      target="_blank"
    >
      <span className={`history-chip__dot ${dotClass}`} />
      <span className="history-chip__chain">
        {chainLabel(target.chain_name)}
      </span>
      <span className="history-chip__hash">
        {shortHash(target.tx_hash!, 6, 4)}
      </span>
    </a>
  );
}
