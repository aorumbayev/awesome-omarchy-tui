use crate::models::ThemeEntry;
use crate::models::{ReadmeContent, Theme};
use crate::parser::ReadmeParser;
use crate::parser::ThemeParser;
use anyhow::Result;
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs;

pub struct HttpClient {
    client: Client,
    cache_dir: PathBuf,
}

impl HttpClient {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("awesome-omarchy-tui");

        Self {
            client: Client::new(),
            cache_dir,
        }
    }

    /// Fetch README content with caching and comprehensive parsing
    pub async fn fetch_readme(&self, force_refresh: bool) -> Result<ReadmeContent> {
        let url = "https://raw.githubusercontent.com/aorumbayev/awesome-omarchy/refs/heads/main/README.md";

        // Try to load from cache first (unless force refresh)
        if !force_refresh && let Ok(cached) = self.load_from_cache().await {
            return Ok(cached);
        }

        // Fetch from GitHub
        let response = self.client.get(url).send().await?;
        let markdown_content = response.text().await?;

        // Parse the markdown content using comprehensive parser
        let parser = ReadmeParser::new();
        let readme_content = parser.parse(&markdown_content).unwrap_or_else(|_| {
            // Fallback to simple parsing if comprehensive parser fails
            self.simple_parse(&markdown_content)
                .unwrap_or_else(|_| ReadmeContent::default())
        });

        // Cache the result
        self.save_to_cache(&readme_content).await?;

        Ok(readme_content)
    }

    /// Fetch themes from the cached README "Themes" section
    pub async fn fetch_themes_from_readme(&self) -> Result<Vec<ThemeEntry>> {
        // Load cached README content
        let readme_content = match self.load_from_cache().await {
            Ok(content) => content,
            Err(_) => self.fetch_readme(false).await?,
        };

        // Extract themes from the "Themes" section
        let parser = ReadmeParser::new();
        parser.extract_themes_from_readme(&readme_content)
    }

    /// Lazy load a specific theme's alacritty.toml from GitHub
    pub async fn fetch_theme_colors(&self, theme_entry: &ThemeEntry) -> Result<Theme> {
        // Extract GitHub owner/repo from URL
        let url_parts: Vec<&str> = theme_entry.url.split('/').collect();
        if url_parts.len() < 5 || !theme_entry.url.starts_with("https://github.com/") {
            return Err(anyhow::anyhow!("Invalid GitHub URL: {}", theme_entry.url));
        }

        let owner = url_parts[3];
        let repo = url_parts[4];

        // Try common alacritty config file locations
        let possible_paths = vec![
            "alacritty.toml".to_string(),
            "alacritty.yml".to_string(),
            ".config/alacritty/alacritty.toml".to_string(),
            ".config/alacritty/alacritty.yml".to_string(),
            "config/alacritty.toml".to_string(),
            "config/alacritty.yml".to_string(),
            format!("{}.toml", theme_entry.name.to_lowercase()),
            format!("{}.yml", theme_entry.name.to_lowercase()),
        ];

        for path in possible_paths {
            let raw_url = format!("https://raw.githubusercontent.com/{owner}/{repo}/main/{path}");

            if let Ok(response) = self.client.get(&raw_url).send().await
                && response.status().is_success()
                && let Ok(content) = response.text().await
            {
                let parser = ThemeParser::new();

                // Try parsing as TOML first
                if path.ends_with(".toml") {
                    if let Ok(theme) = parser.parse_alacritty_theme(&theme_entry.name, &content) {
                        return Ok(Theme {
                            name: theme_entry.name.clone(),
                            description: theme_entry.description.clone(),
                            source_url: theme_entry.url.clone(),
                            colors: theme.colors,
                        });
                    }
                }
                // Try parsing as YAML if TOML fails or for .yml files
                else if (path.ends_with(".yml") || path.ends_with(".yaml"))
                    && let Ok(theme) = parser.parse_alacritty_yaml(&theme_entry.name, &content)
                {
                    return Ok(Theme {
                        name: theme_entry.name.clone(),
                        description: theme_entry.description.clone(),
                        source_url: theme_entry.url.clone(),
                        colors: theme.colors,
                    });
                }
            }
        }

        // Fallback: create a default theme with the entry information
        Ok(self.create_fallback_theme(theme_entry))
    }

    fn create_fallback_theme(&self, theme_entry: &ThemeEntry) -> Theme {
        // Create a visually distinct fallback theme
        let theme_colors = match theme_entry.name.to_lowercase().as_str() {
            name if name.contains("dark") => self.create_dark_theme_colors(),
            name if name.contains("light") => self.create_light_theme_colors(),
            name if name.contains("blue") => self.create_blue_theme_colors(),
            name if name.contains("green") => self.create_green_theme_colors(),
            name if name.contains("red") => self.create_red_theme_colors(),
            name if name.contains("purple") || name.contains("violet") => {
                self.create_purple_theme_colors()
            }
            _ => self.create_default_theme_colors(),
        };

        Theme {
            name: theme_entry.name.clone(),
            description: format!("{} (fallback colors)", theme_entry.description),
            source_url: theme_entry.url.clone(),
            colors: theme_colors,
        }
    }

    fn create_dark_theme_colors(&self) -> crate::models::ThemeColors {
        use crate::models::{ThemeColorPalette, ThemeColors};
        ThemeColors {
            background: "#0d1117".to_string(),
            foreground: "#c9d1d9".to_string(),
            normal: ThemeColorPalette {
                black: "#484f58".to_string(),
                red: "#ff7b72".to_string(),
                green: "#7ee787".to_string(),
                yellow: "#f2cc60".to_string(),
                blue: "#79c0ff".to_string(),
                magenta: "#d2a8ff".to_string(),
                cyan: "#56d4dd".to_string(),
                white: "#c9d1d9".to_string(),
            },
            bright: ThemeColorPalette {
                black: "#6e7681".to_string(),
                red: "#ffa198".to_string(),
                green: "#56d364".to_string(),
                yellow: "#e3b341".to_string(),
                blue: "#58a6ff".to_string(),
                magenta: "#bc8cff".to_string(),
                cyan: "#39c5cf".to_string(),
                white: "#f0f6fc".to_string(),
            },
        }
    }

    fn create_light_theme_colors(&self) -> crate::models::ThemeColors {
        use crate::models::{ThemeColorPalette, ThemeColors};
        ThemeColors {
            background: "#ffffff".to_string(),
            foreground: "#24292f".to_string(),
            normal: ThemeColorPalette {
                black: "#24292f".to_string(),
                red: "#cf222e".to_string(),
                green: "#116329".to_string(),
                yellow: "#4d2d00".to_string(),
                blue: "#0969da".to_string(),
                magenta: "#8250df".to_string(),
                cyan: "#1b7c83".to_string(),
                white: "#6e7781".to_string(),
            },
            bright: ThemeColorPalette {
                black: "#57606a".to_string(),
                red: "#a40e26".to_string(),
                green: "#1a7f37".to_string(),
                yellow: "#633c01".to_string(),
                blue: "#218bff".to_string(),
                magenta: "#a475f9".to_string(),
                cyan: "#3192aa".to_string(),
                white: "#8c959f".to_string(),
            },
        }
    }

    fn create_blue_theme_colors(&self) -> crate::models::ThemeColors {
        use crate::models::{ThemeColorPalette, ThemeColors};
        ThemeColors {
            background: "#0f1419".to_string(),
            foreground: "#bfbab0".to_string(),
            normal: ThemeColorPalette {
                black: "#1c2023".to_string(),
                red: "#f07178".to_string(),
                green: "#c3e88d".to_string(),
                yellow: "#ffcb6b".to_string(),
                blue: "#82aaff".to_string(),
                magenta: "#c792ea".to_string(),
                cyan: "#89ddff".to_string(),
                white: "#d6deeb".to_string(),
            },
            bright: ThemeColorPalette {
                black: "#5c6370".to_string(),
                red: "#f78c6c".to_string(),
                green: "#addb67".to_string(),
                yellow: "#f78c6a".to_string(),
                blue: "#82aaff".to_string(),
                magenta: "#c792ea".to_string(),
                cyan: "#7fdbca".to_string(),
                white: "#ffffff".to_string(),
            },
        }
    }

    fn create_green_theme_colors(&self) -> crate::models::ThemeColors {
        use crate::models::{ThemeColorPalette, ThemeColors};
        ThemeColors {
            background: "#0d1b2a".to_string(),
            foreground: "#a8cc8c".to_string(),
            normal: ThemeColorPalette {
                black: "#0d1b2a".to_string(),
                red: "#e06c75".to_string(),
                green: "#98c379".to_string(),
                yellow: "#e5c07b".to_string(),
                blue: "#61afef".to_string(),
                magenta: "#c678dd".to_string(),
                cyan: "#56b6c2".to_string(),
                white: "#979eab".to_string(),
            },
            bright: ThemeColorPalette {
                black: "#393e46".to_string(),
                red: "#f07178".to_string(),
                green: "#c3e88d".to_string(),
                yellow: "#ffcb6b".to_string(),
                blue: "#82aaff".to_string(),
                magenta: "#c792ea".to_string(),
                cyan: "#89ddff".to_string(),
                white: "#ffffff".to_string(),
            },
        }
    }

    fn create_red_theme_colors(&self) -> crate::models::ThemeColors {
        use crate::models::{ThemeColorPalette, ThemeColors};
        ThemeColors {
            background: "#2b1b17".to_string(),
            foreground: "#d4d4d4".to_string(),
            normal: ThemeColorPalette {
                black: "#2b1b17".to_string(),
                red: "#f07178".to_string(),
                green: "#c3e88d".to_string(),
                yellow: "#ffcb6b".to_string(),
                blue: "#82aaff".to_string(),
                magenta: "#c792ea".to_string(),
                cyan: "#89ddff".to_string(),
                white: "#eeffff".to_string(),
            },
            bright: ThemeColorPalette {
                black: "#5c6370".to_string(),
                red: "#ff6b6b".to_string(),
                green: "#95e1d3".to_string(),
                yellow: "#f8ca93".to_string(),
                blue: "#74b9ff".to_string(),
                magenta: "#e17899".to_string(),
                cyan: "#81ecec".to_string(),
                white: "#fdcb6e".to_string(),
            },
        }
    }

    fn create_purple_theme_colors(&self) -> crate::models::ThemeColors {
        use crate::models::{ThemeColorPalette, ThemeColors};
        ThemeColors {
            background: "#1e1a2e".to_string(),
            foreground: "#d4c5f9".to_string(),
            normal: ThemeColorPalette {
                black: "#1e1a2e".to_string(),
                red: "#e06c75".to_string(),
                green: "#98c379".to_string(),
                yellow: "#e5c07b".to_string(),
                blue: "#61afef".to_string(),
                magenta: "#c678dd".to_string(),
                cyan: "#56b6c2".to_string(),
                white: "#abb2bf".to_string(),
            },
            bright: ThemeColorPalette {
                black: "#5c6370".to_string(),
                red: "#be5046".to_string(),
                green: "#98c379".to_string(),
                yellow: "#d19a66".to_string(),
                blue: "#61afef".to_string(),
                magenta: "#c678dd".to_string(),
                cyan: "#56b6c2".to_string(),
                white: "#ffffff".to_string(),
            },
        }
    }

    fn create_default_theme_colors(&self) -> crate::models::ThemeColors {
        use crate::models::{ThemeColorPalette, ThemeColors};
        ThemeColors {
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
        }
    }

    /// Simple markdown parser until module resolution is fixed
    fn simple_parse(&self, markdown_content: &str) -> Result<ReadmeContent> {
        use crate::models::{ReadmeMetadata, Section};
        use anyhow::anyhow;
        use pulldown_cmark::{Event, Parser, Tag, TagEnd};

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
        let mut metadata = ReadmeMetadata::default();
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
                        if current_header_level >= 2 {
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
                Event::Text(text) => {
                    if is_in_header {
                        header_text.push_str(&text);
                    } else {
                        current_text.push_str(&text);
                        if let Some(ref mut section) = current_section {
                            section.raw_content.push_str(&text);
                        }
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !is_in_header {
                        current_text.push('\n');
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

        // Create fallback content if no sections found
        if readme_content.sections.is_empty() {
            let section = Section {
                title: "README".to_string(),
                entries: vec![],
                raw_content: markdown_content.to_string(),
                entry_count: 0,
            };
            readme_content.sections.push(section);
        }

        // Update metadata
        metadata.total_entries = readme_content.sections.iter().map(|s| s.entry_count).sum();
        readme_content.metadata = metadata;

        Ok(readme_content)
    }

    /// Load cached README content
    async fn load_from_cache(&self) -> Result<ReadmeContent> {
        let cache_path = self.cache_dir.join("readme.json");
        let content = fs::read_to_string(cache_path).await?;
        let readme_content: ReadmeContent = serde_json::from_str(&content)?;
        Ok(readme_content)
    }

    /// Save README content to cache
    async fn save_to_cache(&self, content: &ReadmeContent) -> Result<()> {
        fs::create_dir_all(&self.cache_dir).await?;
        let cache_path = self.cache_dir.join("readme.json");
        let json = serde_json::to_string_pretty(content)?;
        fs::write(cache_path, json).await?;
        Ok(())
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
