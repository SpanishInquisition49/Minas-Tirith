-- Add down migration script here
DROP INDEX IF EXISTS idx_item_collections_collection;
DROP TABLE IF EXISTS item_collections;
DROP TRIGGER IF EXISTS trg_collections_updated_at;
DROP TABLE IF EXISTS collections;
CREATE TABLE IF NOT EXISTS categories(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  slug TEXT UNIQUE
);
CREATE TABLE IF NOT EXISTS item_categories(
  item_id INTEGER NOT NULL,
  category_id INTEGER NOT NULL PRIMARY KEY(
    item_id,
    category_id
  ),
  FOREIGN KEY(item_id) REFERENCES items(id)
ON DELETE CASCADE FOREIGN KEY(category_id) REFERENCES categories(id)
ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_item_categories_category
ON item_categories(category_id);
