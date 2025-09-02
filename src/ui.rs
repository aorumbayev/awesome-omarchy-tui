use crate::{
    App,
    models::{AppState, FocusArea},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

/// Dynamic color scheme that adapts to applied themes
pub struct ThemeColors {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
    pub highlight: Color,

    pub border_focused: Color,
    pub border_normal: Color,
}

impl ThemeColors {
    pub fn new() -> Self {
        Self {
            // Use ANSI colors for full terminal theme compatibility
            background: Color::Indexed(0), // ANSI black - adapts to terminal theme
            foreground: Color::Indexed(15), // ANSI bright white - adapts to terminal theme
            primary: Color::Indexed(4),    // ANSI blue - adapts to terminal theme
            secondary: Color::Indexed(6),  // ANSI cyan - adapts to terminal theme
            accent: Color::Indexed(12),    // ANSI bright blue - adapts to terminal theme
            success: Color::Indexed(2),    // ANSI green - adapts to terminal theme
            warning: Color::Indexed(3),    // ANSI yellow - adapts to terminal theme
            error: Color::Indexed(1),      // ANSI red - adapts to terminal theme
            muted: Color::Indexed(8), // ANSI bright black (dark gray) - adapts to terminal theme
            highlight: Color::Indexed(10), // ANSI bright green - adapts to terminal theme

            border_focused: Color::Indexed(2), // ANSI green for focused borders
            border_normal: Color::Indexed(8),  // ANSI bright black (muted) for normal borders
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    // Use default theme colors for main UI - themes only affect preview panels
    let default_theme_colors = ThemeColors::default();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content area (sidebar + content)
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    draw_header(f, main_chunks[0], app, &default_theme_colors);

    // Split the main content area horizontally for sidebar and content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left sidebar
            Constraint::Percentage(80), // Right content area
        ])
        .split(main_chunks[1]);

    draw_sidebar(f, content_chunks[0], app, &default_theme_colors);
    draw_main_content(f, content_chunks[1], app, &default_theme_colors);
    draw_footer(f, main_chunks[2], app, &default_theme_colors);

    if app.theme_browser_mode {
        draw_theme_browser_popup(f, app, &default_theme_colors);
    } else if app.search_mode {
        draw_search_popup(f, app, &default_theme_colors);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let title_text = if let Some(summary) = app.get_metadata_summary() {
        summary
    } else {
        "Loading...".to_string()
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Enhanced main title with premium typography and visual flair
    let title = Paragraph::new(Line::from(vec![
        Span::styled("✨ ", Style::default().fg(theme.warning)),
        Span::styled(
            "┌",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Awesome Omarchy ",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "┐",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ✨", Style::default().fg(theme.warning)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let meta_style = if app.state.is_loading() {
        Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(theme.muted)
    };

    let meta = Paragraph::new(Line::from(vec![
        Span::styled("│ ", Style::default().fg(theme.accent)),
        Span::styled(title_text, meta_style),
        Span::styled(" │", Style::default().fg(theme.accent)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(meta, chunks[1]);
}

fn draw_sidebar(f: &mut Frame, area: Rect, app: &mut App, theme: &ThemeColors) {
    match &app.state {
        AppState::Loading => {
            let loading = Paragraph::new("🔄 Loading...")
                .style(Style::default().fg(theme.warning))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("📂 Sections")
                        .border_style(Style::default().fg(theme.primary)),
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

                        if is_selected {
                            ListItem::new(Line::from(vec![
                                Span::styled(
                                    "◆ ",
                                    Style::default()
                                        .fg(theme.highlight)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    &tab.title,
                                    Style::default()
                                        .fg(theme.foreground)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(" ", Style::default()),
                                Span::styled("[", Style::default().fg(theme.accent)),
                                Span::styled(
                                    entry_count.to_string(),
                                    Style::default()
                                        .fg(theme.accent)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled("]", Style::default().fg(theme.accent)),
                            ]))
                        } else {
                            ListItem::new(Line::from(vec![
                                Span::styled("◇ ", Style::default().fg(theme.muted)),
                                Span::styled(
                                    &tab.title,
                                    Style::default()
                                        .fg(theme.foreground)
                                        .add_modifier(Modifier::DIM),
                                ),
                                Span::styled(" ", Style::default()),
                                Span::styled("[", Style::default().fg(theme.muted)),
                                Span::styled(
                                    entry_count.to_string(),
                                    Style::default().fg(theme.muted),
                                ),
                                Span::styled("]", Style::default().fg(theme.muted)),
                            ]))
                        }
                    })
                    .collect();

                // Enhanced border style based on focus
                let border_style = if app.focus_area == FocusArea::Sidebar {
                    Style::default().fg(theme.border_focused) // Focused
                } else {
                    Style::default().fg(theme.border_normal) // Not focused
                };

                let sidebar_list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(Line::from(vec![
                                Span::styled("📂 ", Style::default().fg(theme.primary)),
                                Span::styled(
                                    "Sections",
                                    Style::default()
                                        .fg(theme.foreground)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ]))
                            .border_style(border_style),
                    )
                    .style(Style::default())
                    .highlight_style(Style::default())
                    .highlight_symbol("")
                    .direction(ratatui::widgets::ListDirection::TopToBottom);

                f.render_widget(sidebar_list, area);
            } else {
                let empty = Paragraph::new("No sections available")
                    .style(Style::default().fg(theme.muted))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("📂 Sections")
                            .border_style(Style::default().fg(theme.warning)),
                    );
                f.render_widget(empty, area);
            }
        }
        AppState::Error(error) => {
            let error_text = Paragraph::new(format!("❌ Error: {error}"))
                .style(Style::default().fg(theme.error))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("📂 Sections")
                        .border_style(Style::default().fg(theme.error)),
                );
            f.render_widget(error_text, area);
        }
    }
}

fn draw_main_content(f: &mut Frame, area: Rect, app: &mut App, theme: &ThemeColors) {
    match &app.state {
        AppState::Loading => {
            draw_loading(f, area, theme);
        }
        AppState::Ready => {
            draw_repository_content(f, area, app, theme);
        }
        AppState::Error(error) => {
            draw_error(f, area, error, theme);
        }
    }
}

fn draw_repository_content(f: &mut Frame, area: Rect, app: &mut App, theme: &ThemeColors) {
    use ratatui::widgets::ListState;

    if let Some(current_tab) = app.current_tab() {
        let section_title = current_tab.title.clone();
        let section_index = current_tab.section_index;

        // Get selected index from list state
        let selected_index = current_tab.list_state.selected_index;

        // Get section data
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
                // Create List items from repository entries
                let items: Vec<ListItem> = entries
                    .iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        let is_selected = selected_index == Some(idx);

                        // Create formatted list item with enhanced visual hierarchy
                        let title_line = if is_selected {
                            Line::from(vec![
                                Span::styled(
                                    format!("{:2}. ", idx + 1),
                                    Style::default().fg(theme.muted),
                                ),
                                Span::styled("◆ ", Style::default().fg(theme.highlight)),
                                Span::styled(
                                    &entry.title,
                                    Style::default()
                                        .fg(theme.foreground)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            ])
                        } else {
                            Line::from(vec![
                                Span::styled(
                                    format!("{:2}. ", idx + 1),
                                    Style::default().fg(theme.muted).add_modifier(Modifier::DIM),
                                ),
                                Span::styled("◇ ", Style::default().fg(theme.muted)),
                                Span::styled(
                                    &entry.title,
                                    Style::default()
                                        .fg(theme.foreground)
                                        .add_modifier(Modifier::DIM),
                                ),
                            ])
                        };

                        let mut lines = vec![title_line];

                        // Add enhanced description with markdown-style formatting
                        if !entry.description.is_empty() {
                            let formatted_desc = format_markdown_text(&entry.description);
                            let desc_line = if is_selected {
                                Line::from(vec![
                                    Span::styled("    ", Style::default()),
                                    Span::styled("┃ ", Style::default().fg(theme.accent)),
                                    Span::styled(
                                        formatted_desc,
                                        Style::default()
                                            .fg(theme.foreground)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ])
                            } else {
                                Line::from(vec![
                                    Span::styled("      ", Style::default()),
                                    Span::styled("┃ ", Style::default().fg(theme.muted)),
                                    Span::styled(
                                        formatted_desc,
                                        Style::default()
                                            .fg(theme.muted)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ])
                            };
                            lines.push(desc_line);
                        }

                        // Add enhanced tags with visual badges
                        if !entry.tags.is_empty() {
                            let tag_badges: Vec<Span> = entry
                                .tags
                                .iter()
                                .enumerate()
                                .flat_map(|(i, tag)| {
                                    let badge_color = get_tag_color(tag, theme);
                                    let mut spans = vec![
                                        Span::styled("[", Style::default().fg(badge_color)),
                                        Span::styled(
                                            tag,
                                            Style::default()
                                                .fg(badge_color)
                                                .add_modifier(Modifier::BOLD),
                                        ),
                                        Span::styled("]", Style::default().fg(badge_color)),
                                    ];
                                    if i < entry.tags.len() - 1 {
                                        spans.push(Span::styled(" ", Style::default()));
                                    }
                                    spans
                                })
                                .collect();

                            let mut tag_spans = if is_selected {
                                vec![
                                    Span::styled("    ", Style::default()),
                                    Span::styled("🏷️ ", Style::default().fg(theme.warning)),
                                ]
                            } else {
                                vec![
                                    Span::styled("      ", Style::default()),
                                    Span::styled("🏷️ ", Style::default().fg(theme.muted)),
                                ]
                            };
                            tag_spans.extend(tag_badges);
                            lines.push(Line::from(tag_spans));
                        }

                        // Add enhanced URL with better formatting
                        let formatted_url = format_github_url(&entry.url);
                        let url_line = if is_selected {
                            Line::from(vec![
                                Span::styled("    ", Style::default()),
                                Span::styled("🔗 ", Style::default().fg(theme.primary)),
                                Span::styled(
                                    formatted_url,
                                    Style::default()
                                        .fg(theme.primary)
                                        .add_modifier(Modifier::UNDERLINED),
                                ),
                            ])
                        } else {
                            Line::from(vec![
                                Span::styled("      ", Style::default()),
                                Span::styled("🔗 ", Style::default().fg(theme.muted)),
                                Span::styled(
                                    formatted_url,
                                    Style::default()
                                        .fg(theme.primary)
                                        .add_modifier(Modifier::DIM),
                                ),
                            ])
                        };
                        lines.push(url_line);

                        // Add enhanced separator and metadata for selected items
                        if is_selected {
                            // Add repository stats if available (placeholder for future enhancement)
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::default()),
                                Span::styled("└─", Style::default().fg(theme.accent)),
                                Span::styled(
                                    " GitHub Repository ",
                                    Style::default()
                                        .fg(theme.accent)
                                        .add_modifier(Modifier::DIM),
                                ),
                                Span::styled("⭐", Style::default().fg(theme.warning)),
                            ]));
                        }

                        // Add enhanced spacing between entries
                        lines.push(Line::from(Span::styled("", Style::default())));

                        ListItem::new(lines)
                    })
                    .collect();

                // Create ratatui ListState
                let mut ratatui_state = ListState::default();
                ratatui_state.select(selected_index);

                // Enhanced border style based on focus with better visual feedback
                let border_style = if app.focus_area == FocusArea::Content {
                    Style::default().fg(theme.border_focused)
                } else {
                    Style::default().fg(theme.border_normal)
                };

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(Line::from(vec![
                                Span::styled("📋 ", Style::default().fg(theme.accent)),
                                Span::styled(
                                    &section_title,
                                    Style::default()
                                        .fg(theme.foreground)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!(" ({entry_count} entries)"),
                                    Style::default().fg(theme.muted).add_modifier(Modifier::DIM),
                                ),
                            ]))
                            .border_style(border_style),
                    )
                    .style(Style::default().fg(theme.foreground))
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
                        Style::default().fg(theme.success)
                    } else {
                        Style::default().fg(theme.primary)
                    };

                    let paragraph = Paragraph::new(raw_content)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(format!("📄 {section_title}"))
                                .border_style(border_style),
                        )
                        .style(Style::default().fg(theme.foreground))
                        .wrap(Wrap { trim: true });
                    f.render_widget(paragraph, area);
                } else {
                    // Empty section
                    let border_style = if app.focus_area == FocusArea::Content {
                        Style::default().fg(theme.success)
                    } else {
                        Style::default().fg(theme.primary)
                    };

                    let empty = Paragraph::new("📭 No entries in this section")
                        .style(Style::default().fg(theme.muted))
                        .alignment(Alignment::Center)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(format!("📂 {section_title}"))
                                .border_style(border_style),
                        );
                    f.render_widget(empty, area);
                }
            }
        } else {
            // Section not found
            let border_style = if app.focus_area == FocusArea::Content {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.primary)
            };

            let error = Paragraph::new("Section not found")
                .style(Style::default().fg(theme.error))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("❌ Error")
                        .border_style(border_style),
                );
            f.render_widget(error, area);
        }
    } else {
        // No current tab
        let error = Paragraph::new("No section selected")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📂 Content")
                    .border_style(Style::default().fg(theme.primary)),
            );
        f.render_widget(error, area);
    }
}

