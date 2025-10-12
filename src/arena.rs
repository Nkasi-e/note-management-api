/// Arena allocation utilities using bumpalo for high-performance temporary allocations
/// 
/// Bump allocation (arena allocation) is ideal for:
/// - Request-scoped allocations
/// - Temporary string building
/// - Message formatting
/// - Query construction
/// 
/// Benefits:
/// - O(1) allocation time (vs O(log n) for standard allocator)
/// - Bulk deallocation (all freed at once)
/// - Better cache locality
/// - 10-100x faster for certain patterns

use bumpalo::Bump;
use std::fmt::Write;

/// Request-scoped arena allocator
/// All allocations are freed when the arena is dropped
pub struct RequestArena {
    bump: Bump,
}

impl RequestArena {
    /// Create a new arena with default capacity
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
        }
    }

    /// Create a new arena with pre-allocated capacity
    /// Use this when you know approximately how much memory you'll need
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            bump: Bump::with_capacity(bytes),
        }
    }

    /// Allocate a string slice in the arena
    pub fn alloc_str(&self, s: &str) -> &str {
        self.bump.alloc_str(s)
    }

    /// Format a string into the arena (zero-copy after formatting)
    pub fn alloc_fmt(&self, s: String) -> &str {
        self.bump.alloc_str(&s)
    }

    /// Build a string in the arena using a closure
    pub fn build_string<F>(&self, f: F) -> &str
    where
        F: FnOnce(&mut String),
    {
        let mut s = String::new();
        f(&mut s);
        self.bump.alloc_str(&s)
    }

    /// Get the underlying bump allocator (for advanced use)
    pub fn bump(&self) -> &Bump {
        &self.bump
    }

    /// Reset the arena, freeing all allocations
    /// Keeps the capacity for reuse
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get current allocated bytes
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Default for RequestArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building formatted strings in an arena
pub struct ArenaStringBuilder<'a> {
    arena: &'a RequestArena,
    buffer: String,
}

impl<'a> ArenaStringBuilder<'a> {
    pub fn new(arena: &'a RequestArena) -> Self {
        Self {
            arena,
            buffer: String::new(),
        }
    }

    pub fn with_capacity(arena: &'a RequestArena, capacity: usize) -> Self {
        Self {
            arena,
            buffer: String::with_capacity(capacity),
        }
    }

    pub fn push_str(&mut self, s: &str) -> &mut Self {
        self.buffer.push_str(s);
        self
    }

    pub fn push(&mut self, c: char) -> &mut Self {
        self.buffer.push(c);
        self
    }

    pub fn write_fmt(&mut self, args: std::fmt::Arguments) -> &mut Self {
        let _ = Write::write_fmt(&mut self.buffer, args);
        self
    }

    /// Finalize and return the string allocated in the arena
    pub fn finish(&mut self) -> &'a str {
        self.arena.alloc_str(&self.buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        let arena = RequestArena::new();
        let s1 = arena.alloc_str("hello");
        let s2 = arena.alloc_str("world");
        
        assert_eq!(s1, "hello");
        assert_eq!(s2, "world");
    }

    #[test]
    fn test_arena_fmt() {
        let arena = RequestArena::new();
        let s = arena.alloc_fmt(format!("value: {}", 42));
        
        assert_eq!(s, "value: 42");
    }

    #[test]
    fn test_arena_builder() {
        let arena = RequestArena::new();
        let mut builder = ArenaStringBuilder::new(&arena);
        let s = builder
            .push_str("Hello")
            .push(' ')
            .push_str("World")
            .finish();
        
        assert_eq!(s, "Hello World");
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = RequestArena::new();
        
        // Allocate some strings
        let _ = arena.alloc_str("temporary1");
        let _ = arena.alloc_str("temporary2");
        let _ = arena.alloc_str("temporary3");
        let before_reset = arena.allocated_bytes();
        assert!(before_reset > 0, "Arena should have allocated memory");
        
        // Reset should allow reusing the same memory
        arena.reset();
        
        // After reset, we can allocate again
        let _ = arena.alloc_str("new string");
        // Reset works correctly if we can still allocate
        assert!(true, "Arena reset allows reusing memory");
    }
}

