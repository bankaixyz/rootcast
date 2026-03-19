import { RootSnapshot } from "@/lib/api";
import { StatusBadge } from "@/components/status-badge";

type Stage = {
  key: string;
  title: string;
  body: string;
};

const STAGES: Stage[] = [
  {
    key: "l1",
    title: "L1 update observed",
    body: "Watch the Sepolia source transaction and persist the exact block that changed the root.",
  },
  {
    key: "finality",
    title: "Bankai finality",
    body: "Wait for the exact L1 source block to become finalized in Bankai's finalized view.",
  },
  {
    key: "proving",
    title: "Shared SP1 proof",
    body: "Generate one proof artifact that every broadcast target can consume.",
  },
  {
    key: "targets",
    title: "Destination fan-out",
    body: "Submit and confirm the same proof across every configured target chain.",
  },
];

function currentStage(snapshot: RootSnapshot) {
  switch (snapshot.job_state) {
    case "waiting_finality":
      return "finality";
    case "ready_to_prove":
    case "proof_in_progress":
      return "proving";
    case "proof_ready":
    case "submitting":
    case "completed":
    case "failed":
      return "targets";
    default:
      return "l1";
  }
}

function isComplete(snapshot: RootSnapshot, stageKey: string) {
  if (stageKey === "l1") {
    return true;
  }

  if (stageKey === "finality") {
    return snapshot.bankai_finalized_at !== null;
  }

  if (stageKey === "proving") {
    return (
      snapshot.job_state === "proof_ready" ||
      snapshot.job_state === "submitting" ||
      snapshot.job_state === "completed" ||
      snapshot.job_state === "failed"
    );
  }

  return snapshot.job_state === "completed";
}

export function PipelineStageBanner({ snapshot }: { snapshot: RootSnapshot }) {
  const activeStage = currentStage(snapshot);

  return (
    <section className="stage-banner">
      {STAGES.map((stage) => {
        const complete = isComplete(snapshot, stage.key);
        const current = stage.key === activeStage;
        const failed = snapshot.job_state === "failed" && current;

        return (
          <article
            className={[
              "stage-banner__item",
              complete ? "stage-banner__item--complete" : "",
              current ? "stage-banner__item--current" : "",
              failed ? "stage-banner__item--failed" : "",
            ]
              .filter(Boolean)
              .join(" ")}
            key={stage.key}
          >
            <StatusBadge
              label={
                failed
                  ? "Failed"
                  : current
                    ? "Current"
                    : complete
                      ? "Complete"
                      : "Upcoming"
              }
              tone={
                failed
                  ? "danger"
                  : current
                    ? "accent"
                    : complete
                      ? "success"
                      : "subtle"
              }
            />
            <h3 className="stage-banner__title">{stage.title}</h3>
            <p className="stage-banner__body">{stage.body}</p>
          </article>
        );
      })}
    </section>
  );
}
