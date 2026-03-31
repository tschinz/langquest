//! Render cache for parsed markdown content.
//!
//! This module provides caching of parsed markdown to avoid expensive
//! re-parsing and syntax highlighting on every frame. The cache is
//! invalidated when the exercise, page, terminal width, or display
//! settings change.

use std::collections::HashMap;

use ratatui::text::Line;

use super::markdown::{CodeBlockOptions, LinkSpan};

/// Cache key identifying a specific rendered content.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Exercise relative path (e.g., "01-basics/01-hello").
    pub exercise_path: String,
    /// Content type being rendered.
    pub content_type: ContentType,
    /// Terminal width at render time.
    pub width: u16,
    /// Whether line numbers are enabled.
    pub line_numbers: bool,
    /// Whether syntax highlighting is enabled.
    pub syntax_highlighting: bool,
}

/// Type of content being cached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// Theory page markdown.
    Theory,
    /// Task page markdown.
    Task,
    /// Solution page (source + explanation).
    Solution,
    /// About page (static content).
    About,
}

/// Cached render result containing parsed lines and link positions.
#[derive(Debug, Clone)]
pub struct CachedContent {
    /// Parsed ratatui lines ready for rendering.
    pub lines: Vec<Line<'static>>,
    /// Link positions for OSC 8 hyperlinks.
    pub links: Vec<LinkSpan>,
    /// Raw content hash to detect file changes.
    content_hash: u64,
}

impl CachedContent {
    /// Create a new cached content entry.
    pub fn new(lines: Vec<Line<'static>>, links: Vec<LinkSpan>, content: &str) -> Self {
        Self {
            lines,
            links,
            content_hash: Self::hash_content(content),
        }
    }

    /// Check if the cached content matches the given raw content.
    pub fn matches_content(&self, content: &str) -> bool {
        self.content_hash == Self::hash_content(content)
    }

    /// Simple FNV-1a hash for content comparison.
    fn hash_content(content: &str) -> u64 {
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;

        let mut hash = FNV_OFFSET;
        for byte in content.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }
}

/// Render cache storing parsed markdown content.
///
/// The cache uses a simple strategy:
/// - Store a limited number of entries (LRU-style eviction)
/// - Invalidate on width/settings changes
/// - Check content hash to detect file modifications
#[derive(Debug, Default)]
pub struct RenderCache {
    /// Cached entries keyed by cache key.
    entries: HashMap<CacheKey, CachedContent>,
    /// Maximum number of entries to keep.
    max_entries: usize,
    /// Keys in insertion order for LRU eviction.
    insertion_order: Vec<CacheKey>,
}

