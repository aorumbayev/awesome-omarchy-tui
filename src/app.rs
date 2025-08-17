use crate::{
    HttpClient,
    models::{AppState, FocusArea, ListState, ReadmeContent, SearchResult, TabState},
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
        if self.search_mode {
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
                        Some(_) => 0, // Wrap to beginning
                        None => 0,
                    });
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Navigate up in search results
                if !self.search_results.is_empty() {
                    self.search_selection = Some(match self.search_selection {
                        Some(idx) if idx > 0 => idx - 1,
                        Some(_) => self.search_results.len() - 1, // Wrap to end
                        None => self.search_results.len() - 1,
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
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.load_readme(true).await?;
            }
            // Open GitHub
            KeyCode::Char('g') | KeyCode::Char('G') => {
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
            // Quit
            KeyCode::Char('q') | KeyCode::Char('Q') => {
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
        if let Some(ref readme) = self.readme_content {
            if !self.search_query.trim().is_empty() {
                self.search_results = readme.search_index.search(&self.search_query);
                // Reset selection when performing new search
                self.search_selection = if self.search_results.is_empty() {
                    None
                } else {
                    Some(0)
                };
            } else {
                self.search_results.clear();
                self.search_selection = None;
            }
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

    pub fn get_current_list_state(&mut self) -> Option<&mut crate::models::ListState> {
        // Ensure we get the list state for the current tab
        if self.current_tab < self.tabs.len() {
            Some(&mut self.tabs[self.current_tab].list_state)
        } else {
            None
        }
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
}
