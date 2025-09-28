use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::domain::task::TaskStatus;

/// Pagination parameters for task queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u32,
    
    /// Number of items per page
    #[serde(default = "default_limit")]
    pub limit: u32,
    
    /// Sort field
    #[serde(default = "default_sort_field")]
    pub sort_by: String,
    
    /// Sort direction (asc, desc)
    #[serde(default = "default_sort_direction")]
    pub sort_direction: String,
}

/// Task filtering parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilters {
    /// Filter by task status
    pub status: Option<TaskStatus>,
    
    /// Filter by user ID
    pub user_id: Option<Uuid>,
    
    /// Filter by date range - start date
    pub created_after: Option<DateTime<Utc>>,
    
    /// Filter by date range - end date
    pub created_before: Option<DateTime<Utc>>,
    
    /// Search in title and description
    pub search: Option<String>,
}

/// Combined query parameters for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    
    #[serde(flatten)]
    pub filters: TaskFilters,
}

/// Pagination metadata for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number
    pub page: u32,
    
    /// Number of items per page
    pub limit: u32,
    
    /// Total number of items
    pub total: u64,
    
    /// Total number of pages
    pub total_pages: u32,
    
    /// Whether there's a next page
    pub has_next: bool,
    
    /// Whether there's a previous page
    pub has_prev: bool,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The actual data
    pub data: Vec<T>,
    
    /// Pagination metadata
    pub pagination: PaginationMeta,
}

// Default values
fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }
fn default_sort_field() -> String { "created_at".to_string() }
fn default_sort_direction() -> String { "desc".to_string() }

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            limit: default_limit(),
            sort_by: default_sort_field(),
            sort_direction: default_sort_direction(),
        }
    }
}

impl Default for TaskFilters {
    fn default() -> Self {
        Self {
            status: None,
            user_id: None,
            created_after: None,
            created_before: None,
            search: None,
        }
    }
}

impl Default for TaskQueryParams {
    fn default() -> Self {
        Self {
            pagination: PaginationParams::default(),
            filters: TaskFilters::default(),
        }
    }
}

impl PaginationParams {
    /// Calculate offset for database queries
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.limit
    }
    
    /// Validate pagination parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.page == 0 {
            return Err("Page must be greater than 0".to_string());
        }
        
        if self.limit == 0 || self.limit > 100 {
            return Err("Limit must be between 1 and 100".to_string());
        }
        
        if !["created_at", "updated_at", "title", "status"].contains(&self.sort_by.as_str()) {
            return Err("Invalid sort field".to_string());
        }
        
        if !["asc", "desc"].contains(&self.sort_direction.as_str()) {
            return Err("Sort direction must be 'asc' or 'desc'".to_string());
        }
        
        Ok(())
    }
}

impl PaginationMeta {
    /// Create pagination metadata from query params and total count
    pub fn new(pagination: &PaginationParams, total: u64) -> Self {
        let total_pages = ((total as f64) / (pagination.limit as f64)).ceil() as u32;
        
        Self {
            page: pagination.page,
            limit: pagination.limit,
            total,
            total_pages,
            has_next: pagination.page < total_pages,
            has_prev: pagination.page > 1,
        }
    }
}
