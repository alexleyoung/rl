CREATE TABLE IF NOT EXISTS embeddings (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source_kind TEXT    NOT NULL,
    source_id   INTEGER NOT NULL,
    chunk_idx   INTEGER NOT NULL,
    text        TEXT    NOT NULL,
    dim         INTEGER NOT NULL,
    vector      BLOB    NOT NULL,
    created_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS embeddings_src_idx ON embeddings(source_kind, source_id);
