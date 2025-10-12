-- Drop task_attachments table
DROP INDEX IF EXISTS idx_task_attachments_file_id;
DROP INDEX IF EXISTS idx_task_attachments_task_id;
DROP TABLE IF EXISTS task_attachments;

