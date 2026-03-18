import {
  useRef,
  useState,
  useEffect,
  useLayoutEffect,
  type CSSProperties,
  type ReactNode,
} from "react";
import type { ChainStatus, RootSnapshot } from "@/lib/api";
import {
  allKnownTargetChains,
  chainAddressUrl,
  chainLabel,
  chainOrder,
  chainTargetLabel,
  chainTxUrl,
  sourceTxUrl,
} from "@/lib/chain-metadata";
import { formatBlock, shortHash } from "@/lib/format";

const TARGET_GAP = 10;

type ReplicationTopologyProps = {
  snapshot: RootSnapshot | null;
  chains: ChainStatus[];
  errorMessage?: string | null;
  hideHeader?: boolean;
  headerContent?: ReactNode;
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

const FLOW_HUB_OFFSET = 6;
const FLOW_SOURCE_CORNER = 14;

export function ReplicationTopology({
  snapshot,
  chains,
  errorMessage,
  hideHeader = false,
  headerContent,
}: ReplicationTopologyProps) {
  const targets = buildTargets(chains, snapshot, Boolean(errorMessage));
  const sourceState = deriveSourceState(snapshot);
  const finalized = Boolean(snapshot?.bankai_finalized_at);
  const showStageContext = Boolean(errorMessage) || snapshot?.job_state !== "completed";
  const stageNumber = deriveStageNumber(snapshot, errorMessage);
  const stageLabel = errorMessage
    ? "Backend unreachable"
    : snapshot?.stage_label ?? "Watching for the next root update";
  const stageDescription = errorMessage
    ? errorMessage
    : snapshot?.stage_description ??
      "The topology stays in place and fills in as soon as the next L1 root update is observed.";
  const stageTone = errorMessage ? "error" : "active";

  const canvasRef = useRef<HTMLDivElement>(null);
  const sourceRef = useRef<HTMLElement | null>(null);
  const targetAreaRef = useRef<HTMLDivElement>(null);

  const [scrollOffset, setScrollOffset] = useState(0);
  const [cardHeight, setCardHeight] = useState(148);
  const [layoutReady, setLayoutReady] = useState(false);
  const [flowLayout, setFlowLayout] = useState({
    canvasWidth: 1000,
    canvasHeight: 640,
    targetAreaHeight: 640,
    targetContentHeight: 0,
    hubX: 350,
    endX: 700,
  });

  // Reset scroll when target count changes
  useEffect(() => {
    setScrollOffset(0);
  }, [targets.length]);

  const { hubX, endX, canvasWidth, canvasHeight, targetAreaHeight, targetContentHeight } =
    flowLayout;
  const stride = cardHeight + TARGET_GAP;
  const contentHeight =
    targetContentHeight ||
    targets.length * cardHeight + Math.max(0, targets.length - 1) * TARGET_GAP;
  const viewportHeight = targetAreaHeight || canvasHeight;
  const canScroll = contentHeight > viewportHeight;
  const maxScroll = Math.max(0, contentHeight - viewportHeight);
  const startOffset = canScroll ? 0 : (viewportHeight - contentHeight) / 2;

  // Measure X positions via ResizeObserver
  useLayoutEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const measure = () => {
      const cr = canvas.getBoundingClientRect();
      const source = sourceRef.current;
      const targetArea = targetAreaRef.current;
      const inner = targetArea?.querySelector(".target-scroll-inner") as HTMLDivElement | null;
      const firstCard = inner?.querySelector(".target-node") as HTMLElement | null;

      if (firstCard) {
        setCardHeight(firstCard.offsetHeight);
      }

      setFlowLayout({
        canvasWidth: cr.width,
        canvasHeight: cr.height,
        targetAreaHeight: targetArea?.clientHeight ?? cr.height,
        targetContentHeight: inner?.scrollHeight ?? 0,
        hubX: source
          ? source.getBoundingClientRect().right - cr.left + FLOW_HUB_OFFSET
          : cr.width * 0.25,
        endX: targetArea
          ? targetArea.getBoundingClientRect().left - cr.left - 10
          : cr.width * 0.7,
      });
      setLayoutReady(true);
    };

    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(canvas);
    return () => ro.disconnect();
  }, []);

  // Wheel-based scrolling on the target list; pass through to page at limits
  useEffect(() => {
    const el = canvasRef.current;
    if (!el || !canScroll) return;

    const handler = (e: WheelEvent) => {
      const atTop = scrollOffset <= 0 && e.deltaY < 0;
      const atBottom = scrollOffset >= maxScroll && e.deltaY > 0;
      if (atTop || atBottom) return;

      e.preventDefault();
      setScrollOffset((prev) =>
        Math.max(0, Math.min(maxScroll, prev + e.deltaY)),
      );
    };

    el.addEventListener("wheel", handler, { passive: false });
    return () => el.removeEventListener("wheel", handler);
  }, [canScroll, maxScroll, scrollOffset]);

  const active = Boolean(snapshot);
  const hubY = canvasHeight / 2;

  const targetYs = targets.map(
    (_, i) => startOffset + i * stride + cardHeight / 2 - scrollOffset,
  );
  const geometries = targets.map((_, i) =>
    buildFlowGeometry(hubX, hubY, endX, targetYs[i]),
  );
  const topGeometries = geometries.filter((geometry) => geometry.direction === -1);
  const bottomGeometries = geometries.filter((geometry) => geometry.direction === 1);
  const sharedFlow = buildSharedFlowGeometry({
    bottomGeometries,
    canvasHeight,
    cardHeight,
    endX,
    hubX,
    hubY,
    topGeometries,
  });
  const sharedSignalClass = active
    ? "flow-branch__signal--confirmed"
    : "flow-branch__signal--idle";

  return (
    <main className="topology-page">
      {!hideHeader && (
        <header className="topology-header">
          <div className="topology-header__copy">
            <span className="topology-header__eyebrow">World ID</span>
            <h1 className="topology-header__title">Rootcast</h1>
          </div>
        </header>
      )}

      <section className="topology-card">
        {headerContent}
        <section
          className="topology-canvas"
          ref={canvasRef}
        >
          <div className="source-column">
            {!hideHeader && showStageContext ? (
              <div className={`stage-context stage-context--${stageTone}`}>
                <div className="stage-context__header">
                  <span className="stage-context__step">Step {stageNumber} of 4</span>
                  <span className="stage-context__dot" />
                </div>
                <span className="stage-context__label">{stageLabel}</span>
                <p className="stage-context__description">{stageDescription}</p>
              </div>
            ) : null}

          <article
            className="source-panel"
            ref={(el) => { sourceRef.current = el; }}
          >
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
          </div>

          {/* SVG flow overlay — absolutely positioned across the full canvas */}
          <svg
            aria-hidden="true"
            className="flow-overlay"
            style={{ opacity: layoutReady ? 1 : 0 }}
            viewBox={`0 0 ${canvasWidth} ${canvasHeight}`}
          >
            <defs>
              <linearGradient id="topology-flow-gradient" x1="0%" x2="100%" y1="0%" y2="0%">
                <stop offset="0%" stopColor="white" stopOpacity="0.02" />
                <stop offset="45%" stopColor="white" stopOpacity="0.48" />
                <stop offset="100%" stopColor="white" stopOpacity="0.16" />
              </linearGradient>
            </defs>

            <line
              className={`flow-trunk ${active ? "flow-trunk--active" : ""}`}
              x1={hubX - FLOW_HUB_OFFSET}
              x2={hubX}
              y1={hubY}
              y2={hubY}
            />
            <path
              className={`flow-branch__signal ${sharedSignalClass}`}
              d={`M ${hubX - FLOW_HUB_OFFSET} ${hubY} L ${hubX} ${hubY}`}
            />
            <circle className="flow-hub" cx={hubX} cy={hubY} r={3.5} />

            {sharedFlow.topPath ? (
              <path
                className="flow-branch__base"
                d={sharedFlow.topPath}
              />
            ) : null}

            {sharedFlow.bottomPath ? (
              <path
                className="flow-branch__base"
                d={sharedFlow.bottomPath}
              />
            ) : null}

            {sharedFlow.topPath ? (
              <path
                className={`flow-branch__signal ${sharedSignalClass}`}
                d={sharedFlow.topPath}
              />
            ) : null}

            {sharedFlow.bottomPath ? (
              <path
                className={`flow-branch__signal ${sharedSignalClass}`}
                d={sharedFlow.bottomPath}
              />
            ) : null}

            {geometries.map((geometry, i) => {
              const target = targets[i];
              const signalClass = flowSignalClass(target.display_state, active);

              return (
                <g
                  className="flow-branch"
                  key={target.chain_name}
                  style={{ "--flow-delay": `${i * 1.1}s` } as CSSProperties}
                >
                  <path
                    className="flow-branch__base"
                    d={geometry.branchPath}
                  />
                  <path
                    className={`flow-branch__signal ${signalClass}`}
                    d={geometry.signalPath}
                  />
                  <circle className="flow-branch__node" cx={endX} cy={geometry.targetY} r={3.5} />
                </g>
              );
            })}
          </svg>

          <div
            className="target-scroll-area"
            ref={targetAreaRef}
            style={{ justifyContent: canScroll ? "flex-start" : "center" }}
          >
            <div
              className="target-scroll-inner"
              style={{
                transform: canScroll
                  ? `translateY(${-scrollOffset}px)`
                  : undefined,
              }}
            >
              {targets.map((target) => (
                <article className="target-node" key={target.chain_name}>
                  <div className="target-node__header">
                    <div>
                      <span className="target-node__eyebrow">Target</span>
                      <h3 className="target-node__title">
                        {chainLabel(target.chain_name)}
                      </h3>
                    </div>
                    <span className={`target-state target-state--${target.display_state}`}>
                      {targetStatusLabel(target.display_state, snapshot)}
                    </span>
                  </div>

                  <div className="target-node__rows">
                    <TargetRow
                      label={chainTargetLabel(target.chain_name)}
                      value={
                        target.registry_address ? (
                          <a
                            className="data-link"
                            href={chainAddressUrl(
                              target.chain_name,
                              target.registry_address,
                            )}
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
                            className={`data-link ${
                              target.display_state === "failed"
                                ? "data-link--failed"
                                : ""
                            }`}
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

            {canScroll && (
              <>
                <div
                  className="target-scroll-fade target-scroll-fade--top"
                  style={{ opacity: scrollOffset > 0 ? 1 : 0 }}
                />
                <div
                  className="target-scroll-fade target-scroll-fade--bottom"
                  style={{ opacity: scrollOffset < maxScroll ? 1 : 0 }}
                />
              </>
            )}
          </div>
        </section>
      </section>
    </main>
  );
}

function deriveStageNumber(
  snapshot: RootSnapshot | null,
  errorMessage?: string | null,
): number {
  if (errorMessage || !snapshot) return 1;

  switch (snapshot.job_state) {
    case "waiting_finality":
      return 2;
    case "ready_to_prove":
    case "proof_in_progress":
      return 3;
    case "proof_ready":
    case "submitting":
    case "failed":
      return 4;
    default:
      return 1;
  }
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

  return "flow-branch__signal--confirmed";
}

type FlowGeometry = {
  branchPath: string;
  direction: -1 | 0 | 1;
  divergenceY: number;
  endX: number;
  signalPath: string;
  spineX: number;
  targetY: number;
};

type SharedFlowGeometry = {
  bottomPath: string | null;
  topPath: string | null;
};

function buildFlowGeometry(
  hubX: number,
  hubY: number,
  endX: number,
  targetY: number,
): FlowGeometry {
  const deltaY = targetY - hubY;

  if (Math.abs(deltaY) < 1) {
    return {
      branchPath: `M ${hubX} ${hubY} L ${endX} ${targetY}`,
      direction: 0,
      divergenceY: targetY,
      endX,
      signalPath: `M ${hubX} ${hubY} L ${endX} ${targetY}`,
      spineX: hubX,
      targetY,
    };
  }

  const direction = deltaY > 0 ? 1 : -1;
  const spineX = flowSpineX(hubX, endX);
  const targetCorner = Math.max(
    4,
    Math.min(18, Math.abs(deltaY) / 2, (endX - spineX) * 0.14),
  );
  const kappa = 0.5522847498;
  const secondHorizontalX = spineX + targetCorner;
  const divergenceY = targetY - direction * targetCorner;

  return {
    branchPath: [
      `M ${spineX} ${divergenceY}`,
      `C ${spineX} ${targetY - direction * targetCorner * (1 - kappa)}, ${secondHorizontalX - targetCorner * kappa} ${targetY}, ${secondHorizontalX} ${targetY}`,
      `L ${endX} ${targetY}`,
    ].join(" "),
    direction,
    divergenceY,
    endX,
    signalPath: [
      `M ${spineX} ${divergenceY}`,
      `C ${spineX} ${targetY - direction * targetCorner * (1 - kappa)}, ${secondHorizontalX - targetCorner * kappa} ${targetY}, ${secondHorizontalX} ${targetY}`,
      `L ${endX} ${targetY}`,
    ].join(" "),
    spineX,
    targetY,
  };
}

function flowSpineX(hubX: number, endX: number) {
  const spanX = Math.max(1, endX - hubX);
  const lead = Math.min(56, Math.max(34, spanX * 0.14));
  return hubX + lead;
}

function buildSharedFlowGeometry({
  bottomGeometries,
  canvasHeight,
  cardHeight,
  endX,
  hubX,
  hubY,
  topGeometries,
}: {
  bottomGeometries: FlowGeometry[];
  canvasHeight: number;
  cardHeight: number;
  endX: number;
  hubX: number;
  hubY: number;
  topGeometries: FlowGeometry[];
}): SharedFlowGeometry {
  const spineX = flowSpineX(hubX, endX);
  const visibilityBuffer = Math.max(18, cardHeight * 0.35);
  const visibleTopGeometries = topGeometries.filter((geometry) =>
    isFlowGeometryVisible(geometry, canvasHeight, visibilityBuffer),
  );
  const visibleBottomGeometries = bottomGeometries.filter((geometry) =>
    isFlowGeometryVisible(geometry, canvasHeight, visibilityBuffer),
  );
  const topExtentY = (visibleTopGeometries.reduce<number | null>(
    (closest, geometry) =>
      closest === null ? geometry.divergenceY : Math.min(closest, geometry.divergenceY),
    null,
  ) ?? null);
  const bottomExtentY = (visibleBottomGeometries.reduce<number | null>(
    (closest, geometry) =>
      closest === null ? geometry.divergenceY : Math.max(closest, geometry.divergenceY),
    null,
  ) ?? null);

  return {
    bottomPath:
      bottomExtentY === null
        ? null
        : buildSharedBranchPath(hubX, hubY, spineX, 1, bottomExtentY),
    topPath:
      topExtentY === null
        ? null
        : buildSharedBranchPath(hubX, hubY, spineX, -1, topExtentY),
  };
}

function isFlowGeometryVisible(
  geometry: FlowGeometry,
  canvasHeight: number,
  visibilityBuffer: number,
) {
  return (
    geometry.targetY >= -visibilityBuffer &&
    geometry.targetY <= canvasHeight + visibilityBuffer
  );
}

function buildSharedBranchPath(
  hubX: number,
  hubY: number,
  spineX: number,
  direction: -1 | 1,
  extentY: number,
) {
  const kappa = 0.5522847498;
  const firstHorizontalX = spineX - FLOW_SOURCE_CORNER;
  const firstVerticalY = hubY + direction * FLOW_SOURCE_CORNER;

  return [
    `M ${hubX} ${hubY}`,
    `L ${firstHorizontalX} ${hubY}`,
    `C ${firstHorizontalX + FLOW_SOURCE_CORNER * kappa} ${hubY}, ${spineX} ${hubY + direction * FLOW_SOURCE_CORNER * (1 - kappa)}, ${spineX} ${firstVerticalY}`,
    `L ${spineX} ${extentY}`,
  ].join(" ");
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
