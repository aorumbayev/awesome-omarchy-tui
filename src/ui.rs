use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap, List, ListItem, ListState},
    Frame,
};
use crate::{App, models::AppState};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(2), // Footer
        ])
        .split(f.size());

    draw_header(f, chunks[0], app);
    draw_content(f, chunks[1], app);
    draw_footer(f, chunks[2], app);

    if app.search_mode {
        draw_search_popup(f, app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let title_text = if let Some(summary) = app.get_metadata_summary() {
        summary
    } else {
        "Loading...".to_string()
    };
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    
    // Main title
    let title = Paragraph::new("🌟 Awesome Omarchy")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);
    
    // Metadata info
    let meta = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(meta, chunks[1]);
}

fn draw_content(f: &mut Frame, area: Rect, app: &mut App) {
    match &app.state {
        AppState::Loading => {
            draw_loading(f, area);
        }
        AppState::Ready => {
            draw_tabs_and_content(f, area, app);
        }
        AppState::Error(error) => {
            draw_error(f, area, error);
        }
    }
}

fn draw_loading(f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(40),
        ])
        .split(area);
        
    let loading_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    let loading = Paragraph::new("🔄 Loading README content...")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status")
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(loading, loading_chunks[1]);
}

fn draw_error(f: &mut Frame, area: Rect, error: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(5),
            Constraint::Percentage(30),
        ])
        .split(area);
        
    let error_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(chunks[1]);

    let error_text = Paragraph::new(format!("❌ Error: {}\n\nPress 'R' to retry", error))
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .border_style(Style::default().fg(Color::Red)),
        );
    f.render_widget(error_text, error_chunks[1]);
}

fn draw_tabs_and_content(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Draw tabs
    if !app.tabs.is_empty() {
        let tab_titles: Vec<Line> = app
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                if i == app.current_tab {
                    Line::from(vec![
                        Span::styled("● ", Style::default().fg(Color::Green)),
                        Span::styled(&tab.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("○ ", Style::default().fg(Color::Gray)),
                        Span::styled(&tab.title, Style::default().fg(Color::Gray)),
                    ])
                }
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📂 Sections")
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .select(app.current_tab)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::Blue));
        f.render_widget(tabs, chunks[0]);
    }

    // Draw content as List - Fixed borrowing and section-specific rendering
    if let Some(current_tab) = app.current_tab() {
        let section_title = current_tab.title.clone();
        let section_index = current_tab.section_index;
        let scroll_offset = current_tab.scroll_offset;
        
        // Get list state first (mutable borrow)
        let selected_index = if let Some(list_state) = app.get_current_list_state() {
            list_state.selected_index
        } else {
            None
        };
        
        // Collect section-specific data (immutable borrow, separate from above)
        let section_data = if let Some(ref readme) = app.readme_content {
            readme.sections.get(section_index).map(|section| (&section.entries, section.raw_content.clone()))
        } else {
            None
        };
        
        if let Some((entries, raw_content)) = section_data {
            let has_entries = !entries.is_empty();
            let entry_count = entries.len();
            
            if has_entries {
                // Create List items from repository entries with enumeration - Fixed data access
                let items: Vec<ListItem> = entries
                    .iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        // Create beautiful formatted list item with enumeration
                        let title_line = Line::from(vec![
                            Span::styled(format!("{}. ▶ ", idx + 1), Style::default().fg(Color::Cyan)),
                            Span::styled(&entry.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                        ]);
                        
                        let mut lines = vec![title_line];
                        
                        // Add description if available
                        if !entry.description.is_empty() {
                            let desc_line = Line::from(vec![
                                Span::styled("     ", Style::default()), // Indent to align with title
                                Span::styled(&entry.description, Style::default().fg(Color::Gray))
                            ]);
                            lines.push(desc_line);
                        }
                        
                        // Add tags if available
                        if !entry.tags.is_empty() {
                            let tags_text = format!("     🏷  {}", entry.tags.join(", "));
                            let tags_line = Line::from(vec![
                                Span::styled(tags_text, Style::default().fg(Color::Yellow))
                            ]);
                            lines.push(tags_line);
                        }
                        
                        // Add URL
                        let url_line = Line::from(vec![
                            Span::styled("     🔗 ", Style::default().fg(Color::Blue)),
                            Span::styled(&entry.url, Style::default().fg(Color::Blue))
                        ]);
                        lines.push(url_line);
                        
                        // Add separator
                        lines.push(Line::from(""));
                        
                        ListItem::new(lines)
                    })
                    .collect();

                // Create ratatui ListState with the selected index
                let mut ratatui_state = ListState::default();
                ratatui_state.select(selected_index);

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("📋 {} ({} entries)", section_title, entry_count))
                            .border_style(Style::default().fg(Color::Green)),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    .highlight_symbol("▸ ")
                    .direction(ratatui::widgets::ListDirection::TopToBottom);
                    
                f.render_stateful_widget(list, chunks[1], &mut ratatui_state);
            } else {
                // No entries - show raw content or empty section
                if !raw_content.trim().is_empty() && 
                   !raw_content.starts_with("•") && 
                   !raw_content.contains("URL:") {
                    // Show raw content for sections without structured entries
                    let content = Paragraph::new(raw_content.clone())
                        .wrap(Wrap { trim: true })
                        .scroll((scroll_offset as u16, 0))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(format!("📄 {}", section_title))
                                .border_style(Style::default().fg(Color::Green)),
                        );
                    f.render_widget(content, chunks[1]);
                } else {
                    draw_empty_section(f, chunks[1], &section_title);
                }
            }
        } else {
            // Section not found - this should not happen but handle gracefully
            draw_empty_section(f, chunks[1], "Unknown Section");
        }
    } else {
        // No current tab
        draw_empty_section(f, chunks[1], "No Content");
    }
}

