-- Remove the old attachment_id column since we now use task_attachments junction table
ALTER TABLE tasks DROP COLUMN IF EXISTS attachment_id;

