import type { CSSProperties, ReactNode } from "react";
import type { ChainStatus, RootSnapshot } from "@/lib/api";
import {
  allKnownTargetChains,
  chainAddressUrl,
  chainLabel,
  chainOrder,
  chainTxUrl,
  sourceTxUrl,
} from "@/lib/chain-metadata";
import { formatBlock, shortHash } from "@/lib/format";

type ReplicationTopologyProps = {
  snapshot: RootSnapshot | null;
  chains: ChainStatus[];
  errorMessage?: string | null;
};

type TopologyTarget = {
  chain_name: string;
  registry_address: string | null;
  display_state:
    | "idle"
    | "blocked"
    | "queued"
    | "submitting"
    | "confirmed"
    | "failed";
  tx_hash: string | null;
  blocked_reason: string | null;
  error_message: string | null;
};

export function ReplicationTopology({
  snapshot,
  chains,
  errorMessage,
}: ReplicationTopologyProps) {
  const targets = buildTargets(chains, snapshot, Boolean(errorMessage));
  const flowHeight = Math.max(280, 140 + targets.length * 108);
  const sourceState = deriveSourceState(snapshot);
  const finalized = Boolean(snapshot?.bankai_finalized_at);
  const showStageContext = Boolean(errorMessage) || snapshot?.job_state !== "completed";
  const stageLabel = errorMessage
    ? "Backend unreachable"
    : snapshot?.stage_label ?? "Watching for the next root update";
  const stageDescription = errorMessage
    ? errorMessage
    : snapshot?.stage_description ??
      "The topology stays in place and fills in as soon as the next L1 root update is observed.";

  return (
    <main className="topology-page">
      <header className="topology-header">
        <div className="topology-header__copy">
          <span className="topology-header__eyebrow">World ID Root Replicator</span>
          <h1 className="topology-header__title">World ID Replication</h1>
        </div>

        {showStageContext ? (
          <div className="topology-header__status">
            <span className="topology-header__stage">{stageLabel}</span>
            <p className="topology-header__description">{stageDescription}</p>
          </div>
        ) : null}
      </header>

      <section className="topology-card">
        <section className="topology-canvas">
          <article className="source-panel">
            <div className="source-panel__header">
              <span className="source-panel__eyebrow">L1 source</span>
              <h2 className="source-panel__title">Ethereum Sepolia</h2>
              <p className="source-panel__meta">
                {snapshot
                  ? `Block ${formatBlock(snapshot.source_block_number)}`
                  : "Waiting for the next root update"}
              </p>
            </div>

            <div className="source-panel__rows">
              <SourceRow
                label="Root"
                value={
                  snapshot ? (
                    <span className="data-row__monoText">
                      {shortHash(snapshot.root_hex, 12, 8)}
                    </span>
                  ) : (
                    "Waiting"
                  )
                }
                tone={snapshot ? "live" : "muted"}
              />
              <SourceRow
                label="Tx received"
                value={
                  snapshot ? (
                    <a
                      className="data-link"
                      href={sourceTxUrl(snapshot.source_tx_hash)}
                      rel="noreferrer"
                      target="_blank"
                    >
                      {shortHash(snapshot.source_tx_hash, 10, 6)}
                    </a>
                  ) : (
                    "Waiting"
                  )
                }
                tone={snapshot ? "live" : "muted"}
              />
              <SourceRow
                label="Finalized"
                value={finalized ? "True" : "Waiting"}
                tone={finalized ? "success" : "muted"}
              />
              <SourceRow
                label="Proven"
                value={sourceState.proofReady ? "Yes" : "No"}
                tone={sourceState.proofReady ? "success" : "muted"}
              />
              <SourceRow
                label="Replication"
                value={sourceState.replicationLabel}
                tone={sourceState.replicationTone}
              />
            </div>

            <div className="source-panel__footer">
              <span
                className={`source-panel__signal source-panel__signal--${
                  errorMessage ? "error" : snapshot ? "live" : "muted"
                }`}
              />
              <span className="source-panel__footerText">
                {errorMessage
                  ? "Live snapshot unavailable"
                  : snapshot
                    ? stageLabel
                    : "Source watcher online"}
              </span>
            </div>
          </article>

          <TopologyFlow
            active={Boolean(snapshot)}
            height={flowHeight}
            targets={targets}
          />

          <div
            className="target-column"
            style={{ minHeight: `${flowHeight}px` } as CSSProperties}
          >
            {targets.map((target) => (
              <article className="target-node" key={target.chain_name}>
                <div className="target-node__header">
                  <div>
                    <span className="target-node__eyebrow">Target</span>
                    <h3 className="target-node__title">{chainLabel(target.chain_name)}</h3>
                  </div>
                  <span className={`target-state target-state--${target.display_state}`}>
                    {targetStatusLabel(target.display_state, snapshot)}
                  </span>
                </div>

                <div className="target-node__rows">
                  <TargetRow
                    label="Contract"
                    value={
                      target.registry_address ? (
                        <a
                          className="data-link"
                          href={chainAddressUrl(target.chain_name, target.registry_address)}
                          rel="noreferrer"
                          target="_blank"
                        >
                          {shortHash(target.registry_address, 8, 6)}
                        </a>
                      ) : (
                        "Pending"
                      )
                    }
                  />
                  <TargetRow
                    label="Tx"
                    value={
                      target.tx_hash ? (
                        <a
                          className="data-link"
                          href={chainTxUrl(target.chain_name, target.tx_hash)}
                          rel="noreferrer"
                          target="_blank"
                        >
                          {shortHash(target.tx_hash, 8, 6)}
                        </a>
                      ) : target.display_state === "blocked" ? (
                        snapshot?.blocked_by === "bankai_finality" ? (
                          "Waiting for finality"
                        ) : (
                          "Waiting for proof"
                        )
                      ) : (
                        "Waiting"
                      )
                    }
                  />
                </div>

                <p className="target-node__note">
                  {target.error_message ??
                    target.blocked_reason ??
                    fallbackTargetNote(target.display_state)}
                </p>
              </article>
            ))}
          </div>
        </section>
      </section>
    </main>
  );
}