fn draw_empty_section(f: &mut Frame, area: Rect, section_title: &str) {
    let empty_message = Paragraph::new("📭 This section contains no repository entries.\n\nPress 'R' to reload content or try another section.")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("📄 {}", section_title))
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(empty_message, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = if app.search_mode {
        "ESC: Exit search | j/k: Navigate results | Enter: Open URL | Type to search..."
    } else {
        "h/l,Tab: Switch sections | j/k: Navigate entries | R: Reload | G: GitHub | /: Search | Q: Quit"
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(footer, area);
}

fn draw_search_popup(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(80, 60, f.size());
    
    f.render_widget(Clear, popup_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(popup_area);
    
    // Search input with cursor indicator
    let cursor_indicator = if app.search_query.is_empty() { "Type to search..." } else { "" };
    let search_text = if app.search_query.is_empty() && cursor_indicator == "Type to search..." {
        cursor_indicator.to_string()
    } else {
        format!("{}█", app.search_query) // █ as cursor
    };
    
    let input_style = if app.search_query.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Yellow)
    };
    
    let search_input = Paragraph::new(search_text)
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔍 Search")
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(search_input, chunks[0]);
    
    // Search results
    if !app.search_results.is_empty() {
        let results: Vec<ListItem> = app.search_results
            .iter()
            .take(20) // Limit to top 20 results
            .enumerate()
            .map(|(i, result)| {
                let content = if let Some(ref readme) = app.readme_content {
                    if let Some(section) = readme.sections.get(result.section_index) {
                        // Format: "1. [Section Name] Repository Title - Description"
                        if let Some(entry_idx) = result.entry_index {
                            if let Some(entry) = section.entries.get(entry_idx) {
                                format!("{}. [{}] {} - {}", 
                                    i + 1, 
                                    section.title, 
                                    entry.title,
                                    if entry.description.is_empty() {
                                        "No description".to_string()
                                    } else {
                                        entry.description.chars().take(60).collect::<String>() + 
                                        if entry.description.len() > 60 { "..." } else { "" }
                                    }
                                )
                            } else {
                                format!("{}. [{}] {}", i + 1, section.title, result.line_content.chars().take(80).collect::<String>())
                            }
                        } else {
                            format!("{}. [{}] {}", i + 1, section.title, result.line_content.chars().take(80).collect::<String>())
                        }
                    } else {
                        format!("{}. Result", i + 1)
                    }
                } else {
                    format!("{}. Result", i + 1)
                };
                
                ListItem::new(content)
                    .style(Style::default().fg(Color::White))
            })
            .collect();

        // Create a ListState for the search results
        let mut search_state = ratatui::widgets::ListState::default();
        search_state.select(app.search_selection);
            
        let results_list = List::new(results)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("📋 Results ({}/{})", 
                        app.search_results.len().min(20), 
                        app.search_results.len()))
                    .border_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");
        f.render_stateful_widget(results_list, chunks[1], &mut search_state);
    } else if !app.search_query.is_empty() {
        let no_results = Paragraph::new("No results found\n\nTry searching for:\n• Section names\n• Repository titles\n• Descriptions\n• Tags")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 Results (0)")
                    .border_style(Style::default().fg(Color::Red)),
            );
        f.render_widget(no_results, chunks[1]);
    } else {
        let help_text = Paragraph::new("Start typing to search repository names and descriptions...\n\n🔍 Search features:\n• Repository names (highest priority)\n• Repository descriptions (medium priority)\n• j/k keys to navigate results\n• Enter to open GitHub URL\n• Case insensitive matching")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 Search Help")
                    .border_style(Style::default().fg(Color::Blue)),
            );
        f.render_widget(help_text, chunks[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
