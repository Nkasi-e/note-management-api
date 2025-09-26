-- Add unique constraint to slug column
ALTER TABLE tasks ADD CONSTRAINT tasks_slug_unique UNIQUE (slug);