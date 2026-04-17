ALTER TABLE resources ADD COLUMN status TEXT NOT NULL DEFAULT 'inbox'
    CHECK (status IN ('inbox','reading','queue','done'));

CREATE INDEX IF NOT EXISTS idx_resources_status ON resources(status);
