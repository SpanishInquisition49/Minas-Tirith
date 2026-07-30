-- Add up migration script here
DROP INDEX IF EXISTS idx_item_categories_category;
DROP TABLE IF EXISTS item_categories;
DROP TABLE IF EXISTS categories;

CREATE TABLE IF NOT EXISTS collections (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  slug TEXT UNIQUE,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS trg_collections_updated_at
AFTER UPDATE ON collections
BEGIN
  UPDATE collections SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TABLE IF NOT EXISTS item_collections (
  item_id INTEGER NOT NULL,
  collection_id INTEGER NOT NULL,
  PRIMARY KEY (item_id, collection_id),
  FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
  FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_item_collections_collection ON item_collections(collection_id);
