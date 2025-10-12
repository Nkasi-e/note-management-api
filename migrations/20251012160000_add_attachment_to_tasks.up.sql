-- Add attachment_id column to tasks table
ALTER TABLE tasks 
ADD COLUMN attachment_id UUID REFERENCES files(id) ON DELETE SET NULL;

-- Create index on attachment_id for faster lookups
CREATE INDEX idx_tasks_attachment_id ON tasks(attachment_id);

