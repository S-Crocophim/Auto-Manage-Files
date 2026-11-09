PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rules (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
  priority INTEGER NOT NULL,
  destination TEXT NOT NULL,
  conflict_policy TEXT NOT NULL CHECK (conflict_policy IN ('autoRename', 'skip', 'safeOverwrite')),
  recursive INTEGER NOT NULL CHECK (recursive IN (0, 1)),
  notifications INTEGER NOT NULL CHECK (notifications IN (0, 1)),
  min_size_bytes INTEGER,
  max_size_bytes INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS rule_sources (
  id TEXT PRIMARY KEY,
  rule_id TEXT NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  canonical_path TEXT NOT NULL,
  available INTEGER NOT NULL CHECK (available IN (0, 1)),
  UNIQUE(rule_id, canonical_path)
);

CREATE TABLE IF NOT EXISTS rule_extensions (
  rule_id TEXT NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
  value TEXT NOT NULL,
  PRIMARY KEY(rule_id, value)
);

CREATE TABLE IF NOT EXISTS rule_globs (
  rule_id TEXT NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
  value TEXT NOT NULL,
  PRIMARY KEY(rule_id, value)
);

CREATE TABLE IF NOT EXISTS operation_batches (
  id TEXT PRIMARY KEY,
  trigger TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  completed_at TEXT,
  total_count INTEGER NOT NULL DEFAULT 0,
  completed_count INTEGER NOT NULL DEFAULT 0,
  failed_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS operation_items (
  id TEXT PRIMARY KEY,
  batch_id TEXT NOT NULL REFERENCES operation_batches(id) ON DELETE CASCADE,
  rule_id TEXT REFERENCES rules(id) ON DELETE SET NULL,
  rule_name TEXT,
  trigger TEXT NOT NULL,
  source_path TEXT NOT NULL,
  destination_path TEXT NOT NULL,
  recovery_path TEXT,
  staging_path TEXT,
  conflict_policy TEXT NOT NULL,
  source_size INTEGER NOT NULL,
  source_mtime INTEGER NOT NULL,
  source_hash TEXT,
  destination_hash TEXT,
  idempotency_key TEXT NOT NULL,
  status TEXT NOT NULL,
  error_code TEXT,
  error_message TEXT,
  detected_at TEXT NOT NULL,
  started_at TEXT,
  completed_at TEXT,
  undone_at TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  theme TEXT NOT NULL,
  language TEXT NOT NULL,
  minimize_to_tray_on_close INTEGER NOT NULL,
  auto_start INTEGER NOT NULL,
  notifications_enabled INTEGER NOT NULL,
  monitoring_enabled INTEGER NOT NULL,
  onboarding_complete INTEGER NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rules_priority ON rules(enabled, priority, created_at);
CREATE INDEX IF NOT EXISTS idx_sources_canonical ON rule_sources(canonical_path);
CREATE INDEX IF NOT EXISTS idx_activity ON operation_items(detected_at DESC, id DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_operation_idempotency
ON operation_items(idempotency_key)
WHERE status IN ('pending', 'staging', 'staged', 'destinationCommitted', 'completed', 'undoPending');

INSERT OR IGNORE INTO settings (
  id, theme, language, minimize_to_tray_on_close, auto_start,
  notifications_enabled, monitoring_enabled, onboarding_complete, updated_at
) VALUES (1, 'system', 'id', 1, 0, 1, 0, 0, CURRENT_TIMESTAMP);

INSERT OR IGNORE INTO schema_migrations(version, applied_at)
VALUES (1, CURRENT_TIMESTAMP);
