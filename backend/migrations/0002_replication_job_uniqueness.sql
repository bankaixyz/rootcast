DELETE FROM replication_jobs
WHERE id IN (
    SELECT id
    FROM (
        SELECT
            id,
            ROW_NUMBER() OVER (
                PARTITION BY observed_root_id
                ORDER BY id
            ) AS row_number
        FROM replication_jobs
    )
    WHERE row_number > 1
);

CREATE UNIQUE INDEX idx_replication_jobs_observed_root_id
ON replication_jobs(observed_root_id);
