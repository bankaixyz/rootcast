export type StatusResponse = {
  phase: string;
  service: string;
  status: string;
  destination_chain_count: number;
  latest_observed_source_block: number | null;
  latest_proof_request_age_seconds: number | null;
  current_stage_label: string | null;
  current_source_block_number: number | null;
};

export type ReplicationTarget = {
  chain_name: string;
  chain_id: number;
  registry_address: string;
  submission_state: string;
  tx_hash: string | null;
  error_message: string | null;
  retry_count: number;
  display_state: "blocked" | "queued" | "submitting" | "confirmed" | "failed";
  blocked_reason: string | null;
};

export type RootSnapshot = {
  job_id: number;
  observed_root_id: number;
  root_hex: string;
  source_block_number: number;
  source_tx_hash: string;
  observed_at: string;
  bankai_finalized_at: string | null;
  bankai_finalized_block_number?: number | null;
  observed_root_status: string;
  job_state: string;
  proof_ready?: boolean;
  replication_triggered?: boolean;
  stage_label: string;
  stage_description: string;
  blocked_by: "bankai_finality" | "proving" | null;
  error_message: string | null;
  retry_count: number;
  confirmed_target_count: number;
  failed_target_count: number;
  targets: ReplicationTarget[];
};

export type ChainStatus = {
  chain_name: string;
  chain_id: number;
  registry_address: string;
  latest_job_id: number | null;
  latest_root_hex: string | null;
  latest_source_block_number: number | null;
  submission_state: string | null;
  display_state: "idle" | "blocked" | "queued" | "submitting" | "confirmed" | "failed";
  blocked_reason: string | null;
  tx_hash: string | null;
  error_message: string | null;
};

type LatestRootResponse = {
  snapshot: RootSnapshot | null;
};

type RootsResponse = {
  roots: RootSnapshot[];
};

type ChainsResponse = {
  chains: ChainStatus[];
};

type ApiErrorResponse = {
  error?: string;
};

const API_ORIGIN =
  process.env.WORLD_ID_REPLICATOR_API_ORIGIN ?? "http://127.0.0.1:3001";

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(`${API_ORIGIN}${path}`, {
    cache: "no-store",
  });

  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;

    try {
      const body = (await response.json()) as ApiErrorResponse;
      if (body.error) {
        message = body.error;
      }
    } catch {
      // Ignore JSON parse errors and keep the default status message.
    }

    throw new Error(message);
  }

  return (await response.json()) as T;
}

export async function getLandingData() {
  const [status, latest] = await Promise.all([
    fetchJson<StatusResponse>("/api/status"),
    fetchJson<LatestRootResponse>("/api/roots/latest"),
  ]);

  return {
    status,
    snapshot: latest.snapshot,
  };
}

export async function getDashboardData() {
  const [status, latest, roots, chains] = await Promise.all([
    fetchJson<StatusResponse>("/api/status"),
    fetchJson<LatestRootResponse>("/api/roots/latest"),
    fetchJson<RootsResponse>("/api/roots"),
    fetchJson<ChainsResponse>("/api/chains"),
  ]);

  return {
    status,
    snapshot: latest.snapshot,
    roots: roots.roots,
    chains: chains.chains,
  };
}
