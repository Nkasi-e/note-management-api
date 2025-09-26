-- Remove unique constraint from slug column
ALTER TABLE tasks DROP CONSTRAINT tasks_slug_unique;