fn draw_loading(f: &mut Frame, area: Rect, theme: &ThemeColors) {
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
                .fg(theme.warning)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(
            "Loading README content...\n\n",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "✨ Fetching awesome resources from GitHub ✨",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::DIM),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled("📡 ", Style::default().fg(theme.warning)),
                Span::styled(
                    "Loading",
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .border_style(Style::default().fg(theme.warning)),
    );
    f.render_widget(loading, loading_chunks[1]);
}

fn draw_error(f: &mut Frame, area: Rect, error: &str, theme: &ThemeColors) {
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
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Error Occurred\n\n",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(error, Style::default().fg(theme.muted)),
        Span::styled("\n\n", Style::default()),
        Span::styled("💡 Press ", Style::default().fg(theme.muted)),
        Span::styled(
            "'R'",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to retry", Style::default().fg(theme.muted)),
    ]))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled("⚠️  ", Style::default().fg(theme.error)),
                Span::styled(
                    "Error",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .border_style(Style::default().fg(theme.error)),
    );
    f.render_widget(error_text, error_chunks[1]);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let help_text = if app.search_mode {
        Line::from(vec![
            Span::styled(
                "ESC",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Exit search │ ", Style::default().fg(theme.muted)),
            Span::styled(
                "j/k",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Navigate │ ", Style::default().fg(theme.muted)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                ": Open URL │ Type to search...",
                Style::default().fg(theme.muted),
            ),
        ])
    } else {
        let theme_key_style = if app.is_theme_applied() {
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD)
        };

        Line::from(vec![
            Span::styled(
                "h/l",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Switch sections │ ", Style::default().fg(theme.muted)),
            Span::styled(
                "j/k",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Navigate │ ", Style::default().fg(theme.muted)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Open URL │ ", Style::default().fg(theme.muted)),
            Span::styled(
                "R",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Reload │ ", Style::default().fg(theme.muted)),
            Span::styled("T", theme_key_style),
            Span::styled(
                if app.is_theme_applied() {
                    ": Themes (applied) │ "
                } else {
                    ": Themes │ "
                },
                Style::default().fg(theme.muted),
            ),
            Span::styled(
                "Q",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Quit", Style::default().fg(theme.muted)),
        ])
    };

    let footer = Paragraph::new(help_text).alignment(Alignment::Center);
    f.render_widget(footer, area);
}

fn draw_search_popup(f: &mut Frame, app: &App, theme: &ThemeColors) {
    let popup_area = centered_rect(80, 60, f.area());

    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(popup_area);

    // Enhanced search input with premium styling
    let search_text = Line::from(vec![
        Span::styled("🔍 ", Style::default().fg(theme.warning)),
        Span::styled(&app.search_query, Style::default().fg(theme.warning)),
        Span::styled(
            "▌",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]);

    let search_input = Paragraph::new(search_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled("🔍 ", Style::default().fg(theme.warning)),
                Span::styled(
                    "Search",
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .border_style(Style::default().fg(theme.warning)),
    );
    f.render_widget(search_input, chunks[0]);

    // Simplified results display
    if !app.search_results.is_empty() {
        let results: Vec<ListItem> = app
            .search_results
            .iter()
            .take(10)
            .enumerate()
            .map(|(i, result)| {
                let display_text = app
                    .readme_content
                    .as_ref()
                    .and_then(|readme| {
                        let section = readme.sections.get(result.section_index)?;
                        match result.entry_index {
                            Some(entry_idx) => section.entries.get(entry_idx).map(|e| &e.title),
                            None => Some(&section.title),
                        }
                    })
                    .unwrap_or(&result.line_content);

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(theme.secondary)),
                    Span::styled(
                        display_text.chars().take(60).collect::<String>(),
                        Style::default().fg(theme.foreground),
                    ),
                ]))
            })
            .collect();

        let mut search_state = ratatui::widgets::ListState::default();
        search_state.select(app.search_selection);

        let results_list = List::new(results)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Results ({})", app.search_results.len()))
                    .border_style(Style::default().fg(theme.success)),
            )
            .highlight_style(Style::default().fg(theme.warning))
            .highlight_symbol("▶ ");
        f.render_stateful_widget(results_list, chunks[1], &mut search_state);
    }
}

