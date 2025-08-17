use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AppState {
    Loading,
    Ready,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeContent {
    pub sections: Vec<Section>,
    pub search_index: SearchIndex,
    pub metadata: ReadmeMetadata,
}

impl ReadmeContent {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            search_index: SearchIndex::new(),
            metadata: ReadmeMetadata::default(),
        }
    }
}

impl Default for ReadmeContent {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryEntry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub terms: HashMap<String, Vec<SearchLocation>>,
    pub total_terms: usize,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
            total_terms: 0,
        }
    }

    pub fn add_term(&mut self, term: String, location: SearchLocation) {
        self.terms.entry(term.to_lowercase()).or_default().push(location);
        self.total_terms += 1;
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (term, locations) in &self.terms {
            if term.contains(&query_lower) {
                for location in locations {
                    // Skip raw content matches - we only want names and descriptions
                    if location.search_priority == SearchPriority::RawContent {
                        continue;
                    }

                    results.push(SearchResult {
                        section_index: location.section_index,
                        entry_index: location.entry_index,
                        line_content: location.line_content.clone(),
                        relevance_score: calculate_relevance_score(term, &query_lower) * location.search_priority.score_multiplier(),
                        github_url: location.github_url.clone(),
                    });
                }
            }
        }

        // Sort by relevance score (higher is better)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
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
        if let Some(idx) = index {
            if idx < self.offset {
                self.offset = idx;
            }
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
        if i >= self.offset + 10 { // Assume 10-item viewport
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