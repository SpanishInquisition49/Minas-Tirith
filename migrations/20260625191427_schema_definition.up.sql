-- Add up migration script here
-- NOTE: Items table
CREATE TABLE IF NOT EXISTS items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  description TEXT,
  path TEXT NOT NULL,
  type TEXT CHECK ( type IN ('book', 'article', 'misc', 'report', 'thesis') ),
  doi TEXT,
  isbn TEXT,
  publication_date TEXT,
  slug TEXT NOT NULL UNIQUE,
  cover_image_url TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS trg_items_updated_at
AFTER UPDATE ON items
BEGIN
  UPDATE items SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- NOTE: Authors tables
CREATE TABLE IF NOT EXISTS authors (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  slug TEXT UNIQUE,
  bio TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS trg_authors_updated_at
AFTER UPDATE ON authors
BEGIN
  UPDATE authors SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TABLE IF NOT EXISTS item_authors (
  item_id INTEGER NOT NULL,
  author_id INTEGER NOT NULL,
  author_order INTEGER DEFAULT 0,
  PRIMARY KEY (item_id, author_id),
  FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
  FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
);

-- NOTE: Tags tables
CREATE TABLE IF NOT EXISTS tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  slug TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS item_tags (
  item_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  PRIMARY KEY (item_id, tag_id),
  FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- NOTE: Categories tables
CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  slug TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS item_categories (
  item_id INTEGER NOT NULL,
  category_id INTEGER NOT NULL,
  PRIMARY KEY (item_id, category_id),
  FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- NOTEL Indici per lookup inverso (categoria/tag/autore -> items)
CREATE INDEX IF NOT EXISTS idx_item_authors_author ON item_authors(author_id);
CREATE INDEX IF NOT EXISTS idx_item_tags_tag ON item_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_item_categories_category ON item_categories(category_id);