fn draw_theme_browser_popup(f: &mut Frame, app: &App, theme: &ThemeColors) {
    let popup_area = centered_rect(85, 70, f.area());

    f.render_widget(Clear, popup_area);

    // Add search bar if in search mode
    let (title_area, main_area) = if app.theme_browser.search_mode {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // Main content
            ])
            .split(popup_area);
        (chunks[0], chunks[2])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(popup_area);
        (chunks[0], chunks[1])
    };

    // Theme browser title with enhanced styling
    let title_text = "🎨 Hybrid Multi-Panel Theme Preview";

    let preview_info = match app.preview_state {
        crate::models::PreviewState::Loading => " - Loading theme preview...",
        crate::models::PreviewState::Applied(_) => " - Multi-panel preview active! ESC to restore",
        crate::models::PreviewState::Error => " - Error loading theme",
        _ => {
            if app.theme_browser.search_mode {
                " - Type to filter themes, j/k navigate, Enter to apply, ESC to clear"
            } else {
                " - Navigate with j/k, Enter for preview, / to search, ESC to close"
            }
        }
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled("🎨 ", Style::default().fg(theme.secondary)),
        Span::styled(
            title_text,
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(preview_info, Style::default().fg(theme.muted)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.secondary)),
    );
    f.render_widget(title, title_area);

    // Render search bar if in search mode
    if app.theme_browser.search_mode {
        let search_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // Main content
            ])
            .split(popup_area);

        let search_text = Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(theme.warning)),
            Span::styled(
                &app.theme_browser.search_query,
                Style::default().fg(theme.warning),
            ),
            Span::styled(
                "▌",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]);

        let result_count = if app.theme_browser.search_query.trim().is_empty() {
            String::new()
        } else {
            format!(" ({} matches)", app.theme_browser.filtered_themes.len())
        };

        let search_input = Paragraph::new(search_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled("🔍 ", Style::default().fg(theme.warning)),
                    Span::styled(
                        "Filter Themes",
                        Style::default()
                            .fg(theme.warning)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&result_count, Style::default().fg(theme.muted)),
                ]))
                .border_style(Style::default().fg(theme.warning)),
        );
        f.render_widget(search_input, search_chunks[1]);
    }

    // Theme content
    if app.theme_browser.loading {
        let loading = Paragraph::new("🔄 Loading themes...")
            .style(Style::default().fg(theme.warning))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Loading")
                    .border_style(Style::default().fg(theme.warning)),
            );
        f.render_widget(loading, main_area);
    } else if let Some(ref error) = app.theme_browser.error {
        let error_text = Paragraph::new(format!("❌ Error: {error}"))
            .style(Style::default().fg(theme.error))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Error")
                    .border_style(Style::default().fg(theme.error)),
            );
        f.render_widget(error_text, main_area);
    } else {
        // Show theme content
        draw_aur_theme_content(f, main_area, app, theme);
    }
}

