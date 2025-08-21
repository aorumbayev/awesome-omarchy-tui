#[cfg(feature = "aur-theme-preview")]
use crate::models::ThemeEntry;
use crate::models::{
    ReadmeContent, ReadmeMetadata, RepositoryEntry, SearchIndex, SearchLocation, SearchPriority,
    Section,
};
#[cfg(feature = "aur-theme-preview")]
use crate::models::{Theme, ThemeColorPalette, ThemeColors};
use anyhow::{Result, anyhow};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// Parser for awesome-omarchy README markdown content
pub struct ReadmeParser {
    /// Common section headers found in awesome-* repositories
    known_sections: Vec<&'static str>,
    /// Patterns to exclude from parsing (e.g., TOC, badges)
    exclusion_patterns: Vec<&'static str>,
}

impl ReadmeParser {
    pub fn new() -> Self {
        Self {
            known_sections: vec![
                "Official Resources",
                "Alternative Implementations",
                "Themes",
                "Development Tools",
                "Related Projects",
                "Community Resources",
                "Documentation",
                "Tutorials",
                "Examples",
                "Libraries",
                "Plugins",
                "Extensions",
                "Integrations",
                "Testing",
                "Deployment",
                "Monitoring",
                "Security",
                "Performance",
                "Utilities",
                "Resources",
            ],
            exclusion_patterns: vec![
                "Contents",
                "Table of Contents",
                "TOC",
                "Contributing",
                "License",
                "Awesome",
                "Badge",
            ],
        }
    }

