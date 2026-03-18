import type { RootSnapshot } from "@/lib/api";

const STEPS = [
  { key: "observe", label: "Observe" },
  { key: "finalize", label: "Finalize" },
  { key: "prove", label: "Prove" },
  { key: "replicate", label: "Replicate" },
] as const;

type StepKey = (typeof STEPS)[number]["key"];

function activeStep(snapshot: RootSnapshot): StepKey {
  switch (snapshot.job_state) {
    case "waiting_finality":
      return "finalize";
    case "ready_to_prove":
    case "proof_in_progress":
      return "prove";
    case "proof_ready":
    case "submitting":
    case "completed":
    case "failed":
      return "replicate";
    default:
      return "observe";
  }
}

function isStepComplete(snapshot: RootSnapshot, key: StepKey): boolean {
  const order: StepKey[] = ["observe", "finalize", "prove", "replicate"];
  const activeIdx = order.indexOf(activeStep(snapshot));
  const stepIdx = order.indexOf(key);
  return stepIdx < activeIdx || snapshot.job_state === "completed";
}

type PipelineIndicatorProps = {
  snapshot: RootSnapshot | null;
  errorMessage?: string | null;
};

export function PipelineIndicator({
  snapshot,
  errorMessage,
}: PipelineIndicatorProps) {
  if (!snapshot) {
    return (
      <section className="pipeline">
        <div className="pipeline__idle">
          <span className="pipeline__idle-dot" />
          Watching for the next root update&hellip;
        </div>
      </section>
    );
  }

  const allComplete = snapshot.job_state === "completed";
  const current = allComplete ? null : activeStep(snapshot);
  const failed = snapshot.job_state === "failed";

  const stageLabel = errorMessage
    ? "Backend unreachable"
    : snapshot.stage_label;
  const stageDescription = errorMessage
    ? errorMessage
    : snapshot.stage_description;

  return (
    <section className="pipeline">
      <div className="pipeline__row">
        {stageLabel && (
          <div className="pipeline__stage">
            <span className="pipeline__stage-label">{stageLabel}</span>
            {stageDescription && (
              <span className="pipeline__stage-desc">{stageDescription}</span>
            )}
          </div>
        )}

        <div className="pipeline__steps">
          {STEPS.map((step, i) => {
            const complete = isStepComplete(snapshot, step.key);
            const isCurrent = !allComplete && step.key === current;
            const isFailed = failed && isCurrent;

            const cls = [
              "pipeline__step",
              complete ? "pipeline__step--complete" : "",
              isCurrent ? "pipeline__step--current" : "",
              isFailed ? "pipeline__step--failed" : "",
            ]
              .filter(Boolean)
              .join(" ");

            return (
              <div className={cls} key={step.key}>
                <span className="pipeline__step-num">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <span className="pipeline__step-label">{step.label}</span>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