fn draw_aur_theme_content(f: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    if app.theme_entries.is_empty() {
        let empty_text = Paragraph::new("No themes found in README")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Empty")
                    .border_style(Style::default().fg(theme.muted)),
            );
        f.render_widget(empty_text, area);
        return;
    }

    // Split the area into two parts: theme selector and preview
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Theme list
            Constraint::Percentage(75), // Multi-panel preview
        ])
        .split(area);

    // Draw theme selector on the left
    draw_theme_selector(f, main_chunks[0], app, theme);

    // Draw multi-panel preview on the right
    if let Some(current_theme) = get_current_preview_theme(app) {
        draw_hybrid_multi_panel_preview(f, main_chunks[1], current_theme, theme);
    } else {
        // Show instructions when no theme is selected
        draw_preview_instructions(f, main_chunks[1], theme);
    }
}

fn draw_theme_selector(f: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let (items, selected_index) = if app.theme_browser.search_mode
        && !app.theme_browser.filtered_themes.is_empty()
    {
        // Show filtered results
        let filtered_items: Vec<ListItem> = app
                .theme_browser.filtered_themes
                .iter()
                .enumerate()
                .filter_map(|(list_idx, &theme_idx)| {
                    app.theme_entries.get(theme_idx).map(|theme_entry| {
                        let is_selected = app.theme_browser.filtered_selected == Some(list_idx);
                        let is_applied = matches!(app.preview_state,
                            crate::models::PreviewState::Applied(ref t) if t.as_ref().name == theme_entry.name
                        );

                        let name_style = if is_selected {
                            if is_applied {
                                Style::default()
                                    .fg(theme.success)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                                    .fg(theme.foreground)
                                    .add_modifier(Modifier::BOLD)
                            }
                        } else {
                            Style::default().fg(theme.muted)
                        };

                        let mut spans = vec![
                            if is_selected {
                                Span::styled("▶ ", Style::default().fg(theme.success))
                            } else {
                                Span::styled("  ", Style::default())
                            },
                            Span::styled(&theme_entry.name, name_style),
                        ];

                        if is_applied {
                            spans.push(Span::styled(" ✨", Style::default().fg(theme.success)));
                        }

                        ListItem::new(Line::from(spans))
                    })
                })
                .collect();

        (filtered_items, app.theme_browser.filtered_selected)
    } else if app.theme_browser.search_mode
        && app.theme_browser.filtered_themes.is_empty()
        && !app.theme_browser.search_query.trim().is_empty()
    {
        // Show "no matches" when search has no results
        let no_matches = vec![ListItem::new(Line::from(vec![
            Span::styled("  No themes match \"", Style::default().fg(theme.muted)),
            Span::styled(
                &app.theme_browser.search_query,
                Style::default().fg(theme.warning),
            ),
            Span::styled("\"", Style::default().fg(theme.muted)),
        ]))];
        (no_matches, None)
    } else {
        // Show all themes
        let all_items: Vec<ListItem> = app
                .theme_entries
                .iter()
                .enumerate()
                .map(|(i, theme_entry)| {
                    let is_selected = app.theme_browser.selected_index == Some(i);
                    let is_applied = matches!(app.preview_state,
                        crate::models::PreviewState::Applied(ref t) if t.as_ref().name == theme_entry.name
                    );

                    let name_style = if is_selected {
                        if is_applied {
                            Style::default()
                                .fg(theme.success)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                                .fg(theme.foreground)
                                .add_modifier(Modifier::BOLD)
                        }
                    } else {
                        Style::default().fg(theme.muted)
                    };

                    let mut spans = vec![
                        if is_selected {
                            Span::styled("▶ ", Style::default().fg(theme.success))
                        } else {
                            Span::styled("  ", Style::default())
                        },
                        Span::styled(&theme_entry.name, name_style),
                    ];

                    if is_applied {
                        spans.push(Span::styled(" ✨", Style::default().fg(theme.success)));
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect();

        (all_items, app.theme_browser.selected_index)
    };

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(selected_index);

    let theme_count =
        if app.theme_browser.search_mode && !app.theme_browser.search_query.trim().is_empty() {
            format!(
                " ({}/{})",
                app.theme_browser.filtered_themes.len(),
                app.theme_entries.len()
            )
        } else {
            format!(" ({})", app.theme_entries.len())
        };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("🎨 Themes{theme_count}"))
                .border_style(Style::default().fg(theme.primary)),
        )
        .highlight_style(Style::default())
        .highlight_symbol("");

    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_preview_instructions(f: &mut Frame, area: Rect, theme: &ThemeColors) {
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(theme.warning)),
            Span::styled(
                "Theme Preview",
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "  j/k",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Navigate themes", Style::default().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled(
                "  Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " - Apply theme and show preview",
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  ESC",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - Close and restore", Style::default().fg(theme.muted)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Select a theme and press ",
                Style::default().fg(theme.muted),
            ),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to see:", Style::default().fg(theme.muted)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(theme.warning)),
            Span::styled(
                "Terminal simulation with theme colors",
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(theme.warning)),
            Span::styled(
                "Application preview (btop-style)",
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(theme.warning)),
            Span::styled(
                "Code editor with syntax highlighting",
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(theme.warning)),
            Span::styled(
                "Color palette with hex values",
                Style::default().fg(theme.muted),
            ),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("📋 Preview Instructions")
            .border_style(Style::default().fg(theme.primary)),
    );

    f.render_widget(instructions, area);
}

