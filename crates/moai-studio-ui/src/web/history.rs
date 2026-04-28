// Navigation history management for SPEC-V3-007 MS-2
//
// This module provides navigation history tracking for WebViewSurface.
// Maintains a cursor-based history with max 100 entries.

/// Single navigation entry with URL and title
#[derive(Debug, Clone, PartialEq)]
pub struct NavigationEntry {
    /// Full URL of the page
    pub url: String,
    /// Page title (may be empty initially)
    pub title: String,
}

impl NavigationEntry {
    /// Create a new navigation entry
    pub fn new(url: String) -> Self {
        Self {
            url,
            title: String::new(),
        }
    }
}

/// Navigation history with cursor-based tracking
///
/// # Invariants
/// - Always has at least 1 entry (initial URL)
/// - cursor is always valid (0 <= cursor < entries.len())
/// - Max 100 entries (oldest removed when exceeded)
#[derive(Debug, Clone)]
pub struct NavigationHistory {
    /// History entries (max 100)
    entries: Vec<NavigationEntry>,
    /// Current position in history
    cursor: usize,
}

impl NavigationHistory {
    /// Create new history with initial entry
    ///
    /// # Arguments
    /// * `initial_url` - Starting URL (cursor will be at 0)
    pub fn new(initial_url: String) -> Self {
        Self {
            entries: vec![NavigationEntry::new(initial_url)],
            cursor: 0,
        }
    }

    /// Navigate to a new URL
    ///
    /// Truncates forward history, appends new entry, moves cursor forward.
    /// Enforces max 100 entries by removing oldest entries when needed.
    ///
    /// # Arguments
    /// * `url` - Target URL to navigate to
    pub fn navigate(&mut self, url: String) {
        // Truncate entries after cursor (clear forward history)
        self.entries.truncate(self.cursor + 1);

        // Add new entry
        self.entries.push(NavigationEntry::new(url));

        // Enforce max 100 entries
        if self.entries.len() > 100 {
            self.entries.remove(0);
            // Adjust cursor since we removed from front
            self.cursor = self.entries.len() - 1;
        } else {
            // Move cursor to new entry
            self.cursor = self.entries.len() - 1;
        }
    }

    /// Go back in history
    ///
    /// # Returns
    /// * `true` - Successfully moved back
    /// * `false` - Already at beginning of history
    pub fn go_back(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    /// Go forward in history
    ///
    /// # Returns
    /// * `true` - Successfully moved forward
    /// * `false` - Already at end of history
    pub fn go_forward(&mut self) -> bool {
        if self.cursor < self.entries.len().saturating_sub(1) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    /// Get current navigation entry
    ///
    /// # Panics
    /// Never (invariant: entries always has at least 1 element)
    pub fn current(&self) -> &NavigationEntry {
        // SAFETY: entries is never empty (invariant)
        self.entries.get(self.cursor).expect("NavigationHistory invariant violated: entries is empty")
    }

    /// Check if can go back
    pub fn can_go_back(&self) -> bool {
        self.cursor > 0
    }

    /// Check if can go forward
    pub fn can_go_forward(&self) -> bool {
        self.cursor < self.entries.len().saturating_sub(1)
    }

    /// Get total number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Get current cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_new_has_one_entry() {
        let history = NavigationHistory::new("https://example.com".to_string());
        assert_eq!(history.len(), 1);
        assert_eq!(history.cursor(), 0);
        assert_eq!(history.current().url, "https://example.com");
    }

    #[test]
    fn test_history_navigate_appends() {
        let mut history = NavigationHistory::new("https://example.com".to_string());
        history.navigate("https://modu.ai".to_string());

        assert_eq!(history.len(), 2);
        assert_eq!(history.cursor(), 1);
        assert_eq!(history.current().url, "https://modu.ai");
    }

    #[test]
    fn test_history_navigate_truncates_forward() {
        let mut history = NavigationHistory::new("https://example.com".to_string());
        history.navigate("https://modu.ai".to_string());
        history.navigate("https://github.com".to_string());
        history.go_back(); // cursor at 1 (modu.ai)
        history.go_back(); // cursor at 0 (example.com)

        // Navigate from middle - should truncate forward history
        history.navigate("https://google.com".to_string());

        assert_eq!(history.len(), 2); // example.com + google.com
        assert_eq!(history.cursor(), 1);
        assert_eq!(history.current().url, "https://google.com");
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_history_go_back_forward() {
        let mut history = NavigationHistory::new("https://example.com".to_string());
        history.navigate("https://modu.ai".to_string());
        history.navigate("https://github.com".to_string());

        // Go back twice
        assert!(history.go_back());
        assert_eq!(history.current().url, "https://modu.ai");
        assert!(history.go_back());
        assert_eq!(history.current().url, "https://example.com");

        // Can't go back further
        assert!(!history.go_back());
        assert_eq!(history.current().url, "https://example.com");

        // Go forward
        assert!(history.go_forward());
        assert_eq!(history.current().url, "https://modu.ai");
        assert!(history.go_forward());
        assert_eq!(history.current().url, "https://github.com");

        // Can't go forward further
        assert!(!history.go_forward());
        assert_eq!(history.current().url, "https://github.com");
    }

    #[test]
    fn test_history_max_100_entries() {
        let mut history = NavigationHistory::new("https://example.com".to_string());

        // Add 101 URLs
        for i in 0..=100 {
            history.navigate(format!("https://example.com/{}", i));
        }

        // Should be capped at 100
        assert_eq!(history.len(), 100);
        // First entry should be removed
        assert_ne!(history.entries[0].url, "https://example.com");
        assert_eq!(history.cursor(), 99);
    }

    #[test]
    fn test_history_cannot_go_back_at_start() {
        let mut history = NavigationHistory::new("https://example.com".to_string());
        assert!(!history.can_go_back());
        assert!(!history.go_back());
        assert_eq!(history.cursor(), 0);
    }

    #[test]
    fn test_history_cannot_go_forward_at_end() {
        let mut history = NavigationHistory::new("https://example.com".to_string());
        history.navigate("https://modu.ai".to_string());
        assert!(!history.can_go_forward());
        assert!(!history.go_forward());
        assert_eq!(history.cursor(), 1);
    }
}
