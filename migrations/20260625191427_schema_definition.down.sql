-- Add down migration script here
DROP INDEX IF EXISTS idx_item_categories_category;
DROP INDEX IF EXISTS idx_item_tags_tag;
DROP INDEX IF EXISTS idx_item_authors_author;
DROP TABLE IF EXISTS item_categories;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS item_tags;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS item_authors;
DROP TRIGGER IF EXISTS trg_authors_updated_at;
DROP TABLE IF EXISTS authors;
DROP TRIGGER IF EXISTS trg_items_updated_at;
DROP TABLE IF EXISTS items;