fn get_current_preview_theme(app: &App) -> Option<&crate::models::Theme> {
    match &app.preview_state {
        crate::models::PreviewState::Applied(theme) => Some(theme.as_ref()),
        _ => None,
    }
}

fn draw_hybrid_multi_panel_preview(
    f: &mut Frame,
    area: Rect,
    theme_colors: &crate::models::Theme,
    ui_theme: &ThemeColors,
) {
    // Create 4-panel layout
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Top row (Terminal + Application)
            Constraint::Percentage(50), // Bottom row (Editor + Palette)
        ])
        .split(area);

    // Top row: Terminal and Application panels
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Terminal panel
            Constraint::Percentage(50), // Application panel
        ])
        .split(main_chunks[0]);

    // Bottom row: Editor and Palette panels
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Editor panel
            Constraint::Percentage(50), // Palette panel
        ])
        .split(main_chunks[1]);

    // Draw each panel with the applied theme
    draw_terminal_preview_panel(f, top_chunks[0], theme_colors, ui_theme);
    draw_application_preview_panel(f, top_chunks[1], theme_colors, ui_theme);
    draw_editor_preview_panel(f, bottom_chunks[0], theme_colors, ui_theme);
    draw_palette_preview_panel(f, bottom_chunks[1], theme_colors, ui_theme);
}