    /// Parse markdown content into structured ReadmeContent
    pub fn parse(&self, markdown_content: &str) -> Result<ReadmeContent> {
        if markdown_content.trim().is_empty() {
            return Err(anyhow!("Empty markdown content"));
        }

        let parser = Parser::new(markdown_content);
        let mut readme_content = ReadmeContent::default();
        let mut current_section: Option<Section> = None;
        let mut current_text = String::new();
        let mut is_in_header = false;
        let mut current_header_level = 0;
        let mut header_text = String::new();
        let mut link_url = String::new();
        let mut is_in_list_item = false;
        let mut current_item_text = String::new(); // Text for current list item only
        let mut metadata = ReadmeMetadata::default();

        // Extract title from first heading
        let mut title_extracted = false;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    is_in_header = true;
                    current_header_level = level as u32;
                    header_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if is_in_header {
                        let header = header_text.trim().to_string();

                        // Extract title from first H1
                        if !title_extracted && current_header_level == 1 {
                            metadata.title = header.clone();
                            title_extracted = true;
                        }

                        // Check if this is a section header we should parse
                        if current_header_level >= 2 && self.should_parse_section(&header) {
                            // Save previous section if exists
                            if let Some(section) = current_section.take() {
                                readme_content.sections.push(section);
                            }

                            // Start new section
                            current_section = Some(Section::new(header));
                            current_text.clear();
                        }

                        is_in_header = false;
                    }
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    link_url = dest_url.to_string();
                }
                Event::End(TagEnd::Link) => {
                    // Link ended, continue accumulating text for description
                }
                Event::Text(text) => {
                    if is_in_header {
                        header_text.push_str(&text);
                    } else {
                        current_text.push_str(&text);
                        if is_in_list_item {
                            current_item_text.push_str(&text); // Accumulate ALL text for current item
                        }
                        if let Some(ref mut section) = current_section {
                            section.raw_content.push_str(&text);
                        }
                    }
                }
                Event::Code(code) => {
                    if is_in_header {
                        header_text.push_str(&code);
                    } else {
                        current_text.push_str(&code);
                        if is_in_list_item {
                            current_item_text.push_str(&code); // Accumulate code for current item only
                        }
                        if let Some(ref mut section) = current_section {
                            section.raw_content.push_str(&code);
                        }
                    }
                }
                Event::Start(Tag::List(_)) => {
                    if !is_in_header && !current_text.trim().is_empty() {
                        current_text.push('\n');
                    }
                }
                Event::Start(Tag::Item) => {
                    if !is_in_header {
                        current_text.push_str("• ");
                        is_in_list_item = true;
                        current_item_text.clear(); // Clear item text for new list item
                    }
                }
                Event::End(TagEnd::Item) => {
                    if !is_in_header {
                        current_text.push('\n');
                        if let Some(ref mut section) = current_section {
                            section.raw_content.push('\n');
                        }

                        // Process the accumulated text for the current item if we have a GitHub link
                        if is_in_list_item
                            && !link_url.is_empty()
                            && !current_item_text.trim().is_empty()
                            && let Some(ref mut section) = current_section
                            && self.is_github_link(&link_url)
                        {
                            let entry =
                                self.extract_repository_entry(&current_item_text, &link_url);
                            section.entries.push(entry);
                            section.entry_count += 1;
                        }

                        is_in_list_item = false;
                        current_item_text.clear();
                        link_url.clear();
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !is_in_header {
                        current_text.push('\n');
                        if is_in_list_item {
                            current_item_text.push('\n');
                        }
                        if let Some(ref mut section) = current_section {
                            section.raw_content.push('\n');
                        }
                    }
                }
                _ => {}
            }
        }

        // Save last section
        if let Some(section) = current_section {
            readme_content.sections.push(section);
        }

        // Update metadata
        metadata.total_entries = readme_content.sections.iter().map(|s| s.entry_count).sum();
        readme_content.metadata = metadata;

        // Build search index
        readme_content.search_index = self.build_search_index(&readme_content.sections)?;

        // Validate we have meaningful content
        if readme_content.sections.is_empty() {
            return Err(anyhow!("No valid sections found in markdown content"));
        }

        Ok(readme_content)
    }

    fn should_parse_section(&self, header: &str) -> bool {
        // Skip excluded patterns
        for pattern in &self.exclusion_patterns {
            if header.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }

        // Include known sections
        for section in &self.known_sections {
            if header.to_lowercase().contains(&section.to_lowercase()) {
                return true;
            }
        }

        // Include sections that look like categories (contain certain keywords)
        let category_indicators = vec![
            "tools",
            "libraries",
            "resources",
            "projects",
            "extensions",
            "plugins",
            "integrations",
            "frameworks",
            "platforms",
            "services",
            "utilities",
            "apps",
            "applications",
            "implementations",
            "solutions",
        ];

        for indicator in category_indicators {
            if header.to_lowercase().contains(indicator) {
                return true;
            }
        }

        // Default to including sections (better to have too many than too few)
        true
    }

    /// Check if URL is a GitHub repository link
    fn is_github_link(&self, url: &str) -> bool {
        url.starts_with("https://github.com/") && 
            url.matches('/').count() >= 4 && // github.com/user/repo
            !url.contains("/issues") && 
            !url.contains("/wiki") && 
            !url.contains("/releases")
    }

    /// Extract repository entry from link text and URL
    fn extract_repository_entry(&self, text: &str, url: &str) -> RepositoryEntry {
        let (title, description) = self.split_title_description(text);
        let tags = self.extract_tags(&description);

        RepositoryEntry {
            title,
            url: url.to_string(),
            description,
            tags,
        }
    }

    /// Split text into title and description
    fn split_title_description(&self, text: &str) -> (String, String) {
        let text = text.trim();

        // Handle markdown list item format: "link_text - description"
        // The link_text is the repository name, description follows after " - "
        if let Some(pos) = text.find(" - ") {
            let title = text[..pos].trim().to_string();
            let description = text[pos + 3..].trim().to_string();
            return (title, description);
        }

        // Handle colon separator format: "title: description"
        if let Some(pos) = text.find(": ") {
            let title = text[..pos].trim().to_string();
            let description = text[pos + 2..].trim().to_string();
            return (title, description);
        }

        // If no separator found, use the whole text as title
        (text.to_string(), String::new())
    }

    /// Extract tags from description text
    fn extract_tags(&self, description: &str) -> Vec<String> {
        let mut tags = Vec::new();
        let description_lower = description.to_lowercase();

        // Common tag patterns in awesome lists (removed generic "go" to prevent spurious matches)
        let tag_indicators = vec![
            ("rust", "rust"),
            ("python", "python"),
            ("javascript", "javascript"),
            ("typescript", "typescript"),
            ("golang", "go"), // More specific pattern for Go language
            ("java", "java"),
            ("c++", "cpp"),
            ("cli", "command-line"),
            ("web", "web"),
            ("api", "api"),
            ("tool", "tool"),
            ("library", "library"),
            ("framework", "framework"),
            ("plugin", "plugin"),
            ("extension", "extension"),
        ];

        for (pattern, tag) in tag_indicators {
            if description_lower.contains(pattern) {
                tags.push(tag.to_string());
            }
        }

        tags
    }

    /// Extract theme entries from README content
    #[cfg(feature = "aur-theme-preview")]
    pub fn extract_themes_from_readme(&self, readme: &ReadmeContent) -> Result<Vec<ThemeEntry>> {
        let mut theme_entries = Vec::new();

        // Find the "Themes" section
        for section in &readme.sections {
            if section.title.to_lowercase().contains("theme") {
                // Extract theme entries from this section
                for entry in &section.entries {
                    // Ensure it's a GitHub repository
                    if self.is_github_link(&entry.url) {
                        theme_entries.push(ThemeEntry {
                            name: entry.title.clone(),
                            url: entry.url.clone(),
                            description: entry.description.clone(),
                        });
                    }
                }
                break; // Found themes section, stop looking
            }
        }

        if theme_entries.is_empty() {
            return Err(anyhow!("No theme entries found in README"));
        }

        Ok(theme_entries)
    }

    /// Build search index from parsed sections
    fn build_search_index(&self, sections: &[Section]) -> Result<SearchIndex> {
        let mut search_index = SearchIndex::default();

        for (section_idx, section) in sections.iter().enumerate() {
            // Index section title (but not prioritized for repository search)
            // self.index_text(&section.title, section_idx, None, SearchPriority::RawContent, None, &mut search_index);

            // Index entries with priorities
            for (entry_idx, entry) in section.entries.iter().enumerate() {
                // Priority 1: Repository names
                self.index_text(
                    &entry.title,
                    section_idx,
                    Some(entry_idx),
                    SearchPriority::RepositoryName,
                    Some(&entry.url),
                    &mut search_index,
                );

                // Priority 2: Repository descriptions
                if !entry.description.is_empty() {
                    self.index_text(
                        &entry.description,
                        section_idx,
                        Some(entry_idx),
                        SearchPriority::Description,
                        Some(&entry.url),
                        &mut search_index,
                    );
                }

                // Don't index tags or raw content for prioritized search
            }

            // Don't index raw content for prioritized search
            // self.index_text(&section.raw_content, section_idx, None, SearchPriority::RawContent, None, &mut search_index);
        }

        Ok(search_index)
    }

    /// Add text to search index with location information
    fn index_text(
        &self,
        text: &str,
        section_idx: usize,
        entry_idx: Option<usize>,
        priority: SearchPriority,
        github_url: Option<&str>,
        search_index: &mut SearchIndex,
    ) {
        let words = text
            .split_whitespace()
            .filter(|word| word.len() > 2) // Skip very short words
            .map(|word| {
                // Clean word of punctuation
                word.chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>()
                    .trim()
                    .to_lowercase()
            })
            .filter(|word| !word.is_empty() && word.len() > 2);

        for word in words {
            let location = SearchLocation {
                section_index: section_idx,
                entry_index: entry_idx,
                line_content: text.to_string(),
                start_pos: 0, // Could be enhanced to track actual positions
                end_pos: text.len(),
                search_priority: priority.clone(),
                github_url: github_url.map(|s| s.to_string()),
            };

            search_index.add_term(word, location);
        }
    }
}

