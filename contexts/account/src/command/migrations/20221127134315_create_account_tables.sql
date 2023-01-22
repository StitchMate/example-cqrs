CREATE TABLE account_events(
    aggregate_type TEXT,
    aggregate_id TEXT,
    sequence TEXT,
    event_type TEXT,
    event_version TEXT,
    payload JSON,
    metadata JSON,
    timestamp DATETIME
);

CREATE TABLE account_snapshots(
    aggregate_type TEXT,
    aggregate_id TEXT,
    payload JSON,
    last_sequence TEXT,
    snapshot_id TEXT,
    timestamp DATETIME
);

CREATE TABLE account_outbox_events(
    aggregate_type TEXT,
    aggregate_id TEXT,
    sequence TEXT,
    event_type TEXT,
    event_version TEXT,
    payload JSON,
    metadata JSON,
    timestamp DATETIME
);