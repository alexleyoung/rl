PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS resources (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    kind      TEXT NOT NULL CHECK (kind IN ('book','paper','article','blog','repo')),
    title     TEXT NOT NULL,
    author    TEXT,
    url       TEXT,
    file_path TEXT,
    added_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE IF NOT EXISTS resource_tags (
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (resource_id, tag_id)
);

CREATE TABLE IF NOT EXISTS notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    title       TEXT NOT NULL DEFAULT 'Untitled',
    body_md     TEXT NOT NULL DEFAULT '',
    body_html   TEXT NOT NULL DEFAULT '',
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

-- FTS5 over resources + notes
CREATE VIRTUAL TABLE IF NOT EXISTS search_fts USING fts5(
    source_kind,
    source_id UNINDEXED,
    title,
    body,
    tokenize='porter unicode61'
);

-- Keep FTS in sync with resources
CREATE TRIGGER IF NOT EXISTS resources_ai AFTER INSERT ON resources BEGIN
    INSERT INTO search_fts(source_kind, source_id, title, body)
    VALUES ('resource', new.id, new.title, COALESCE(new.author,''));
END;

CREATE TRIGGER IF NOT EXISTS resources_au AFTER UPDATE ON resources BEGIN
    DELETE FROM search_fts WHERE source_kind='resource' AND source_id=old.id;
    INSERT INTO search_fts(source_kind, source_id, title, body)
    VALUES ('resource', new.id, new.title, COALESCE(new.author,''));
END;

CREATE TRIGGER IF NOT EXISTS resources_ad AFTER DELETE ON resources BEGIN
    DELETE FROM search_fts WHERE source_kind='resource' AND source_id=old.id;
END;

-- Keep FTS in sync with notes
CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO search_fts(source_kind, source_id, title, body)
    VALUES ('note', new.id, new.title, new.body_md);
END;

CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
    DELETE FROM search_fts WHERE source_kind='note' AND source_id=old.id;
    INSERT INTO search_fts(source_kind, source_id, title, body)
    VALUES ('note', new.id, new.title, new.body_md);
END;

CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
    DELETE FROM search_fts WHERE source_kind='note' AND source_id=old.id;
END;
