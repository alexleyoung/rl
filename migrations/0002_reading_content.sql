CREATE TABLE IF NOT EXISTS reading_content (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id  INTEGER NOT NULL UNIQUE REFERENCES resources(id) ON DELETE CASCADE,
    content_html TEXT NOT NULL DEFAULT '',
    content_text TEXT NOT NULL DEFAULT '',
    source_type  TEXT NOT NULL CHECK (source_type IN ('url','pdf')),
    word_count   INTEGER NOT NULL DEFAULT 0,
    status       TEXT NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending','ok','failed')),
    extracted_at INTEGER NOT NULL DEFAULT (unixepoch())
);