impl Default for ReadmeParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for theme files (alacritty.toml)
#[cfg(feature = "aur-theme-preview")]
pub struct ThemeParser;

#[cfg(feature = "aur-theme-preview")]
impl ThemeParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse alacritty.toml theme file
    pub fn parse_alacritty_theme(&self, theme_name: &str, toml_content: &str) -> Result<Theme> {
        use serde_json::Value;

        // Parse TOML content
        let toml_value: toml::Value =
            toml::from_str(toml_content).map_err(|e| anyhow!("Failed to parse TOML: {}", e))?;

        // Convert to JSON for easier navigation
        let json_str = serde_json::to_string(&toml_value)?;
        let json: Value = serde_json::from_str(&json_str)?;

        // Extract colors from the TOML structure
        let colors = self.extract_colors_from_json(&json)?;

        Ok(Theme {
            name: self.format_theme_name(theme_name),
            description: format!("Theme colors from {theme_name}"),
            source_url: format!(
                "https://github.com/basecamp/omarchy/tree/main/themes/{theme_name}"
            ),
            colors,
        })
    }

    /// Parse alacritty.yml theme file
    #[cfg(feature = "aur-theme-preview")]
    pub fn parse_alacritty_yaml(&self, theme_name: &str, yaml_content: &str) -> Result<Theme> {
        use serde_json::Value;
        use serde_yaml;

        // Parse YAML content
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content)
            .map_err(|e| anyhow!("Failed to parse YAML: {}", e))?;

        // Convert to JSON for easier navigation
        let json_str = serde_json::to_string(&yaml_value)?;
        let json: Value = serde_json::from_str(&json_str)?;

        // Extract colors from the YAML structure
        let colors = self.extract_colors_from_json(&json)?;

        Ok(Theme {
            name: self.format_theme_name(theme_name),
            description: format!("Theme colors from {}", theme_name),
            source_url: format!("Custom theme: {}", theme_name),
            colors,
        })
    }

    fn extract_colors_from_json(&self, json: &serde_json::Value) -> Result<ThemeColors> {
        // Try to find colors in common TOML structures
        let colors = if let Some(colors) = json.get("colors") {
            colors
        } else if let Some(theme) = json.get("theme") {
            if let Some(colors) = theme.get("colors") {
                colors
            } else {
                theme
            }
        } else {
            json
        };

        // Extract background and foreground
        let background = self
            .extract_color(colors, &["primary", "background"])
            .or_else(|| self.extract_color(colors, &["background"]))
            .unwrap_or_else(|| "#1a1b26".to_string());

        let foreground = self
            .extract_color(colors, &["primary", "foreground"])
            .or_else(|| self.extract_color(colors, &["foreground"]))
            .unwrap_or_else(|| "#a9b1d6".to_string());

        // Extract normal colors
        let normal = self
            .extract_color_palette(colors, "normal")
            .unwrap_or_else(|| self.create_default_palette());

        // Extract bright colors
        let bright = self
            .extract_color_palette(colors, "bright")
            .unwrap_or_else(|| self.create_default_bright_palette());

        Ok(ThemeColors {
            background,
            foreground,
            normal,
            bright,
        })
    }

    fn extract_color(&self, colors: &serde_json::Value, path: &[&str]) -> Option<String> {
        let mut current = colors;
        for key in path {
            current = current.get(key)?;
        }

        // Handle different color formats
        match current {
            serde_json::Value::String(s) => Some(self.normalize_color(s)),
            serde_json::Value::Number(n) => {
                // Convert number to hex color
                n.as_u64().map(|num| format!("#{num:06x}"))
            }
            _ => None,
        }
    }

    fn extract_color_palette(
        &self,
        colors: &serde_json::Value,
        palette_name: &str,
    ) -> Option<ThemeColorPalette> {
        let palette = colors.get(palette_name)?;

        Some(ThemeColorPalette {
            black: self
                .extract_color(palette, &["black"])
                .unwrap_or_else(|| "#000000".to_string()),
            red: self
                .extract_color(palette, &["red"])
                .unwrap_or_else(|| "#ff0000".to_string()),
            green: self
                .extract_color(palette, &["green"])
                .unwrap_or_else(|| "#00ff00".to_string()),
            yellow: self
                .extract_color(palette, &["yellow"])
                .unwrap_or_else(|| "#ffff00".to_string()),
            blue: self
                .extract_color(palette, &["blue"])
                .unwrap_or_else(|| "#0000ff".to_string()),
            magenta: self
                .extract_color(palette, &["magenta"])
                .unwrap_or_else(|| "#ff00ff".to_string()),
            cyan: self
                .extract_color(palette, &["cyan"])
                .unwrap_or_else(|| "#00ffff".to_string()),
            white: self
                .extract_color(palette, &["white"])
                .unwrap_or_else(|| "#ffffff".to_string()),
        })
    }

    fn normalize_color(&self, color: &str) -> String {
        let color = color.trim();

        // Ensure color starts with #
        if color.starts_with('#') {
            color.to_string()
        } else if color.len() == 6 && color.chars().all(|c| c.is_ascii_hexdigit()) {
            format!("#{color}")
        } else {
            // Try to parse as hex without #
            color.to_string()
        }
    }

    fn format_theme_name(&self, name: &str) -> String {
        name.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.collect::<String>()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn create_default_palette(&self) -> ThemeColorPalette {
        ThemeColorPalette {
            black: "#32344a".to_string(),
            red: "#f7768e".to_string(),
            green: "#9ece6a".to_string(),
            yellow: "#e0af68".to_string(),
            blue: "#7aa2f7".to_string(),
            magenta: "#ad8ee6".to_string(),
            cyan: "#449dab".to_string(),
            white: "#787c99".to_string(),
        }
    }

    fn create_default_bright_palette(&self) -> ThemeColorPalette {
        ThemeColorPalette {
            black: "#444b6a".to_string(),
            red: "#ff7a93".to_string(),
            green: "#b9f27c".to_string(),
            yellow: "#ff9e64".to_string(),
            blue: "#7da6ff".to_string(),
            magenta: "#bb9af7".to_string(),
            cyan: "#0db9d7".to_string(),
            white: "#acb0d0".to_string(),
        }
    }
}

