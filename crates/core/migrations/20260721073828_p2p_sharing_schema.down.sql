-- Add down migration script here
DROP TABLE IF EXISTS library_subscriptions;
DROP TABLE IF EXISTS shared_libraries;
ALTER TABLE items DROP COLUMN shared_paper_id;
