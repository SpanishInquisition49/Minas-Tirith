-- Add up migration script here
-- Add a stable unique id for an item used in shared namespaces
ALTER TABLE items ADD COLUMN shared_paper_id TEXT;
-- Local collections published by some peer as a Shared Library
CREATE TABLE IF NOT EXISTS shared_libraries(
id INTEGER PRIMARY KEY AUTOINCREMENT,
collection_id INTEGER NOT NULL,
namespace_id INTEGER NOT NULL UNIQUE,
name TEXT NOT NULL,
description TEXT,
mode TEXT NOT NULL CHECK(mode IN(
'personal',
'group'
)) DEFAULT 'personal',
created_at TEXT NOT NULL DEFAULT(datetime('now')),
FOREIGN KEY(collection_id) REFERENCES collections(id)
ON DELETE CASCADE
);
-- Libraries from other peers which we are following
CREATE TABLE IF NOT EXISTS library_subscriptions(
id INTEGER PRIMARY KEY AUTOINCREMENT,
namespace_id TEXT NOT NULL UNIQUE,
owner_node_id TEXT NOT NULL,
nickname TEXT NOT NULL,
last_synced_at TEXT,
created_at TEXT NOT NULL DEFAULT(datetime('now'))
);
