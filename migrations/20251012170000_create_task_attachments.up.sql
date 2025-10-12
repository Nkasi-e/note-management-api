-- Create task_attachments junction table for many-to-many relationship
CREATE TABLE IF NOT EXISTS task_attachments (
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (task_id, file_id)
);

-- Create indexes for efficient queries
CREATE INDEX idx_task_attachments_task_id ON task_attachments(task_id);
CREATE INDEX idx_task_attachments_file_id ON task_attachments(file_id);

-- Migrate existing attachment_id data to the new table
INSERT INTO task_attachments (task_id, file_id, created_at)
SELECT id, attachment_id, created_at
FROM tasks
WHERE attachment_id IS NOT NULL;

-- Remove the old attachment_id column (optional - keep it for backward compatibility if needed)
-- ALTER TABLE tasks DROP COLUMN attachment_id;

