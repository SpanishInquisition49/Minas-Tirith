-- Add down migration script here
ALTER TABLE authors DROP COLUMN given_name TEXT;
ALTER TABLE authors DROP COLUMN family_name TEXT;
