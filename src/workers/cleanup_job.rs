use sqlx::PgPool;
use tracing::{info, error};
use chrono::{Utc, Duration};

/// Cleanup orphaned files that were uploaded but never attached to any task
pub async fn cleanup_orphaned_files(pool: &PgPool) -> Result<usize, sqlx::Error> {
    info!("Starting orphaned files cleanup job");
    
    // Find files uploaded more than 24 hours ago with no task association
    let cutoff_time = Utc::now() - Duration::hours(24);
    
    let orphaned_files = sqlx::query!(
        r#"
        SELECT f.id, f.path, f.filename
        FROM files f
        LEFT JOIN task_attachments ta ON f.id = ta.file_id
        WHERE ta.file_id IS NULL
        AND f.created_at < $1
        "#,
        cutoff_time
    )
    .fetch_all(pool)
    .await?;
    
    let count = orphaned_files.len();
    
    if count == 0 {
        info!("No orphaned files found");
        return Ok(0);
    }
    
    info!("Found {} orphaned file(s) to clean up", count);
    
    // Delete files from disk and database
    for file in orphaned_files {
        // Delete from disk
        if let Err(e) = tokio::fs::remove_file(&file.path).await {
            error!("Failed to delete file from disk: {} - {}", file.path, e);
        } else {
            info!("Deleted file from disk: {}", file.filename);
        }
        
        // Delete from database
        if let Err(e) = sqlx::query!(
            "DELETE FROM files WHERE id = $1",
            file.id
        )
        .execute(pool)
        .await {
            error!("Failed to delete file from database: {} - {}", file.id, e);
        } else {
            info!("Deleted file from database: {}", file.id);
        }
    }
    
    info!("Cleanup job completed. Removed {} orphaned file(s)", count);
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cleanup_orphaned_files() {
        // This would require a test database setup
        // For now, this is a placeholder for future testing
    }
}


