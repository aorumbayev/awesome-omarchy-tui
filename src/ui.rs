use crate::{
    App,
    models::{AppState, FocusArea},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content area (sidebar + content)
            Constraint::Length(2), // Footer
        ])
        .split(f.size());

    draw_header(f, main_chunks[0], app);

    // Split the main content area horizontally for sidebar and content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left sidebar
            Constraint::Percentage(70), // Right content area
        ])
        .split(main_chunks[1]);

    draw_sidebar(f, content_chunks[0], app);
    draw_main_content(f, content_chunks[1], app);
    draw_footer(f, main_chunks[2], app);

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

    // Premium main title with enhanced typography
    let title = Paragraph::new(Line::from(vec![
        Span::styled("✨ ", Style::default().fg(Color::Yellow)),
        Span::styled(
            "Awesome Omarchy",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ✨", Style::default().fg(Color::Yellow)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let meta_style = if app.state.is_loading() {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(Color::Gray)
    };

    let meta = Paragraph::new(Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled(title_text, meta_style),
        Span::styled(" │", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(meta, chunks[1]);
}

fn draw_sidebar(f: &mut Frame, area: Rect, app: &mut App) {
    match &app.state {
        AppState::Loading => {
            let loading = Paragraph::new("🔄 Loading...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("📂 Sections")
                        .border_style(Style::default().fg(Color::Blue)),
                );
            f.render_widget(loading, area);
        }
        AppState::Ready => {
            if !app.tabs.is_empty() {
                // Create sidebar items with section names and entry counts
                let items: Vec<ListItem> = app
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(i, tab)| {
                        let is_selected = i == app.current_tab;
                        let entry_count = if let Some(ref readme) = app.readme_content {
                            readme
                                .sections
                                .get(tab.section_index)
                                .map(|section| section.entries.len())
                                .unwrap_or(0)
                        } else {
                            0
                        };

                        let _content_text = format!("{} ({})", tab.title, entry_count);

                        if is_selected {
                            ListItem::new(Line::from(vec![
                                Span::styled(
                                    "▶ ",
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    &tab.title,
                                    Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(" ", Style::default()),
                                Span::styled(
                                    format!("({})", entry_count),
                                    Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                                ),
                            ]))
                        } else {
                            ListItem::new(Line::from(vec![
                                Span::styled("  ", Style::default()),
                                Span::styled(&tab.title, Style::default().fg(Color::Gray)),
                                Span::styled(" ", Style::default()),
                                Span::styled(
                                    format!("({})", entry_count),
                                    Style::default().fg(Color::DarkGray),
                                ),
                            ]))
                        }
                    })
                    .collect();

                // Determine border style based on focus
                let border_style = if app.focus_area == FocusArea::Sidebar {
                    Style::default().fg(Color::Green) // Focused
                } else {
                    Style::default().fg(Color::Blue) // Not focused
                };

                let sidebar_list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("📂 Sections")
                            .border_style(border_style),
                    )
                    .style(Style::default())
                    .highlight_style(Style::default())
                    .highlight_symbol("")
                    .direction(ratatui::widgets::ListDirection::TopToBottom);

                f.render_widget(sidebar_list, area);
            } else {
                let empty = Paragraph::new("No sections available")
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("📂 Sections")
                            .border_style(Style::default().fg(Color::Yellow)),
                    );
                f.render_widget(empty, area);
            }
        }
        AppState::Error(error) => {
            let error_text = Paragraph::new(format!("❌ Error: {}", error))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("📂 Sections")
                        .border_style(Style::default().fg(Color::Red)),
                );
            f.render_widget(error_text, area);
        }
    }
}

fn draw_main_content(f: &mut Frame, area: Rect, app: &mut App) {
    match &app.state {
        AppState::Loading => {
            draw_loading(f, area);
        }
        AppState::Ready => {
            draw_repository_content(f, area, app);
        }
        AppState::Error(error) => {
            draw_error(f, area, error);
        }
    }
}

fn draw_repository_content(f: &mut Frame, area: Rect, app: &mut App) {
    // This is the main content rendering logic, extracted from draw_tabs_and_content
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
            readme
                .sections
                .get(section_index)
                .map(|section| (&section.entries, section.raw_content.clone()))
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
                        let is_selected = selected_index == Some(idx);

                        // Create elegant formatted list item with enhanced typography
                        let title_line = if is_selected {
                            // Elegant selected item with subtle highlight and refined borders
                            Line::from(vec![
                                Span::styled(
                                    "▎",
                                    Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{:2}. ", idx + 1),
                                    Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                                ),
                                Span::styled("● ", Style::default().fg(Color::Green)),
                                Span::styled(
                                    &entry.title,
                                    Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ])
                        } else {
                            Line::from(vec![
                                Span::styled("  ", Style::default()),
                                Span::styled(
                                    format!("{:2}. ", idx + 1),
                                    Style::default().fg(Color::DarkGray),
                                ),
                                Span::styled("○ ", Style::default().fg(Color::Gray)),
                                Span::styled(&entry.title, Style::default().fg(Color::White)),
                            ])
                        };

                        let mut lines = vec![title_line];

                        // Add description with enhanced typography and spacing
                        if !entry.description.is_empty() {
                            let desc_line = if is_selected {
                                Line::from(vec![
                                    Span::styled(
                                        "▎",
                                        Style::default()
                                            .fg(Color::Cyan)
                                            .add_modifier(Modifier::BOLD),
                                    ),
                                    Span::styled("    ", Style::default()),
                                    Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(
                                        &entry.description,
                                        Style::default()
                                            .fg(Color::Gray)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ])
                            } else {
                                Line::from(vec![
                                    Span::styled("      ", Style::default()),
                                    Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(
                                        &entry.description,
                                        Style::default()
                                            .fg(Color::DarkGray)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ])
                            };
                            lines.push(desc_line);
                        }

                        // Add tags with refined presentation
                        if !entry.tags.is_empty() {
                            let tags_text = entry.tags.join(" • ");
                            let tags_line = if is_selected {
                                Line::from(vec![
                                    Span::styled(
                                        "▎",
                                        Style::default()
                                            .fg(Color::Cyan)
                                            .add_modifier(Modifier::BOLD),
                                    ),
                                    Span::styled("    ", Style::default()),
                                    Span::styled("🏷 ", Style::default().fg(Color::Yellow)),
                                    Span::styled(
                                        tags_text,
                                        Style::default()
                                            .fg(Color::Yellow)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ])
                            } else {
                                Line::from(vec![
                                    Span::styled("      ", Style::default()),
                                    Span::styled("🏷 ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(
                                        tags_text,
                                        Style::default()
                                            .fg(Color::DarkGray)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ])
                            };
                            lines.push(tags_line);
                        }

                        // Add URL with premium styling
                        let url_line = if is_selected {
                            Line::from(vec![
                                Span::styled(
                                    "▎",
                                    Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled("    ", Style::default()),
                                Span::styled("🔗 ", Style::default().fg(Color::Blue)),
                                Span::styled(
                                    &entry.url,
                                    Style::default()
                                        .fg(Color::Blue)
                                        .add_modifier(Modifier::UNDERLINED),
                                ),
                            ])
                        } else {
                            Line::from(vec![
                                Span::styled("      ", Style::default()),
                                Span::styled("🔗 ", Style::default().fg(Color::DarkGray)),
                                Span::styled(&entry.url, Style::default().fg(Color::Blue)),
                            ])
                        };
                        lines.push(url_line);

                        // Add subtle separator for selected items
                        if is_selected {
                            lines.push(Line::from(vec![Span::styled(
                                "▎",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            )]));
                        }

                        // Add breathing room between entries
                        lines.push(Line::from(""));

                        ListItem::new(lines)
                    })
                    .collect();

                // Create ratatui ListState with the selected index
                let mut ratatui_state = ListState::default();
                ratatui_state.select(selected_index);

                // Determine border style based on focus
                let border_style = if app.focus_area == FocusArea::Content {
                    Style::default().fg(Color::Green) // Focused
                } else {
                    Style::default().fg(Color::Blue) // Not focused
                };

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("📋 {} ({} entries)", section_title, entry_count))
                            .border_style(border_style),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default())
                    .highlight_symbol("")
                    .direction(ratatui::widgets::ListDirection::TopToBottom);

                f.render_stateful_widget(list, area, &mut ratatui_state);
            } else {
                // No entries - show raw content or empty section
                if !raw_content.trim().is_empty()
                    && !raw_content.starts_with("•")
                    && !raw_content.contains("URL:")
                {
                    // Show raw content for sections without structured entries
                    let border_style = if app.focus_area == FocusArea::Content {
                        Style::default().fg(Color::Green) // Focused
                    } else {
                        Style::default().fg(Color::Blue) // Not focused
                    };

                    let content = Paragraph::new(raw_content.clone())
                        .wrap(Wrap { trim: true })
                        .scroll((scroll_offset as u16, 0))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(format!("📄 {}", section_title))
                                .border_style(border_style),
                        );
                    f.render_widget(content, area);
                } else {
                    draw_empty_section(f, area, &section_title, &app.focus_area);
                }
            }
        } else {
            // Section not found - this should not happen but handle gracefully
            draw_empty_section(f, area, "Unknown Section", &app.focus_area);
        }
    } else {
        // No current tab
        draw_empty_section(f, area, "No Content", &app.focus_area);
    }
}

