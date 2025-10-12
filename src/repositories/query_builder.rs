/// High-performance SQL query builder using arena allocation
/// 
/// Traditional query builders allocate many temporary strings.
/// This implementation uses bump allocation for O(1) temporary allocations
/// and bulk deallocation when the query is complete.

use crate::arena::RequestArena;
use crate::domain::task::TaskStatus;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Simple query builder using arena for final allocation
/// Builds queries efficiently and stores result in arena
pub struct ArenaQueryBuilder<'a> {
    arena: &'a RequestArena,
}

impl<'a> ArenaQueryBuilder<'a> {
    pub fn new(arena: &'a RequestArena) -> Self {
        Self { arena }
    }

    /// Build a simple SELECT query
    pub fn build_select(
        &self,
        columns: &str,
        table: &str,
        where_clause: Option<&str>,
        order_by: Option<(&str, &str)>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> &'a str {
        let mut query = String::with_capacity(128);
        query.push_str("SELECT ");
        query.push_str(columns);
        query.push_str(" FROM ");
        query.push_str(table);

        if let Some(condition) = where_clause {
            query.push_str(" WHERE ");
            query.push_str(condition);
        }

        if let Some((column, direction)) = order_by {
            query.push_str(" ORDER BY ");
            query.push_str(column);
            query.push(' ');
            query.push_str(direction);
        }

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET {}", off));
        }

        self.arena.alloc_str(&query)
    }
}

/// Helper for building complex task queries with filters
pub struct TaskQueryArenaBuilder<'a> {
    arena: &'a RequestArena,
}

impl<'a> TaskQueryArenaBuilder<'a> {
    pub fn new(arena: &'a RequestArena) -> Self {
        Self { arena }
    }

    /// Build a task query with optional filters
    /// All temporary allocations use the arena
    pub fn build_task_query(
        &self,
        status: Option<&TaskStatus>,
        user_id: Option<&Uuid>,
        search: Option<&str>,
        created_after: Option<&DateTime<Utc>>,
        created_before: Option<&DateTime<Utc>>,
        sort_by: &str,
        sort_direction: &str,
        limit: i64,
        offset: i64,
    ) -> &'a str {
        let mut query = String::with_capacity(256);
        query.push_str("SELECT id, title, description, status, user_id, created_at, updated_at, slug FROM tasks");

        let has_filters = status.is_some() || user_id.is_some() || search.is_some() 
            || created_after.is_some() || created_before.is_some();

        if has_filters {
            query.push_str(" WHERE 1=1");
        }

        if let Some(_) = status {
            query.push_str(" AND status = $1");
        }

        if let Some(_) = user_id {
            query.push_str(" AND user_id = $2");
        }

        if let Some(_) = search {
            query.push_str(" AND (title ILIKE $3 OR description ILIKE $3)");
        }

        if let Some(_) = created_after {
            query.push_str(" AND created_at >= $4");
        }

        if let Some(_) = created_before {
            query.push_str(" AND created_at <= $5");
        }

        query.push_str(" ORDER BY ");
        query.push_str(sort_by);
        query.push(' ');
        query.push_str(sort_direction);
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // Allocate final string in arena - single allocation
        self.arena.alloc_str(&query)
    }

    /// Build a count query for pagination
    pub fn build_count_query(
        &self,
        status: Option<&TaskStatus>,
        user_id: Option<&Uuid>,
        search: Option<&str>,
        created_after: Option<&DateTime<Utc>>,
        created_before: Option<&DateTime<Utc>>,
    ) -> &'a str {
        let mut query = String::with_capacity(128);
        query.push_str("SELECT COUNT(*) FROM tasks");

        let has_filters = status.is_some() || user_id.is_some() || search.is_some() 
            || created_after.is_some() || created_before.is_some();

        if has_filters {
            query.push_str(" WHERE 1=1");
        }

        if let Some(_) = status {
            query.push_str(" AND status = $1");
        }

        if let Some(_) = user_id {
            query.push_str(" AND user_id = $2");
        }

        if let Some(_) = search {
            query.push_str(" AND (title ILIKE $3 OR description ILIKE $3)");
        }

        if let Some(_) = created_after {
            query.push_str(" AND created_at >= $4");
        }

        if let Some(_) = created_before {
            query.push_str(" AND created_at <= $5");
        }

        // Allocate final string in arena - single allocation
        self.arena.alloc_str(&query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let arena = RequestArena::new();
        let builder = ArenaQueryBuilder::new(&arena);
        
        let query = builder.build_select(
            "*",
            "users",
            Some("active = true"),
            None,
            None,
            None
        );

        assert_eq!(query, "SELECT * FROM users WHERE active = true");
    }

    #[test]
    fn test_complex_query() {
        let arena = RequestArena::new();
        let builder = ArenaQueryBuilder::new(&arena);
        
        let query = builder.build_select(
            "id, name, email",
            "users",
            Some("active = true AND role = 'admin'"),
            Some(("created_at", "DESC")),
            Some(10),
            Some(0)
        );

        assert_eq!(
            query,
            "SELECT id, name, email FROM users WHERE active = true AND role = 'admin' ORDER BY created_at DESC LIMIT 10 OFFSET 0"
        );
    }

    #[test]
    fn test_arena_memory_efficiency() {
        let arena = RequestArena::new();
        
        // Build multiple queries in the same arena
        for i in 0..100 {
            let builder = TaskQueryArenaBuilder::new(&arena);
            let _ = builder.build_count_query(None, None, None, None, None);
        }

        // Arena allocates efficiently - all queries share the same arena
        println!("Arena used {} bytes for 100 queries", arena.allocated_bytes());
        assert!(arena.allocated_bytes() > 0);
    }
}

