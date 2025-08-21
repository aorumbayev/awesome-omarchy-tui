use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FocusArea {
    Sidebar,
    #[default]
    Content,
}

#[derive(Debug, Clone)]
pub enum AppState {
    Loading,
    Ready,
    Error(String),
}

impl AppState {
    pub fn is_loading(&self) -> bool {
        matches!(self, AppState::Loading)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadmeContent {
    pub sections: Vec<Section>,
    pub search_index: SearchIndex,
    pub metadata: ReadmeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeMetadata {
    pub title: String,
    pub description: String,
    pub last_updated: Option<String>,
    pub total_entries: usize,
}

impl Default for ReadmeMetadata {
    fn default() -> Self {
        Self {
            title: "Awesome Omarchy".to_string(),
            description: "A curated list of awesome resources".to_string(),
            last_updated: None,
            total_entries: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryEntry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub entries: Vec<RepositoryEntry>,
    pub raw_content: String,
    pub entry_count: usize,
}

impl Section {
    pub fn new(title: String) -> Self {
        Self {
            title,
            entries: Vec::new(),
            raw_content: String::new(),
            entry_count: 0,
        }
    }
}

#[cfg(feature = "aur-theme-preview")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeEntry {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchIndex {
    pub terms: HashMap<String, Vec<SearchLocation>>,
    pub total_terms: usize,
}

impl SearchIndex {
    pub fn add_term(&mut self, term: String, location: SearchLocation) {
        self.terms
            .entry(term.to_lowercase())
            .or_default()
            .push(location);
        self.total_terms += 1;
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut entry_results: HashMap<(usize, Option<usize>), (SearchResult, f64)> =
            HashMap::new();

        self.terms
            .iter()
            .filter(|(term, _)| term.contains(&query_lower))
            .flat_map(|(term, locations)| {
                locations.iter().filter_map(|location| {
                    if location.search_priority == SearchPriority::RawContent {
                        return None;
                    }

                    let entry_key = (location.section_index, location.entry_index);
                    let relevance_score = calculate_relevance_score(term, &query_lower)
                        * location.search_priority.score_multiplier();

                    Some((entry_key, location, relevance_score))
                })
            })
            .for_each(|(entry_key, location, relevance_score)| {
                entry_results
                    .entry(entry_key)
                    .and_modify(|(existing_result, existing_max_score)| {
                        // Prefer repository names for display
                        if location.search_priority == SearchPriority::RepositoryName {
                            existing_result.line_content = location.line_content.clone();
                        }
                        if relevance_score > *existing_max_score {
                            *existing_max_score = relevance_score;
                            existing_result.relevance_score = relevance_score;
                        }
                    })
                    .or_insert_with(|| {
                        (
                            SearchResult {
                                section_index: location.section_index,
                                entry_index: location.entry_index,
                                line_content: location.line_content.clone(),
                                relevance_score,
                                github_url: location.github_url.clone(),
                            },
                            relevance_score,
                        )
                    });
            });

        let mut results: Vec<SearchResult> = entry_results
            .into_values()
            .map(|(result, _)| result)
            .collect();
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

fn calculate_relevance_score(term: &str, query: &str) -> f64 {
    if term == query {
        1.0 // Exact match
    } else if term.starts_with(query) {
        0.8 // Prefix match
    } else if term.ends_with(query) {
        0.6 // Suffix match
    } else {
        0.4 // Contains match
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchPriority {
    RepositoryName = 1,
    Description = 2,
    RawContent = 3,
}

impl SearchPriority {
    pub fn score_multiplier(&self) -> f64 {
        match self {
            SearchPriority::RepositoryName => 2.0, // Highest priority
            SearchPriority::Description => 1.5,    // Medium priority
            SearchPriority::RawContent => 0.1,     // Lowest priority (excluded)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchLocation {
    pub section_index: usize,
    pub entry_index: Option<usize>,
    pub line_content: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub search_priority: SearchPriority,
    pub github_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TabState {
    pub title: String,
    pub section_index: usize,
    pub scroll_offset: usize,
    pub selected: bool,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub struct ListState {
    pub selected_index: Option<usize>,
    pub offset: usize,
}

impl Default for ListState {
    fn default() -> Self {
        Self::new()
    }
}

impl ListState {
    pub fn new() -> Self {
        Self {
            selected_index: Some(0),
            offset: 0,
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected_index = index;
        // Ensure offset is reasonable
        if let Some(idx) = index
            && idx < self.offset
        {
            self.offset = idx;
        }
    }

    pub fn select_first(&mut self) {
        self.selected_index = Some(0);
        self.offset = 0;
    }

    pub fn select_previous(&mut self, max_items: usize) {
        if max_items == 0 {
            self.selected_index = None;
            return;
        }

        let i = match self.selected_index {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    max_items - 1
                }
            }
            None => 0,
        };
        self.selected_index = Some(i);

        // Update offset if needed
        if i < self.offset {
            self.offset = i;
        }
    }

    pub fn select_next(&mut self, max_items: usize) {
        if max_items == 0 {
            self.selected_index = None;
            return;
        }

        let i = match self.selected_index {
            Some(i) => {
                if i >= max_items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_index = Some(i);

        // Update offset if needed for viewport management
        // This is a simple implementation - can be enhanced based on viewport size
        if i >= self.offset + 10 {
            // Assume 10-item viewport
            self.offset = i.saturating_sub(9);
        } else if i < self.offset {
            self.offset = i;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub section_index: usize,
    pub entry_index: Option<usize>,
    pub line_content: String,
    pub relevance_score: f64,
    pub github_url: Option<String>,
}

/// Theme browser related models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub source_url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub normal: ThemeColorPalette,
    pub bright: ThemeColorPalette,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColorPalette {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

#[derive(Debug, Clone)]
pub struct ThemeBrowserState {
    pub themes: Vec<Theme>,
    pub selected_index: Option<usize>,
    pub loading: bool,
    pub error: Option<String>,
    pub preview_theme: Option<Theme>,
    pub search_mode: bool,
    pub search_query: String,
    pub filtered_themes: Vec<usize>, // Indices of themes matching search
    pub filtered_selected: Option<usize>, // Selected index in filtered results
}

/// Global theme applicator for in-memory theme switching
#[derive(Debug, Clone, Default)]
pub struct ThemeApplicator {
    pub current_theme: Option<Theme>,
    pub original_theme: Option<Theme>,
    pub is_applied: bool,
}

impl ThemeApplicator {
    pub fn apply_theme(&mut self, theme: Theme) {
        // Store original theme only on first application
        if self.original_theme.is_none() && !self.is_applied {
            self.original_theme = Some(Self::create_default_theme());
        }

        self.current_theme = Some(theme);
        self.is_applied = true;
    }

    pub fn clear_theme(&mut self) {
        *self = Self::default();
    }

    fn create_default_theme() -> Theme {
        Theme {
            name: "Default".to_string(),
            description: "Default TUI theme".to_string(),
            source_url: "builtin".to_string(),
            colors: ThemeColors {
                background: "#1a1b26".to_string(),
                foreground: "#a9b1d6".to_string(),
                normal: ThemeColorPalette {
                    black: "#32344a".to_string(),
                    red: "#f7768e".to_string(),
                    green: "#9ece6a".to_string(),
                    yellow: "#e0af68".to_string(),
                    blue: "#7aa2f7".to_string(),
                    magenta: "#ad8ee6".to_string(),
                    cyan: "#449dab".to_string(),
                    white: "#787c99".to_string(),
                },
                bright: ThemeColorPalette {
                    black: "#444b6a".to_string(),
                    red: "#ff7a93".to_string(),
                    green: "#b9f27c".to_string(),
                    yellow: "#ff9e64".to_string(),
                    blue: "#7da6ff".to_string(),
                    magenta: "#bb9af7".to_string(),
                    cyan: "#0db9d7".to_string(),
                    white: "#acb0d0".to_string(),
                },
            },
        }
    }
}

/// Simple theme entry extracted from README

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PreviewState {
    None,
    Loading,             // Theme loading in progress
    Applied(Box<Theme>), // Theme currently applied
    Error,               // Error loading theme
}

impl Default for PreviewState {
    fn default() -> Self {
        Self::None
    }
}