fn draw_loading(f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(5),
            Constraint::Percentage(40),
        ])
        .split(area);

    let loading_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    let loading = Paragraph::new(Line::from(vec![
        Span::styled(
            "⏳ ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(
            "Loading README content...\n\n",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "✨ Fetching awesome resources from GitHub ✨",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled("📡 ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Loading",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(loading, loading_chunks[1]);
}

fn draw_error(f: &mut Frame, area: Rect, error: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(7),
            Constraint::Percentage(35),
        ])
        .split(area);

    let error_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(chunks[1]);

    let error_text = Paragraph::new(Line::from(vec![
        Span::styled(
            "❌ ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Error Occurred\n\n",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled(error, Style::default().fg(Color::Gray)),
        Span::styled("\n\n", Style::default()),
        Span::styled("💡 Press ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "'R'",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to retry", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled("⚠️  ", Style::default().fg(Color::Red)),
                Span::styled(
                    "Error",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]))
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(error_text, error_chunks[1]);
}

fn draw_empty_section(f: &mut Frame, area: Rect, section_title: &str, focus_area: &FocusArea) {
    let border_style = if *focus_area == FocusArea::Content {
        Style::default().fg(Color::Green) // Focused
    } else {
        Style::default().fg(Color::Blue) // Not focused
    };

    let empty_message = Paragraph::new(Line::from(vec![
        Span::styled("📭 ", Style::default().fg(Color::Gray)),
        Span::styled(
            "This section contains no repository entries.\n\n",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("💡 Try:\n", Style::default().fg(Color::DarkGray)),
        Span::styled("• Press ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "'R' ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("to reload content\n", Style::default().fg(Color::DarkGray)),
        Span::styled("• Use ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "'h/l' ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "to explore other sections",
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled("📄 ", Style::default().fg(Color::Gray)),
                Span::styled(
                    section_title,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .border_style(border_style),
    );
    f.render_widget(empty_message, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = if app.search_mode {
        Line::from(vec![
            Span::styled(
                "ESC",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Exit search │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "j/k",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Navigate │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                ": Open URL │ Type to search...",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "h/l",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Switch sections │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "j/k",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Navigate │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Next │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "R",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Reload │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "G",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": GitHub │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "/",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Search │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Quit", Style::default().fg(Color::DarkGray)),
        ])
    };

    let footer = Paragraph::new(help_text).alignment(Alignment::Center);
    f.render_widget(footer, area);
}

fn draw_search_popup(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(80, 60, f.size());

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(popup_area);

    // Enhanced search input with premium styling
    let cursor_indicator = if app.search_query.is_empty() {
        "Type to search..."
    } else {
        ""
    };
    let search_text = if app.search_query.is_empty() && cursor_indicator == "Type to search..." {
        Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                cursor_indicator,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.search_query, Style::default().fg(Color::Yellow)),
            Span::styled(
                "▌",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::SLOW_BLINK),
            ), // Blinking cursor
        ])
    };

    let search_input = Paragraph::new(search_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled("🔍 ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Search",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(search_input, chunks[0]);

    // Enhanced search results display
    if !app.search_results.is_empty() {
        let results: Vec<ListItem> = app
            .search_results
            .iter()
            .take(20) // Limit to top 20 results
            .enumerate()
            .map(|(i, result)| {
                if let Some(ref readme) = app.readme_content {
                    if let Some(section) = readme.sections.get(result.section_index) {
                        // Premium format with enhanced typography
                        if let Some(entry_idx) = result.entry_index
                            && let Some(entry) = section.entries.get(entry_idx)
                        {
                            let truncated_desc = if entry.description.is_empty() {
                                "No description".to_string()
                            } else {
                                let desc_chars: String =
                                    entry.description.chars().take(50).collect();
                                if entry.description.len() > 50 {
                                    format!("{}...", desc_chars)
                                } else {
                                    desc_chars
                                }
                            };

                            let lines = vec![
                                Line::from(vec![
                                    Span::styled(
                                        format!("{:2}. ", i + 1),
                                        Style::default()
                                            .fg(Color::Cyan)
                                            .add_modifier(Modifier::DIM),
                                    ),
                                    Span::styled("● ", Style::default().fg(Color::Green)),
                                    Span::styled(
                                        entry.title.clone(),
                                        Style::default()
                                            .fg(Color::White)
                                            .add_modifier(Modifier::BOLD),
                                    ),
                                ]),
                                Line::from(vec![
                                    Span::styled("     ", Style::default()),
                                    Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(
                                        truncated_desc,
                                        Style::default()
                                            .fg(Color::Gray)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ]),
                                Line::from(vec![
                                    Span::styled("     ", Style::default()),
                                    Span::styled("└─ ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(
                                        section.title.clone(),
                                        Style::default().fg(Color::Blue),
                                    ),
                                ]),
                            ];
                            return ListItem::new(lines);
                        }

                        // Fallback formatting
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{:2}. ", i + 1),
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                            ),
                            Span::styled("[", Style::default().fg(Color::DarkGray)),
                            Span::styled(section.title.clone(), Style::default().fg(Color::Blue)),
                            Span::styled("] ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                result.line_content.chars().take(60).collect::<String>(),
                                Style::default().fg(Color::White),
                            ),
                        ]))
                    } else {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{:2}. ", i + 1),
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                            ),
                            Span::styled("Result", Style::default().fg(Color::White)),
                        ]))
                    }
                } else {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:2}. ", i + 1),
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
                        ),
                        Span::styled("Result", Style::default().fg(Color::White)),
                    ]))
                }
            })
            .collect();

        // Create a ListState for the search results
        let mut search_state = ratatui::widgets::ListState::default();
        search_state.select(app.search_selection);

        let results_list = List::new(results)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(vec![
                        Span::styled("📋 ", Style::default().fg(Color::Green)),
                        Span::styled(
                            "Results ",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!(
                                "({}/{})",
                                app.search_results.len().min(20),
                                app.search_results.len()
                            ),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::DIM),
                        ),
                    ]))
                    .border_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default())
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        f.render_stateful_widget(results_list, chunks[1], &mut search_state);
    } else if !app.search_query.is_empty() {
        let no_results = Paragraph::new(Line::from(vec![
            Span::styled(
                "😔 No results found\n\n",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Try searching for:\n", Style::default().fg(Color::Gray)),
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled("Section names ", Style::default().fg(Color::Blue)),
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled("Repository titles ", Style::default().fg(Color::Green)),
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled("Descriptions ", Style::default().fg(Color::Yellow)),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled("📋 ", Style::default().fg(Color::Red)),
                    Span::styled(
                        "Results (0)",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]))
                .border_style(Style::default().fg(Color::Red)),
        );
        f.render_widget(no_results, chunks[1]);
    } else {
        let help_text = Paragraph::new(Line::from(vec![
            Span::styled(
                "✨ Start typing to search...\n\n",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("🔍 Search features:\n", Style::default().fg(Color::Gray)),
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled("Repository names ", Style::default().fg(Color::Green)),
            Span::styled(
                "(highest priority)\n",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled("Descriptions ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "(medium priority)\n",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "j/k ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("to navigate • ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Enter ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("to open URLs", Style::default().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled("💡 ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        "Search Help",
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
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
