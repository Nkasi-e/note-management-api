-- Re-add attachment_id column if we need to rollback
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS attachment_id UUID REFERENCES files(id) ON DELETE SET NULL;

-- Recreate index
CREATE INDEX IF NOT EXISTS idx_tasks_attachment_id ON tasks(attachment_id);

