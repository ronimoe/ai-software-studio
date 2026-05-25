CREATE TABLE IF NOT EXISTS verification_runs (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL DEFAULT (datetime('now')),
  finished_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_verification_runs_task ON verification_runs(task_id);

CREATE TABLE IF NOT EXISTS verification_checks (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL REFERENCES verification_runs(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  duration_ms INTEGER,
  log_excerpt TEXT,
  position INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_verification_checks_run ON verification_checks(run_id);

CREATE TABLE IF NOT EXISTS app_settings (
  scope TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  PRIMARY KEY (scope, key)
);
