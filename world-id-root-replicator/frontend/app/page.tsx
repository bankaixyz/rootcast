import { LandingPage } from "@/components/landing-page";
import { getLandingData } from "@/lib/api";
import type { RootSnapshot } from "@/lib/api";

export default async function HomePage() {
  let latestCompleted: RootSnapshot | null = null;

  try {
    const data = await getLandingData();
    latestCompleted =
      data.roots.find((r) => r.targets.some((t) => t.tx_hash)) ?? null;
  } catch {
    // API unreachable — render with static fallback
  }

  return <LandingPage snapshot={latestCompleted} />;
}
