use anyhow::Result;
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs;
use crate::models::ReadmeContent;
use crate::parser::ReadmeParser;

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
        if !force_refresh {
            if let Ok(cached) = self.load_from_cache().await {
                return Ok(cached);
            }
        }

        // Fetch from GitHub
        let response = self.client.get(url).send().await?;
        let markdown_content = response.text().await?;

        // Parse the markdown content using comprehensive parser
        let parser = ReadmeParser::new();
        let readme_content = parser.parse(&markdown_content).unwrap_or_else(|_| {
            // Fallback to simple parsing if comprehensive parser fails
            self.simple_parse(&markdown_content).unwrap_or_else(|_| ReadmeContent::default())
        });

        // Cache the result
        self.save_to_cache(&readme_content).await?;

        Ok(readme_content)
    }

    /// Simple markdown parser until module resolution is fixed
    fn simple_parse(&self, markdown_content: &str) -> Result<ReadmeContent> {
        use crate::models::{ReadmeMetadata, Section};
        use pulldown_cmark::{Parser, Event, Tag, TagEnd};
        use anyhow::anyhow;

        if markdown_content.trim().is_empty() {
            return Err(anyhow!("Empty markdown content"));
        }

        let parser = Parser::new(markdown_content);
        let mut readme_content = ReadmeContent::new();
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
