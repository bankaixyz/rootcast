CREATE TABLE observed_roots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_hex TEXT NOT NULL,
    source_block_number INTEGER NOT NULL,
    source_tx_hash TEXT NOT NULL,
    observed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    bankai_finalized_at TEXT,
    status TEXT NOT NULL DEFAULT 'detected',
    UNIQUE(root_hex, source_block_number)
);

CREATE TABLE replication_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    observed_root_id INTEGER NOT NULL,
    state TEXT NOT NULL,
    proof_artifact_ref TEXT,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(observed_root_id) REFERENCES observed_roots(id) ON DELETE CASCADE
);

CREATE TABLE chain_submissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    replication_job_id INTEGER NOT NULL,
    chain_name TEXT NOT NULL,
    chain_id TEXT NOT NULL,
    registry_address TEXT NOT NULL,
    state TEXT NOT NULL,
    tx_hash TEXT,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(replication_job_id) REFERENCES replication_jobs(id) ON DELETE CASCADE,
    UNIQUE(replication_job_id, chain_name)
);
