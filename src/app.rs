use crate::models::ThemeEntry;
use crate::{
    HttpClient,
    models::{
        AppState, FocusArea, ListState, PreviewState, ReadmeContent, SearchResult, TabState,
        ThemeApplicator, ThemeBrowserState,
    },
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

pub struct App {
    pub state: AppState,
    pub tabs: Vec<TabState>,
    pub current_tab: usize,
    pub search_query: String,
    pub search_mode: bool,
    pub search_results: Vec<SearchResult>,
    pub search_selection: Option<usize>,
    pub readme_content: Option<ReadmeContent>,
    pub quit: bool,
    pub client: HttpClient,
    pub focus_area: FocusArea,
    pub theme_browser: ThemeBrowserState,
    pub theme_browser_mode: bool,
    pub theme_applicator: ThemeApplicator,
    pub preview_state: PreviewState,
    pub theme_entries: Vec<ThemeEntry>,
}

impl App {
    pub async fn new(client: HttpClient) -> Result<Self> {
        let mut app = Self {
            state: AppState::Loading,
            tabs: Vec::new(),
            current_tab: 0,
            search_query: String::new(),
            search_mode: false,
            search_results: Vec::new(),
            search_selection: None,
            readme_content: None,
            quit: false,
            client,
            focus_area: FocusArea::default(),
            theme_browser: ThemeBrowserState {
                themes: Vec::new(),
                selected_index: None,
                loading: false,
                error: None,
                preview_theme: None,
                search_mode: false,
                search_query: String::new(),
                filtered_themes: Vec::new(),
                filtered_selected: None,
            },
            theme_browser_mode: false,
            theme_applicator: ThemeApplicator::default(),
            preview_state: PreviewState::default(),
            theme_entries: Vec::new(),
        };

        app.load_readme(false).await?;
        Ok(app)
    }

    pub async fn load_readme(&mut self, force_refresh: bool) -> Result<()> {
        self.state = AppState::Loading;

        match self.client.fetch_readme(force_refresh).await {
            Ok(content) => {
                self.tabs = content
                    .sections
                    .iter()
                    .enumerate()
                    .map(|(i, section)| {
                        TabState {
                            title: section.title.clone(),
                            section_index: i, // Ensure index matches the section position
                            scroll_offset: 0,
                            selected: i == 0,
                            list_state: if section.entries.is_empty() {
                                ListState {
                                    selected_index: None,
                                    offset: 0,
                                }
                            } else {
                                ListState::new()
                            },
                        }
                    })
                    .collect();

                // Validate current_tab is within bounds
                if self.current_tab >= self.tabs.len() {
                    self.current_tab = 0;
                }

                if !self.tabs.is_empty() {
                    self.tabs[self.current_tab].selected = true;
                }

                self.readme_content = Some(content);
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.state = AppState::Error(e.to_string());
            }
        }

        Ok(())
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.theme_browser_mode {
            self.handle_theme_browser_input(key).await?;
        } else if self.search_mode {
            self.handle_search_input(key).await?;
        } else {
            self.handle_normal_input(key).await?;
        }
        Ok(())
    }

    async fn handle_search_input(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyModifiers;

        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
                self.search_results.clear();
                self.search_selection = None;
            }
            KeyCode::Enter => {
                // If we have a selected search result, open its GitHub URL
                if let Some(selected_idx) = self.search_selection
                    && let Some(result) = self.search_results.get(selected_idx)
                    && let Some(url) = &result.github_url
                {
                    self.open_url(url);
                }
                self.search_mode = false;
                self.search_selection = None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Navigate down in search results
                if !self.search_results.is_empty() {
                    self.search_selection = Some(match self.search_selection {
                        Some(idx) if idx + 1 < self.search_results.len() => idx + 1,
                        Some(_) | None => 0, // Wrap to beginning
                    });
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Navigate up in search results
                if !self.search_results.is_empty() {
                    self.search_selection = Some(match self.search_selection {
                        Some(idx) if idx > 0 => idx - 1,
                        Some(_) | None => self.search_results.len() - 1, // Wrap to end
                    });
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                if !self.search_query.is_empty() {
                    self.perform_search();
                } else {
                    self.search_results.clear();
                    self.search_selection = None;
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit = true;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.perform_search();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_normal_input(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyModifiers;

        match key.code {
            // h/l keys - Switch between sidebar and content area focus
            KeyCode::Char('h') => {
                self.focus_area = FocusArea::Sidebar;
            }
            KeyCode::Char('l') => {
                self.focus_area = FocusArea::Content;
            }
            // Tab navigation - Next section (works from both areas)
            KeyCode::Tab => {
                self.next_tab();
            }
            KeyCode::BackTab => {
                self.previous_tab();
            }
            // j/k keys - Navigate within current focus area
            KeyCode::Char('j') | KeyCode::Down => match self.focus_area {
                FocusArea::Sidebar => {
                    self.next_tab();
                }
                FocusArea::Content => {
                    self.list_next();
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match self.focus_area {
                FocusArea::Sidebar => {
                    self.previous_tab();
                }
                FocusArea::Content => {
                    self.list_previous();
                }
            },
            // Reload
            KeyCode::Char('r' | 'R') => {
                self.load_readme(true).await?;
            }
            // Open GitHub
            KeyCode::Char('g' | 'G') => {
                self.open_github_repo();
            }
            // Search
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_mode = true;
                self.search_query.clear();
                self.search_results.clear();
            }
            // Theme Browser
            KeyCode::Char('t' | 'T') => {
                self.open_theme_browser().await?;
            }
            // Legacy scroll support (for paragraph fallback)
            KeyCode::PageUp => {
                self.page_up();
            }
            KeyCode::PageDown => {
                self.page_down();
            }
            KeyCode::Home => {
                self.list_first();
            }
            KeyCode::End => {
                self.list_last();
            }
            // Enter key - Open selected repository URL (only in content area)
            KeyCode::Enter => match self.focus_area {
                FocusArea::Content => {
                    // Get the currently selected repository entry and open its URL
                    if let Some(tab) = self.tabs.get(self.current_tab)
                        && let Some(selected_idx) = tab.list_state.selected_index
                        && let Some(ref readme) = self.readme_content
                        && let Some(section) = readme.sections.get(tab.section_index)
                        && let Some(entry) = section.entries.get(selected_idx)
                    {
                        self.open_url(&entry.url);
                    }
                }
                FocusArea::Sidebar => {
                    // No action for Enter in sidebar - could be extended in the future
                }
            },
            // Quit
            KeyCode::Char('q' | 'Q') => {
                self.quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit = true;
            }
            KeyCode::Esc => {
                // Clear search results if any
                self.search_results.clear();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            // Validate current index before changing selection
            if self.current_tab < self.tabs.len() {
                self.tabs[self.current_tab].selected = false;
            }
            self.current_tab = (self.current_tab + 1) % self.tabs.len();
            self.tabs[self.current_tab].selected = true;
        }
    }

    pub fn previous_tab(&mut self) {
        if !self.tabs.is_empty() {
            // Validate current index before changing selection
            if self.current_tab < self.tabs.len() {
                self.tabs[self.current_tab].selected = false;
            }
            if self.current_tab == 0 {
                self.current_tab = self.tabs.len() - 1;
            } else {
                self.current_tab -= 1;
            }
            self.tabs[self.current_tab].selected = true;
        }
    }

    pub fn page_up(&mut self) {
        // Keep for potential future use with large lists
        for _ in 0..5 {
            self.list_previous();
        }
    }

    pub fn page_down(&mut self) {
        // Keep for potential future use with large lists
        for _ in 0..5 {
            self.list_next();
        }
    }

    pub fn list_next(&mut self) {
        if self.current_tab < self.tabs.len()
            && let Some(tab) = self.tabs.get_mut(self.current_tab)
            && let Some(ref readme) = self.readme_content
            && let Some(section) = readme.sections.get(tab.section_index)
            && !section.entries.is_empty()
        {
            tab.list_state.select_next(section.entries.len());
        }
    }

    pub fn list_previous(&mut self) {
        if self.current_tab < self.tabs.len()
            && let Some(tab) = self.tabs.get_mut(self.current_tab)
            && let Some(ref readme) = self.readme_content
            && let Some(section) = readme.sections.get(tab.section_index)
            && !section.entries.is_empty()
        {
            tab.list_state.select_previous(section.entries.len());
        }
    }

    pub fn list_first(&mut self) {
        if self.current_tab < self.tabs.len()
            && let Some(tab) = self.tabs.get_mut(self.current_tab)
        {
            tab.list_state.select_first();
            tab.scroll_offset = 0;
        }
    }

    pub fn list_last(&mut self) {
        if self.current_tab < self.tabs.len()
            && let Some(tab) = self.tabs.get_mut(self.current_tab)
            && let Some(ref readme) = self.readme_content
            && let Some(section) = readme.sections.get(tab.section_index)
            && !section.entries.is_empty()
        {
            tab.list_state.select(Some(section.entries.len() - 1));
        }
    }

    pub fn perform_search(&mut self) {
        let Some(ref readme) = self.readme_content else {
            return;
        };

        if self.search_query.trim().is_empty() {
            self.search_results.clear();
            self.search_selection = None;
        } else {
            self.search_results = readme.search_index.search(&self.search_query);
            self.search_selection = (!self.search_results.is_empty()).then_some(0);
        }
    }

    pub fn open_github_repo(&self) {
        let repo_url = "https://github.com/aorumbayev/awesome-omarchy";
        self.open_url(repo_url);
    }

    pub fn open_url(&self, url: &str) {
        // Use system command to open URL in default browser
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .or_else(|_| {
                // Fallback for macOS
                std::process::Command::new("open").arg(url).spawn()
            })
            .or_else(|_| {
                // Fallback for Windows
                std::process::Command::new("cmd")
                    .args(["/c", "start", url])
                    .spawn()
            });
    }

    pub fn handle_resize(&mut self, _width: u16, _height: u16) {
        // Handle terminal resize if needed in the future
    }

    pub async fn on_tick(&mut self) {
        // Handle periodic updates if needed in the future
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn current_tab(&self) -> Option<&TabState> {
        self.tabs.get(self.current_tab)
    }

    pub fn get_metadata_summary(&self) -> Option<String> {
        self.readme_content.as_ref().map(|readme| {
            format!(
                "{} sections | {} total entries",
                readme.sections.len(),
                readme.metadata.total_entries
            )
        })
    }

    // Theme browser methods
    pub async fn open_theme_browser(&mut self) -> Result<()> {
        self.theme_browser_mode = true;
        self.theme_browser.loading = true;
        self.theme_browser.error = None;

        // Load theme entries from README if not already loaded
        if self.theme_entries.is_empty() {
            match self.client.fetch_themes_from_readme().await {
                Ok(entries) => {
                    self.theme_entries = entries;
                    // Convert to display format for the browser
                    self.theme_browser.themes = self
                        .theme_entries
                        .iter()
                        .map(|entry| {
                            use crate::models::{Theme, ThemeColorPalette, ThemeColors};
                            Theme {
                                name: entry.name.clone(),
                                description: entry.description.clone(),
                                source_url: entry.url.clone(),
                                // Placeholder colors - will be loaded lazily
                                colors: ThemeColors {
                                    background: "#000000".to_string(),
                                    foreground: "#ffffff".to_string(),
                                    normal: ThemeColorPalette {
                                        black: "#000000".to_string(),
                                        red: "#ff0000".to_string(),
                                        green: "#00ff00".to_string(),
                                        yellow: "#ffff00".to_string(),
                                        blue: "#0000ff".to_string(),
                                        magenta: "#ff00ff".to_string(),
                                        cyan: "#00ffff".to_string(),
                                        white: "#ffffff".to_string(),
                                    },
                                    bright: ThemeColorPalette {
                                        black: "#666666".to_string(),
                                        red: "#ff6666".to_string(),
                                        green: "#66ff66".to_string(),
                                        yellow: "#ffff66".to_string(),
                                        blue: "#6666ff".to_string(),
                                        magenta: "#ff66ff".to_string(),
                                        cyan: "#66ffff".to_string(),
                                        white: "#ffffff".to_string(),
                                    },
                                },
                            }
                        })
                        .collect();

                    self.theme_browser.selected_index = if !self.theme_browser.themes.is_empty() {
                        Some(0)
                    } else {
                        None
                    };

                    // Initialize filtered themes (empty = show all)
                    self.theme_browser.filtered_themes.clear();
                    self.theme_browser.filtered_selected = None;
                }
                Err(e) => {
                    self.theme_browser.error = Some(format!("Failed to load themes: {e}"));
                }
            }
        } else if self.theme_browser.selected_index.is_none()
            && !self.theme_browser.themes.is_empty()
        {
            self.theme_browser.selected_index = Some(0);
            // Initialize filtered themes (empty = show all)
            self.theme_browser.filtered_themes.clear();
            self.theme_browser.filtered_selected = None;
        }

        self.theme_browser.loading = false;
        Ok(())
    }

    pub fn close_theme_browser(&mut self) {
        self.theme_browser_mode = false;
        self.theme_browser.preview_theme = None;
        self.theme_browser.search_mode = false;
        self.theme_browser.search_query.clear();
        self.theme_browser.filtered_themes.clear();
        self.theme_browser.filtered_selected = None;
        self.preview_state = PreviewState::None;

        // Complete theme restoration
        self.theme_applicator.clear_theme();
    }

    async fn load_and_apply_theme(&mut self, theme_entry: &ThemeEntry) -> Result<()> {
        // Set loading state
        self.preview_state = PreviewState::Loading;

        // Load theme colors lazily
        match self.client.fetch_theme_colors(theme_entry).await {
            Ok(theme) => {
                // Apply theme globally
                self.theme_applicator.apply_theme(theme.clone());
                self.preview_state = PreviewState::Applied(Box::new(theme.clone()));
                self.theme_browser.preview_theme = Some(theme);
            }
            Err(_) => {
                self.preview_state = PreviewState::Error;
            }
        }

        Ok(())
    }

    async fn handle_theme_browser_input(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyModifiers;

        if self.theme_browser.search_mode {
            self.handle_theme_search_input(key).await
        } else {
            match key.code {
                KeyCode::Esc => {
                    self.close_theme_browser();
                }
                KeyCode::Char('/') => {
                    // Enter search mode
                    self.theme_browser.search_mode = true;
                    self.theme_browser.search_query.clear();
                    self.update_theme_search_filter();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.theme_browser_navigate_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.theme_browser_navigate_previous();
                }
                KeyCode::Enter => {
                    self.theme_browser_apply_selected().await?;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.quit = true;
                }
                _ => {}
            }
            Ok(())
        }
    }

    async fn handle_theme_search_input(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyModifiers;

        match key.code {
            KeyCode::Esc => {
                // Clear search and return to normal browse mode
                self.theme_browser.search_mode = false;
                self.theme_browser.search_query.clear();
                self.theme_browser.filtered_themes.clear();
                self.theme_browser.filtered_selected = None;
                self.update_theme_search_filter();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.theme_search_navigate_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.theme_search_navigate_previous();
            }
            KeyCode::Enter => {
                self.theme_browser_apply_selected().await?;
            }
            KeyCode::Backspace => {
                self.theme_browser.search_query.pop();
                self.update_theme_search_filter();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit = true;
            }
            KeyCode::Char(c) => {
                self.theme_browser.search_query.push(c);
                self.update_theme_search_filter();
            }
            _ => {}
        }
        Ok(())
    }

    fn update_theme_search_filter(&mut self) {
        if self.theme_browser.search_query.trim().is_empty() {
            // No search query - show all themes
            self.theme_browser.filtered_themes.clear();
            self.theme_browser.filtered_selected = None;
        } else {
            // Filter themes by name (case-insensitive)
            let query = self.theme_browser.search_query.to_lowercase();

            self.theme_browser.filtered_themes = self
                .theme_entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect();

            // Set selection to first result if we have any
            self.theme_browser.filtered_selected = if self.theme_browser.filtered_themes.is_empty()
            {
                None
            } else {
                Some(0)
            };
        }
    }

    fn theme_browser_navigate_next(&mut self) {
        if !self.theme_entries.is_empty() {
            self.theme_browser.selected_index = Some(match self.theme_browser.selected_index {
                Some(idx) if idx + 1 < self.theme_entries.len() => idx + 1,
                Some(_) => 0, // Wrap to beginning
                None => 0,
            });
        }
    }

    fn theme_browser_navigate_previous(&mut self) {
        if !self.theme_entries.is_empty() {
            self.theme_browser.selected_index = Some(match self.theme_browser.selected_index {
                Some(idx) if idx > 0 => idx - 1,
                Some(_) => self.theme_entries.len() - 1, // Wrap to end
                None => self.theme_entries.len() - 1,
            });
        }
    }

    fn theme_search_navigate_next(&mut self) {
        if !self.theme_browser.filtered_themes.is_empty() {
            self.theme_browser.filtered_selected =
                Some(match self.theme_browser.filtered_selected {
                    Some(idx) if idx + 1 < self.theme_browser.filtered_themes.len() => idx + 1,
                    Some(_) => 0, // Wrap to beginning
                    None => 0,
                });
        }
    }

    fn theme_search_navigate_previous(&mut self) {
        if !self.theme_browser.filtered_themes.is_empty() {
            self.theme_browser.filtered_selected =
                Some(match self.theme_browser.filtered_selected {
                    Some(idx) if idx > 0 => idx - 1,
                    Some(_) => self.theme_browser.filtered_themes.len() - 1, // Wrap to end
                    None => self.theme_browser.filtered_themes.len() - 1,
                });
        }
    }

    async fn theme_browser_apply_selected(&mut self) -> Result<()> {
        let selected_theme_index =
            if self.theme_browser.search_mode && !self.theme_browser.filtered_themes.is_empty() {
                // In search mode - get the actual theme index from filtered results
                if let Some(filtered_idx) = self.theme_browser.filtered_selected {
                    self.theme_browser
                        .filtered_themes
                        .get(filtered_idx)
                        .copied()
                } else {
                    None
                }
            } else {
                // Normal mode - use direct selection
                self.theme_browser.selected_index
            };

        if let Some(theme_idx) = selected_theme_index {
            if let Some(theme_entry) = self.theme_entries.get(theme_idx).cloned() {
                self.load_and_apply_theme(&theme_entry).await?;
            }
        }
        Ok(())
    }

    /// Check if a theme is currently applied
    pub fn is_theme_applied(&self) -> bool {
        self.theme_applicator.is_applied
    }
}
