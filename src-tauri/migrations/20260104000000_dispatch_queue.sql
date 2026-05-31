ALTER TABLE tasks ADD COLUMN queued_at TEXT;
CREATE INDEX IF NOT EXISTS idx_tasks_queued ON tasks(queued_at);
