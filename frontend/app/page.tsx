import { LandingPage } from "@/components/landing-page";
import { getLandingData } from "@/lib/api";
import type { RootSnapshot } from "@/lib/api";

export default async function HomePage() {
  let latestSettledSnapshot: RootSnapshot | null = null;

  try {
    const data = await getLandingData();
    latestSettledSnapshot = data.latestSettledSnapshot;
  } catch {
    // API unreachable — render with static fallback
  }

  return <LandingPage snapshot={latestSettledSnapshot} />;
}
