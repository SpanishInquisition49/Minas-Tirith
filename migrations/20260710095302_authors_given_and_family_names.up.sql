-- Add up migration script here
ALTER TABLE authors ADD COLUMN given_name TEXT;
ALTER TABLE authors ADD COLUMN family_name TEXT;