#[cfg(feature = "aur-theme-preview")]
impl Default for ThemeParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        let parser = ReadmeParser::new();
        let result = parser.parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_basic_structure() {
        let parser = ReadmeParser::new();
        let markdown = r#"
# Awesome Omarchy

## Official Resources

- [Official Website](https://github.com/omarchy/awesome) - The main project website
- [Documentation](https://github.com/omarchy/docs) - Comprehensive documentation

## Development Tools

- [CLI Tool](https://github.com/omarchy/cli) - Command line interface for Omarchy
"#;

        let result = parser.parse(markdown).unwrap();
        assert_eq!(result.metadata.title, "Awesome Omarchy");
        assert_eq!(result.sections.len(), 2);
        assert_eq!(result.sections[0].title, "Official Resources");
        assert_eq!(result.sections[0].entries.len(), 2);
        assert_eq!(result.sections[1].title, "Development Tools");
        assert_eq!(result.sections[1].entries.len(), 1);
    }

    #[test]
    fn test_extract_repository_entry() {
        let parser = ReadmeParser::new();
        let text = "Awesome Tool - A comprehensive tool for doing awesome things";
        let url = "https://github.com/user/awesome-tool";

        let entry = parser.extract_repository_entry(text, url);
        assert_eq!(entry.title, "Awesome Tool");
        assert_eq!(
            entry.description,
            "A comprehensive tool for doing awesome things"
        );
        assert_eq!(entry.url, url);
    }

    #[test]
    fn test_search_functionality() {
        let parser = ReadmeParser::new();
        let markdown = r#"
# Test Awesome List

## Tools

- [Rust CLI](https://github.com/user/rust-cli) - A command line tool written in Rust
- [Python Script](https://github.com/user/python-script) - A useful Python script

## Libraries

- [Web Framework](https://github.com/user/web-framework) - Fast web framework
"#;

        let result = parser.parse(markdown).unwrap();
        let search_results = result.search_index.search("rust");
        assert!(!search_results.is_empty());

        let search_results = result.search_index.search("web");
        assert!(!search_results.is_empty());
    }
}
