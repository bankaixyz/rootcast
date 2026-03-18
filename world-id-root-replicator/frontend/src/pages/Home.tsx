import { LandingPage } from "@/components/landing-page";
import { getLandingData } from "@/lib/api";
import { usePolling } from "@/lib/use-polling";

export function Home() {
  const { data } = usePolling(getLandingData, 30_000);
  return <LandingPage snapshot={data?.snapshot ?? null} />;
}
