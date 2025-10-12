-- Create files table for tracking uploaded files
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename VARCHAR(255) NOT NULL UNIQUE,
    original_filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    size BIGINT NOT NULL,
    path TEXT NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on uploaded_by for faster queries
CREATE INDEX idx_files_uploaded_by ON files(uploaded_by);

-- Create index on created_at for sorting
CREATE INDEX idx_files_created_at ON files(created_at DESC);

-- Create index on filename for faster lookups
CREATE INDEX idx_files_filename ON files(filename);
