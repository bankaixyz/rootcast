type StatusTone =
  | "accent"
  | "success"
  | "warning"
  | "danger"
  | "blocked"
  | "neutral"
  | "subtle";

type StatusBadgeProps = {
  label: string;
  tone?: StatusTone;
};

export function StatusBadge({
  label,
  tone = "neutral",
}: StatusBadgeProps) {
  return <span className={`status-badge status-badge--${tone}`}>{label}</span>;
}

export function toneForDisplayState(
  displayState: "idle" | "blocked" | "queued" | "submitting" | "confirmed" | "failed",
): StatusTone {
  switch (displayState) {
    case "confirmed":
      return "success";
    case "failed":
      return "danger";
    case "submitting":
      return "accent";
    case "queued":
      return "warning";
    case "blocked":
      return "blocked";
    default:
      return "subtle";
  }
}
