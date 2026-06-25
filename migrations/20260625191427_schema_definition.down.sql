-- Add down migration script here
DELETE INDEX IF EXISTS idx_item_categories_category;
DELETE INDEX IF EXISTS idx_item_tags_tag;
DELETE INDEX IF EXISTS idx_item_authors_author;
DELETE TABLE IF EXISTS item_categories;
DELETE TABLE IF EXISTS categories;
DELETE TABLE IF EXISTS item_tags;
DELETE TABLE IF EXISTS tags;
DELETE TABLE IF EXISTS item_authors;
DELETE TRIGGER IF EXISTS trg_authors_updated_at;
DELETE TABLE IF EXISTS authors;
DELETE TRIGGER IF EXISTS trg_items_updated_at;
DELETE TABLE IF EXISTS items;