impl RenderCache {
    /// Create a new render cache with default capacity.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: 16, // Enough for a few exercises worth of pages
            insertion_order: Vec::new(),
        }
    }

    /// Build a cache key from the current render context.
    pub fn make_key(
        exercise_path: &str,
        content_type: ContentType,
        width: u16,
        opts: CodeBlockOptions,
    ) -> CacheKey {
        CacheKey {
            exercise_path: exercise_path.to_string(),
            content_type,
            width,
            line_numbers: opts.line_numbers,
            syntax_highlighting: opts.syntax_highlighting,
        }
    }

    /// Get cached content if available and still valid.
    ///
    /// Returns `None` if:
    /// - No cache entry exists for this key
    /// - The content has changed (hash mismatch)
    pub fn get(&self, key: &CacheKey, content: &str) -> Option<&CachedContent> {
        self.entries.get(key).filter(|cached| cached.matches_content(content))
    }

    /// Store content in the cache.
    ///
    /// If the cache is full, the oldest entry is evicted.
    pub fn insert(&mut self, key: CacheKey, content: CachedContent) {
        // If key already exists, just update it
        if self.entries.contains_key(&key) {
            self.entries.insert(key, content);
            return;
        }

        // Evict oldest if at capacity
        if self.entries.len() >= self.max_entries {
            if let Some(oldest_key) = self.insertion_order.first().cloned() {
                self.entries.remove(&oldest_key);
                self.insertion_order.remove(0);
            }
        }

        // Insert new entry
        self.insertion_order.push(key.clone());
        self.entries.insert(key, content);
    }

    /// Clear all cached entries.
    ///
    /// Call this when display settings change globally.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.insertion_order.clear();
    }

    /// Invalidate cache entries for a specific exercise.
    ///
    /// Call this when an exercise's files might have changed.
    pub fn invalidate_exercise(&mut self, exercise_path: &str) {
        self.insertion_order.retain(|key| {
            if key.exercise_path == exercise_path {
                self.entries.remove(key);
                false
            } else {
                true
            }
        });
    }

    /// Get the number of cached entries.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_key(path: &str, content_type: ContentType) -> CacheKey {
        CacheKey {
            exercise_path: path.to_string(),
            content_type,
            width: 80,
            line_numbers: true,
            syntax_highlighting: true,
        }
    }

    #[test]
    fn cache_hit_with_same_content() {
        let mut cache = RenderCache::new();
        let key = make_test_key("01/01", ContentType::Theory);
        let content = "# Hello World";

        let cached = CachedContent::new(vec![], vec![], content);
        cache.insert(key.clone(), cached);

        assert!(cache.get(&key, content).is_some());
    }

    #[test]
    fn cache_miss_with_different_content() {
        let mut cache = RenderCache::new();
        let key = make_test_key("01/01", ContentType::Theory);
        let content1 = "# Hello World";
        let content2 = "# Changed Content";

        let cached = CachedContent::new(vec![], vec![], content1);
        cache.insert(key.clone(), cached);

        // Should miss because content changed
        assert!(cache.get(&key, content2).is_none());
    }

    #[test]
    fn cache_miss_with_different_width() {
        let mut cache = RenderCache::new();
        let key1 = CacheKey {
            exercise_path: "01/01".to_string(),
            content_type: ContentType::Theory,
            width: 80,
            line_numbers: true,
            syntax_highlighting: true,
        };
        let key2 = CacheKey {
            width: 100,
            ..key1.clone()
        };
        let content = "# Hello";

        let cached = CachedContent::new(vec![], vec![], content);
        cache.insert(key1, cached);

        // Different width = different key = cache miss
        assert!(cache.get(&key2, content).is_none());
    }

    #[test]
    fn invalidate_exercise_removes_all_pages() {
        let mut cache = RenderCache::new();
        let content = "content";

        // Add multiple pages for same exercise
        for ct in [ContentType::Theory, ContentType::Task, ContentType::Solution] {
            let key = make_test_key("01/01", ct);
            cache.insert(key, CachedContent::new(vec![], vec![], content));
        }

        // Add page for different exercise
        let other_key = make_test_key("01/02", ContentType::Theory);
        cache.insert(other_key.clone(), CachedContent::new(vec![], vec![], content));

        assert_eq!(cache.len(), 4);

        cache.invalidate_exercise("01/01");

        assert_eq!(cache.len(), 1);
        assert!(cache.get(&other_key, content).is_some());
    }

    #[test]
    fn lru_eviction_when_full() {
        let mut cache = RenderCache::new();
        cache.max_entries = 3;

        let content = "content";

        // Fill cache
        for i in 0..3 {
            let key = make_test_key(&format!("ex{i}"), ContentType::Theory);
            cache.insert(key, CachedContent::new(vec![], vec![], content));
        }

        assert_eq!(cache.len(), 3);

        // Add one more - should evict oldest (ex0)
        let new_key = make_test_key("ex3", ContentType::Theory);
        cache.insert(new_key.clone(), CachedContent::new(vec![], vec![], content));

        assert_eq!(cache.len(), 3);

        // ex0 should be evicted
        let evicted_key = make_test_key("ex0", ContentType::Theory);
        assert!(cache.get(&evicted_key, content).is_none());

        // ex3 should be present
        assert!(cache.get(&new_key, content).is_some());
    }

    #[test]
    fn clear_removes_all_entries() {
        let mut cache = RenderCache::new();
        let content = "content";

        for i in 0..5 {
            let key = make_test_key(&format!("ex{i}"), ContentType::Theory);
            cache.insert(key, CachedContent::new(vec![], vec![], content));
        }

        assert_eq!(cache.len(), 5);

        cache.clear();

        assert_eq!(cache.len(), 0);
    }
}