function deriveSourceState(snapshot: RootSnapshot | null) {
  if (!snapshot) {
    return {
      proofReady: false,
      replicationLabel: "Waiting",
      replicationTone: "muted" as const,
    };
  }

  const proofReady =
    snapshot.proof_ready === true ||
    ["proof_ready", "submitting", "completed"].includes(snapshot.job_state) ||
    snapshot.targets.some((target) =>
      ["queued", "submitting", "confirmed", "failed"].includes(target.display_state),
    );

  if (snapshot.job_state === "completed") {
    return {
      proofReady,
      replicationLabel: "Complete",
      replicationTone: "success" as const,
    };
  }

  if (
    snapshot.job_state === "submitting" ||
    snapshot.targets.some((target) => target.display_state === "submitting")
  ) {
    return {
      proofReady,
      replicationLabel: "In flight",
      replicationTone: "live" as const,
    };
  }

  if (
    snapshot.replication_triggered === true ||
    snapshot.targets.some((target) =>
      ["queued", "confirmed", "failed"].includes(target.display_state),
    )
  ) {
    return {
      proofReady,
      replicationLabel: "Triggered",
      replicationTone: "live" as const,
    };
  }

  return {
    proofReady,
    replicationLabel: "Waiting",
    replicationTone: "muted" as const,
  };
}

function SourceRow({
  label,
  value,
  tone,
}: {
  label: string;
  value: ReactNode;
  tone: "muted" | "live" | "success";
}) {
  return (
    <div className="data-row">
      <span className="data-row__label">{label}</span>
      <span className={`data-row__value data-row__value--${tone}`}>{value}</span>
    </div>
  );
}

function TargetRow({
  label,
  value,
}: {
  label: string;
  value: ReactNode;
}) {
  return (
    <div className="data-row data-row--compact">
      <span className="data-row__label">{label}</span>
      <span className="data-row__value data-row__value--mono">{value}</span>
    </div>
  );
}