fn draw_terminal_preview_panel(
    f: &mut Frame,
    area: Rect,
    theme: &crate::models::Theme,
    ui_theme: &ThemeColors,
) {
    let bg_color = parse_hex_color(&theme.colors.background).unwrap_or(ui_theme.background);
    let fg_color = parse_hex_color(&theme.colors.foreground).unwrap_or(ui_theme.foreground);
    let prompt_color = parse_hex_color(&theme.colors.bright.green).unwrap_or(ui_theme.success);
    let command_color = parse_hex_color(&theme.colors.normal.yellow).unwrap_or(ui_theme.warning);
    let file_color = parse_hex_color(&theme.colors.bright.blue).unwrap_or(ui_theme.primary);
    let dir_color = parse_hex_color(&theme.colors.bright.cyan).unwrap_or(ui_theme.secondary);

    let terminal_content = vec![
        Line::from(vec![
            Span::styled(
                "user@omarchy ",
                Style::default()
                    .fg(prompt_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "~/awesome-omarchy ",
                Style::default().fg(dir_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("$ ", Style::default().fg(fg_color)),
            Span::styled("ls -la", Style::default().fg(command_color)),
        ]),
        Line::from(vec![Span::styled(
            "total 42",
            Style::default().fg(fg_color),
        )]),
        Line::from(vec![
            Span::styled(
                "drwxr-xr-x 5 user user 4096 Aug 21 14:30 ",
                Style::default().fg(fg_color),
            ),
            Span::styled(
                ".",
                Style::default().fg(dir_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "drwxr-xr-x 3 user user 4096 Aug 21 14:25 ",
                Style::default().fg(fg_color),
            ),
            Span::styled(
                "..",
                Style::default().fg(dir_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "-rw-r--r-- 1 user user 1034 Aug 21 14:30 ",
                Style::default().fg(fg_color),
            ),
            Span::styled("Cargo.toml", Style::default().fg(file_color)),
        ]),
        Line::from(vec![
            Span::styled(
                "-rw-r--r-- 1 user user 5847 Aug 21 14:28 ",
                Style::default().fg(fg_color),
            ),
            Span::styled("README.md", Style::default().fg(file_color)),
        ]),
        Line::from(vec![
            Span::styled(
                "drwxr-xr-x 2 user user 4096 Aug 21 14:25 ",
                Style::default().fg(fg_color),
            ),
            Span::styled(
                "src/",
                Style::default().fg(dir_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "user@omarchy ",
                Style::default()
                    .fg(prompt_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "~/awesome-omarchy ",
                Style::default().fg(dir_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("$ ", Style::default().fg(fg_color)),
            Span::styled("cargo run", Style::default().fg(command_color)),
        ]),
        Line::from(vec![Span::styled(
            "   Compiling awesome-omarchy-tui v0.1.0",
            Style::default().fg(fg_color),
        )]),
        Line::from(vec![
            Span::styled("    Finished ", Style::default().fg(fg_color)),
            Span::styled(
                "dev",
                Style::default()
                    .fg(prompt_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" [unoptimized + debuginfo]", Style::default().fg(fg_color)),
        ]),
    ];

    let terminal_panel = Paragraph::new(terminal_content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🖥️ Terminal")
                .border_style(Style::default().fg(ui_theme.primary)),
        );

    f.render_widget(terminal_panel, area);
}

fn draw_application_preview_panel(
    f: &mut Frame,
    area: Rect,
    theme: &crate::models::Theme,
    ui_theme: &ThemeColors,
) {
    let bg_color = parse_hex_color(&theme.colors.background).unwrap_or(ui_theme.background);
    let fg_color = parse_hex_color(&theme.colors.foreground).unwrap_or(ui_theme.foreground);
    let cpu_color = parse_hex_color(&theme.colors.normal.red).unwrap_or(ui_theme.error);
    let mem_color = parse_hex_color(&theme.colors.normal.blue).unwrap_or(ui_theme.primary);
    let disk_color = parse_hex_color(&theme.colors.normal.green).unwrap_or(ui_theme.success);
    let label_color = parse_hex_color(&theme.colors.bright.white).unwrap_or(ui_theme.foreground);

    let application_content = vec![
        Line::from(vec![Span::styled(
            "System Monitor",
            Style::default()
                .fg(label_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "CPU: ",
                Style::default()
                    .fg(label_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("████████████████░░░░ ", Style::default().fg(cpu_color)),
            Span::styled(
                "67%",
                Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            "     Intel i7-12700K @ 3.60GHz",
            Style::default().fg(fg_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "MEM: ",
                Style::default()
                    .fg(label_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("████████████░░░░░░░░ ", Style::default().fg(mem_color)),
            Span::styled(
                "54%",
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            "     8.7GB / 16.0GB",
            Style::default().fg(fg_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "DSK: ",
                Style::default()
                    .fg(label_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("██████░░░░░░░░░░░░░░ ", Style::default().fg(disk_color)),
            Span::styled(
                "23%",
                Style::default().fg(disk_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            "     247GB / 1TB SSD",
            Style::default().fg(fg_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "NET: ",
                Style::default()
                    .fg(label_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("↓", Style::default().fg(disk_color)),
            Span::styled("127MB/s ", Style::default().fg(fg_color)),
            Span::styled("↑", Style::default().fg(cpu_color)),
            Span::styled("42MB/s", Style::default().fg(fg_color)),
        ]),
    ];

    let application_panel = Paragraph::new(application_content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 System Monitor")
                .border_style(Style::default().fg(ui_theme.primary)),
        );

    f.render_widget(application_panel, area);
}

fn draw_editor_preview_panel(
    f: &mut Frame,
    area: Rect,
    theme: &crate::models::Theme,
    ui_theme: &ThemeColors,
) {
    let bg_color = parse_hex_color(&theme.colors.background).unwrap_or(ui_theme.background);
    let fg_color = parse_hex_color(&theme.colors.foreground).unwrap_or(ui_theme.foreground);
    let keyword_color = parse_hex_color(&theme.colors.bright.magenta).unwrap_or(ui_theme.accent);
    let string_color = parse_hex_color(&theme.colors.normal.green).unwrap_or(ui_theme.success);
    let comment_color = parse_hex_color(&theme.colors.normal.black).unwrap_or(ui_theme.muted);
    let function_color = parse_hex_color(&theme.colors.bright.blue).unwrap_or(ui_theme.primary);
    let number_color = parse_hex_color(&theme.colors.normal.cyan).unwrap_or(ui_theme.secondary);

    let editor_content = vec![
        Line::from(vec![
            Span::styled("1 ", Style::default().fg(comment_color)),
            Span::styled(
                "use ",
                Style::default()
                    .fg(keyword_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("std::collections::HashMap;", Style::default().fg(fg_color)),
        ]),
        Line::from(vec![Span::styled("2 ", Style::default().fg(comment_color))]),
        Line::from(vec![
            Span::styled("3 ", Style::default().fg(comment_color)),
            Span::styled(
                "// Theme preview implementation",
                Style::default()
                    .fg(comment_color)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
        Line::from(vec![
            Span::styled("4 ", Style::default().fg(comment_color)),
            Span::styled(
                "pub fn ",
                Style::default()
                    .fg(keyword_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "apply_theme",
                Style::default()
                    .fg(function_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("(theme: &", Style::default().fg(fg_color)),
            Span::styled("Theme", Style::default().fg(keyword_color)),
            Span::styled(") {", Style::default().fg(fg_color)),
        ]),
        Line::from(vec![
            Span::styled("5 ", Style::default().fg(comment_color)),
            Span::styled("    ", Style::default()),
            Span::styled(
                "let ",
                Style::default()
                    .fg(keyword_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("colors = theme.colors;", Style::default().fg(fg_color)),
        ]),
        Line::from(vec![
            Span::styled("6 ", Style::default().fg(comment_color)),
            Span::styled("    ", Style::default()),
            Span::styled(
                "println!",
                Style::default()
                    .fg(function_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("(", Style::default().fg(fg_color)),
            Span::styled("\"Theme: {}\"", Style::default().fg(string_color)),
            Span::styled(", theme.name);", Style::default().fg(fg_color)),
        ]),
        Line::from(vec![
            Span::styled("7 ", Style::default().fg(comment_color)),
            Span::styled("}", Style::default().fg(fg_color)),
        ]),
        Line::from(vec![Span::styled("8 ", Style::default().fg(comment_color))]),
        Line::from(vec![
            Span::styled("9 ", Style::default().fg(comment_color)),
            Span::styled(
                "const ",
                Style::default()
                    .fg(keyword_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("MAX_THEMES", Style::default().fg(fg_color)),
            Span::styled(": ", Style::default().fg(fg_color)),
            Span::styled("usize ", Style::default().fg(keyword_color)),
            Span::styled("= ", Style::default().fg(fg_color)),
            Span::styled(
                "42",
                Style::default()
                    .fg(number_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(";", Style::default().fg(fg_color)),
        ]),
    ];

    let editor_panel = Paragraph::new(editor_content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📝 Code Editor")
                .border_style(Style::default().fg(ui_theme.primary)),
        );

    f.render_widget(editor_panel, area);
}

fn draw_palette_preview_panel(
    f: &mut Frame,
    area: Rect,
    theme: &crate::models::Theme,
    ui_theme: &ThemeColors,
) {
    let bg_color = parse_hex_color(&theme.colors.background).unwrap_or(ui_theme.background);
    let fg_color = parse_hex_color(&theme.colors.foreground).unwrap_or(ui_theme.foreground);

    let palette_content = vec![
        Line::from(vec![Span::styled(
            "Color Palette",
            Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "BG: ",
                Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("██ ", Style::default().bg(bg_color)),
            Span::styled(
                &theme.colors.background,
                Style::default().fg(ui_theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "FG: ",
                Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("██ ", Style::default().bg(fg_color)),
            Span::styled(
                &theme.colors.foreground,
                Style::default().fg(ui_theme.muted),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "🔴 ",
                Style::default()
                    .fg(parse_hex_color(&theme.colors.normal.red).unwrap_or(Color::Indexed(1))),
            ),
            Span::styled(
                &theme.colors.normal.red,
                Style::default().fg(ui_theme.muted),
            ),
            Span::styled(" RED", Style::default().fg(ui_theme.muted)),
        ]),
        Line::from(vec![
            Span::styled(
                "🟢 ",
                Style::default()
                    .fg(parse_hex_color(&theme.colors.normal.green).unwrap_or(Color::Indexed(2))),
            ),
            Span::styled(
                &theme.colors.normal.green,
                Style::default().fg(ui_theme.muted),
            ),
            Span::styled(" GREEN", Style::default().fg(ui_theme.muted)),
        ]),
        Line::from(vec![
            Span::styled(
                "🔵 ",
                Style::default()
                    .fg(parse_hex_color(&theme.colors.normal.blue).unwrap_or(Color::Indexed(4))),
            ),
            Span::styled(
                &theme.colors.normal.blue,
                Style::default().fg(ui_theme.muted),
            ),
            Span::styled(" BLUE", Style::default().fg(ui_theme.muted)),
        ]),
        Line::from(vec![
            Span::styled(
                "🟡 ",
                Style::default()
                    .fg(parse_hex_color(&theme.colors.normal.yellow).unwrap_or(Color::Indexed(3))),
            ),
            Span::styled(
                &theme.colors.normal.yellow,
                Style::default().fg(ui_theme.muted),
            ),
            Span::styled(" YELLOW", Style::default().fg(ui_theme.muted)),
        ]),
        Line::from(vec![
            Span::styled(
                "🟣 ",
                Style::default()
                    .fg(parse_hex_color(&theme.colors.normal.magenta).unwrap_or(Color::Indexed(5))),
            ),
            Span::styled(
                &theme.colors.normal.magenta,
                Style::default().fg(ui_theme.muted),
            ),
            Span::styled(" MAGENTA", Style::default().fg(ui_theme.muted)),
        ]),
        Line::from(vec![
            Span::styled(
                "🩵 ",
                Style::default()
                    .fg(parse_hex_color(&theme.colors.normal.cyan).unwrap_or(Color::Indexed(6))),
            ),
            Span::styled(
                &theme.colors.normal.cyan,
                Style::default().fg(ui_theme.muted),
            ),
            Span::styled(" CYAN", Style::default().fg(ui_theme.muted)),
        ]),
    ];

    let palette_panel = Paragraph::new(palette_content)
        .style(Style::default().bg(bg_color).fg(fg_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🎨 Color Palette")
                .border_style(Style::default().fg(ui_theme.primary)),
        );

    f.render_widget(palette_panel, area);
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

// Helper function to parse hex colors to ratatui Color
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

/// Format markdown text with basic styling hints
fn format_markdown_text(text: &str) -> String {
    // Simple markdown formatting - replace **bold** and *italic* markers
    let text = text.replace("**", "");
    let text = text.replace("*", "");
    // Remove excessive whitespace and clean up
    text.trim().to_string()
}

/// Get color for different tag types - using ANSI colors for terminal theme compatibility
fn get_tag_color(tag: &str, theme: &ThemeColors) -> Color {
    match tag.to_lowercase().as_str() {
        "rust" => Color::Indexed(11),  // ANSI bright yellow (rust-like)
        "python" => Color::Indexed(3), // ANSI yellow
        "javascript" | "typescript" => Color::Indexed(11), // ANSI bright yellow
        "go" | "golang" => Color::Indexed(6), // ANSI cyan
        "java" => Color::Indexed(1),   // ANSI red
        "cpp" | "c++" => Color::Indexed(4), // ANSI blue
        "tool" | "cli" => theme.primary,
        "library" => theme.secondary,
        "framework" => theme.accent,
        "web" => Color::Indexed(2), // ANSI green
        "api" => Color::Indexed(5), // ANSI magenta
        "plugin" | "extension" => theme.warning,
        _ => theme.muted,
    }
}

/// Format GitHub URL to show only repository name
fn format_github_url(url: &str) -> String {
    if let Some(repo_part) = url.strip_prefix("https://github.com/") {
        // Extract just the owner/repo part
        let parts: Vec<&str> = repo_part.split('/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else {
            repo_part.to_string()
        }
    } else {
        url.to_string()
    }
}
