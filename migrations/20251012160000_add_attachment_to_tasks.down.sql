-- Remove attachment_id column from tasks table
DROP INDEX IF EXISTS idx_tasks_attachment_id;
ALTER TABLE tasks DROP COLUMN IF EXISTS attachment_id;