function TopologyFlow({
  active,
  height,
  targets,
}: {
  active: boolean;
  height: number;
  targets: TopologyTarget[];
}) {
  const width = 520;
  const hubX = 144;
  const endX = width - 24;
  const centerY = height / 2;
  const targetYs = distributedPositions(height, targets.length);

  return (
    <div className="flow-zone" style={{ height: `${height}px` } as CSSProperties}>
      <svg
        aria-hidden="true"
        className="flow-zone__svg"
        preserveAspectRatio="none"
        viewBox={`0 0 ${width} ${height}`}
      >
        <defs>
          <linearGradient id="topology-flow-gradient" x1="0%" x2="100%" y1="0%" y2="0%">
            <stop offset="0%" stopColor="white" stopOpacity="0.02" />
            <stop offset="45%" stopColor="white" stopOpacity="0.48" />
            <stop offset="100%" stopColor="white" stopOpacity="0.16" />
          </linearGradient>
          <marker
            id="topology-flow-arrow"
            markerHeight="10"
            markerWidth="10"
            orient="auto"
            refX="8"
            refY="5"
          >
            <path d="M0,0 L10,5 L0,10 Z" fill="white" fillOpacity="0.34" />
          </marker>
        </defs>

        <line
          className={`flow-trunk ${active ? "flow-trunk--active" : ""}`}
          x1={0}
          x2={hubX}
          y1={centerY}
          y2={centerY}
        />
        <circle className="flow-hub" cx={hubX} cy={centerY} r={4.5} />

        {targets.map((target, index) => {
          const y = targetYs[index];
          const d = `M ${hubX} ${centerY} C ${hubX + 88} ${centerY}, ${hubX + 142} ${y}, ${endX} ${y}`;
          const signalClass = flowSignalClass(target.display_state, active);

          return (
            <g
              className="flow-branch"
              key={target.chain_name}
              style={{ "--flow-delay": `${index * 1.1}s` } as CSSProperties}
            >
              <path
                className="flow-branch__base"
                d={d}
                markerEnd="url(#topology-flow-arrow)"
              />
              <path className={`flow-branch__signal ${signalClass}`} d={d} />
              <circle className="flow-branch__node" cx={endX} cy={y} r={3.5} />
            </g>
          );
        })}
      </svg>
    </div>
  );
}

function buildTargets(
  chains: ChainStatus[],
  snapshot: RootSnapshot | null,
  hasError: boolean,
) {
  const merged = new Map<string, TopologyTarget>();

  for (const chain of chains) {
    merged.set(chain.chain_name, {
      chain_name: chain.chain_name,
      registry_address: chain.registry_address,
      display_state: chain.display_state,
      tx_hash: chain.tx_hash,
      blocked_reason: chain.blocked_reason,
      error_message: chain.error_message,
    });
  }

  for (const target of snapshot?.targets ?? []) {
    if (!merged.has(target.chain_name)) {
      merged.set(target.chain_name, {
        chain_name: target.chain_name,
        registry_address: target.registry_address,
        display_state: target.display_state,
        tx_hash: target.tx_hash,
        blocked_reason: target.blocked_reason,
        error_message: target.error_message,
      });
    }
  }

  if (merged.size === 0 && hasError) {
    for (const chainName of allKnownTargetChains()) {
      merged.set(chainName, {
        chain_name: chainName,
        registry_address: null,
        display_state: "idle",
        tx_hash: null,
        blocked_reason: null,
        error_message: null,
      });
    }
  }

  return [...merged.values()].sort(
    (left, right) => chainOrder(left.chain_name) - chainOrder(right.chain_name),
  );
}

function distributedPositions(height: number, count: number) {
  if (count <= 1) {
    return [height / 2];
  }

  const top = 54;
  const bottom = height - 54;
  const step = (bottom - top) / (count - 1);

  return Array.from({ length: count }, (_, index) => top + step * index);
}

function flowSignalClass(
  displayState: TopologyTarget["display_state"],
  active: boolean,
) {
  if (!active) {
    return "flow-branch__signal--idle";
  }

  if (displayState === "failed") {
    return "flow-branch__signal--failed";
  }

  if (displayState === "confirmed") {
    return "flow-branch__signal--confirmed";
  }

  if (displayState === "queued" || displayState === "submitting") {
    return "flow-branch__signal--fanout";
  }

  return "flow-branch__signal--active";
}

function targetStatusLabel(
  displayState: TopologyTarget["display_state"],
  snapshot: RootSnapshot | null,
) {
  switch (displayState) {
    case "confirmed":
      return "Confirmed";
    case "submitting":
      return "Submitting";
    case "queued":
      return "Queued";
    case "failed":
      return "Failed";
    case "blocked":
      return snapshot?.blocked_by === "bankai_finality"
        ? "Waiting for finality"
        : "Waiting for proof";
    default:
      return "Idle";
  }
}

function fallbackTargetNote(displayState: TopologyTarget["display_state"]) {
  switch (displayState) {
    case "confirmed":
      return "Replication completed for this target.";
    case "submitting":
      return "Transaction submitted and awaiting confirmation.";
    case "queued":
      return "Ready to receive the shared proof.";
    case "failed":
      return "The target needs another replication attempt.";
    case "blocked":
      return "This target is waiting on the shared upstream stage.";
    default:
      return "This target is ready when the next root arrives.";
  }
}
