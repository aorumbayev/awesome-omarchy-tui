use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Boot screen configuration and state
pub struct BootScreen {
    start_time: Instant,
    animation_phase: AnimationPhase,
    show_loading_dots: u8,
    animation_frame: u32,
    #[allow(dead_code)]
    terminal_size: (u16, u16),
}

#[derive(Debug, Clone, PartialEq)]
enum AnimationPhase {
    FadeIn,
    ShowLogo,
    Loading,
    Complete,
}

// Using only Matrix animation - removed animation style enum

impl BootScreen {
    pub fn new(terminal_size: (u16, u16)) -> Self {
        Self {
            start_time: Instant::now(),
            animation_phase: AnimationPhase::FadeIn,
            show_loading_dots: 0,
            animation_frame: 0,
            terminal_size,
        }
    }

    /// Run the boot screen animation
    pub async fn run<F>(&mut self, draw_fn: F) -> Result<bool>
    where
        F: Fn(&mut Self, &mut Frame) + Send + 'static,
    {
        use crossterm::{
            execute,
            terminal::{
                EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
            },
            tty::IsTty,
        };
        use ratatui::{Terminal, backend::CrosstermBackend};
        use std::io;

        // Check if we can setup terminal properly
        if !std::io::stdout().is_tty() {
            // If not in a TTY, skip boot screen
            return Ok(true);
        }

        // Setup terminal for boot screen
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let boot_duration = Duration::from_millis(2500); // 2.5 seconds total (faster)
        let frame_duration = Duration::from_millis(33); // ~30 FPS (smoother)

        loop {
            let elapsed = self.start_time.elapsed();

            // Check for user input to skip
            if poll(Duration::from_millis(0))? {
                if let Event::Key(KeyEvent {
                    code, modifiers, ..
                }) = read()?
                {
                    match code {
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            ratatui::restore();
                            return Ok(false); // Exit application
                        }
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                            break; // Skip boot screen
                        }
                        _ => {}
                    }
                }
            }

            // Update animation phase and frame counter
            self.update_animation_phase(elapsed);
            self.animation_frame = self.animation_frame.wrapping_add(1);

            // Draw the current frame
            terminal.draw(|frame| {
                draw_fn(self, frame);
            })?;

            // Check if boot screen should complete
            if elapsed >= boot_duration || self.animation_phase == AnimationPhase::Complete {
                break;
            }

            sleep(frame_duration).await;
        }

        // Cleanup terminal after boot screen
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(true) // Continue to main application
    }

    fn update_animation_phase(&mut self, elapsed: Duration) {
        self.animation_phase = match elapsed.as_millis() {
            0..=500 => AnimationPhase::FadeIn,      // Faster fade-in: 0.5s
            501..=1800 => AnimationPhase::ShowLogo, // Faster logo: 1.3s
            1801..=2300 => AnimationPhase::Loading, // Faster loading: 0.5s
            _ => AnimationPhase::Complete,
        };

        // Update loading dots animation (faster)
        if elapsed.as_millis() % 200 == 0 {
            self.show_loading_dots = (self.show_loading_dots + 1) % 4;
        }
    }

    // Smooth easing functions for better animation feel
    fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powf(3.0) / 2.0
        }
    }

    fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powf(3.0)
    }

    /// Apply Omarchy-inspired animation effects to ASCII art
    fn apply_animation_style(
        &self,
        lines: Vec<Line<'static>>,
        progress: f32,
    ) -> Vec<Line<'static>> {
        self.apply_minimal_fade(lines, progress)
    }

    /// Minimalistic fade-in animation with subtle breathing glow
    fn apply_minimal_fade(&self, lines: Vec<Line<'static>>, progress: f32) -> Vec<Line<'static>> {
        // Smooth breathing glow
        let time = self.animation_frame as f32 * 0.05;
        let glow = ((time).sin() + 1.0) / 2.0 * 0.3 + 0.7; // 0.7 to 1.0

        let total_lines = lines.len();
        let center = total_lines / 2;

        lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                // Staggered fade-in from center outward
                let distance_from_center =
                    ((i as i32 - center as i32).abs() as f32) / (total_lines as f32);
                let line_progress = ((progress - distance_from_center * 0.3) * 1.5).clamp(0.0, 1.0);

                // Apply fade and glow to each span
                let faded_spans = line
                    .spans
                    .into_iter()
                    .map(|span| {
                        let opacity = (line_progress * glow * 255.0) as u8;

                        // Determine base color from original style - use ANSI colors for theme adaptation
                        let new_color = if span.style.fg == Some(Color::Indexed(15))
                            || span.style.fg == Some(Color::White)
                        {
                            // White text - use ANSI colors based on opacity
                            if opacity > 200 {
                                Color::Indexed(15) // ANSI bright white
                            } else if opacity > 100 {
                                Color::Indexed(7) // ANSI white
                            } else {
                                Color::Indexed(8) // ANSI bright black (dim)
                            }
                        } else if span.style.fg == Some(Color::Indexed(10)) {
                            // Green text (OMARCHY) - use ANSI bright green with opacity
                            if opacity > 128 {
                                Color::Indexed(10) // ANSI bright green
                            } else {
                                Color::Indexed(2) // ANSI regular green (dimmed)
                            }
                        } else {
                            // Default fade - use ANSI grays based on opacity
                            if opacity > 200 {
                                Color::Indexed(15) // ANSI bright white
                            } else if opacity > 100 {
                                Color::Indexed(7) // ANSI white
                            } else {
                                Color::Indexed(8) // ANSI bright black (dim)
                            }
                        };

                        // Add bold modifier only when fully visible and glowing
                        let mut style = Style::default().fg(new_color);
                        if line_progress > 0.9 && glow > 0.95 {
                            style = style.add_modifier(Modifier::BOLD);
                        }

                        Span::styled(span.content, style)
                    })
                    .collect::<Vec<_>>();

                Line::from(faded_spans)
            })
            .collect()
    }

    /// Draw the boot screen
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Clear background with black
        frame.render_widget(Clear, area);

        // Calculate optimal scaling based on terminal size
        let scale_factor = self.calculate_scale_factor(area.width, area.height);

        match self.animation_phase {
            AnimationPhase::FadeIn => self.draw_fade_in(frame, area, scale_factor),
            AnimationPhase::ShowLogo => self.draw_logo(frame, area, scale_factor),
            AnimationPhase::Loading => self.draw_loading(frame, area, scale_factor),
            AnimationPhase::Complete => self.draw_complete(frame, area, scale_factor),
        }
    }

    fn calculate_scale_factor(&self, width: u16, height: u16) -> f32 {
        // Original ASCII art is roughly 80 chars wide and 16 lines tall
        let original_width = 80.0;
        let original_height = 16.0;

        // Calculate scale factors for width and height with generous margins for small screens
        let width_scale = if width < 40 {
            // Very small width - use almost full width
            (width as f32 * 0.95) / 20.0 // Compact art is ~20 chars
        } else {
            (width as f32 * 0.8) / original_width // Leave 20% margin
        };

        let height_scale = if height < 10 {
            // Very small height - use almost full height
            (height as f32 * 0.9) / 4.0 // Compact art is ~4 lines
        } else {
            (height as f32 * 0.6) / original_height // Leave 40% margin
        };

        // Use the smaller scale factor to maintain aspect ratio
        // Allow smaller minimum for tiny terminals
        let min_scale = if width < 30 || height < 8 { 0.1 } else { 0.3 };
        (width_scale.min(height_scale)).clamp(min_scale, 1.5)
    }

    fn draw_fade_in(&self, frame: &mut Frame, area: Rect, _scale_factor: f32) {
        let elapsed_ms = self.start_time.elapsed().as_millis() as f32;
        let fade_progress = Self::ease_in_out_cubic((elapsed_ms / 500.0).min(1.0));

        // Create a smooth pulsing effect with dots
        let pulse_phase = (elapsed_ms / 150.0).sin(); // Smooth sine wave
        let dots = if pulse_phase > 0.5 {
            "●"
        } else if pulse_phase > 0.0 {
            "◐"
        } else if pulse_phase > -0.5 {
            "◑"
        } else {
            "◒"
        };

        let center_y = area.height / 2;
        let center_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(center_y),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let fade_text = Paragraph::new(Line::from(vec![Span::styled(
            format!("  {dots}  Initializing Awesome Omarchy TUI  {dots}  "),
            Style::default()
                .fg(Color::Rgb(
                    (255.0 * fade_progress) as u8,
                    (255.0 * fade_progress) as u8,
                    (255.0 * fade_progress) as u8,
                ))
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(fade_text, center_area[1]);
    }

    fn draw_logo(&self, frame: &mut Frame, area: Rect, scale_factor: f32) {
        let elapsed_ms = self.start_time.elapsed().as_millis() as f32;
        let raw_progress = ((elapsed_ms - 500.0) / 1300.0).clamp(0.0, 1.0);
        let show_progress = Self::ease_out_cubic(raw_progress);

        // Create layout for centered content with adaptive constraints for small screens
        let vertical_constraints = if area.height < 15 {
            let available_height = area.height.saturating_sub(4); // Reserve 4 lines for ASCII
            let margin = available_height / 2;
            [
                Constraint::Length(margin), // Top margin for centering
                Constraint::Length(4),      // Compact ASCII height
                Constraint::Length(margin), // Bottom margin for centering
            ]
        } else {
            [
                Constraint::Percentage(20),
                Constraint::Min(16), // ASCII art height
                Constraint::Percentage(20),
            ]
        };

        let horizontal_constraints = if area.width < 80 {
            let ascii_width = 19; // "AWESOME OMARCHY" is 15 chars, box is 19 chars
            let available_width = area.width.saturating_sub(ascii_width);
            let margin = available_width / 2;
            [
                Constraint::Length(margin),      // Left margin for centering
                Constraint::Length(ascii_width), // Compact ASCII width
                Constraint::Length(margin),      // Right margin for centering
            ]
        } else {
            [
                Constraint::Percentage(10),
                Constraint::Min(80), // ASCII art width
                Constraint::Percentage(10),
            ]
        };

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(vertical_chunks[1]);

        let ascii_area = horizontal_chunks[1];

        // Get ASCII art based on terminal size - use more aggressive thresholds
        let base_ascii_lines = if area.width < 80 || area.height < 20 {
            // Small terminal - always use compact version
            vec![
                Line::from(vec![
                    Span::styled(
                        "AWESOME ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "OMARCHY",
                        Style::default()
                            .fg(Color::Indexed(10))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    "╔═══════╗ ╔══════╗",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Cyan)),
                    Span::styled("◆ ◆ ◆", Style::default().fg(Color::White)),
                    Span::styled(" ║ ║ ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        "TUI",
                        Style::default()
                            .fg(Color::Indexed(10))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" ║", Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![Span::styled(
                    "╚═══════╝ ╚══════╝",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
            ]
        } else if scale_factor >= 1.0 {
            self.get_full_ascii_art()
        } else if scale_factor >= 0.7 {
            self.get_medium_ascii_art()
        } else {
            self.get_compact_ascii_art()
        };

        // Apply animation style to the ASCII art
        let animated_lines = self.apply_animation_style(base_ascii_lines, show_progress);

        let ascii_paragraph = Paragraph::new(animated_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(ascii_paragraph, ascii_area);
    }

    fn draw_loading(&self, frame: &mut Frame, area: Rect, scale_factor: f32) {
        // Draw the complete logo first
        self.draw_logo_complete(frame, area, scale_factor);

        // Add loading animation below
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Length(3),
                Constraint::Percentage(37),
            ])
            .split(area);

        let loading_dots = match self.show_loading_dots {
            0 => "●  ○  ○",
            1 => "○  ●  ○",
            2 => "○  ○  ●",
            _ => "○  ●  ○",
        };

        let loading_text = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "Loading awesome resources",
                Style::default()
                    .fg(Color::Indexed(12))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                loading_dots,
                Style::default().fg(Color::Indexed(10)),
            )]),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(loading_text, vertical_chunks[1]);

        // Add skip instruction
        let skip_text = Paragraph::new(Line::from(vec![Span::styled(
            "Press any key to skip",
            Style::default()
                .fg(Color::Indexed(8))
                .add_modifier(Modifier::ITALIC),
        )]))
        .alignment(Alignment::Center);

        let bottom_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(vertical_chunks[2]);

        frame.render_widget(skip_text, bottom_area[1]);
    }

    fn draw_complete(&self, frame: &mut Frame, area: Rect, scale_factor: f32) {
        self.draw_logo_complete(frame, area, scale_factor);

        // Add "Ready!" message
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(65),
                Constraint::Length(1),
                Constraint::Percentage(34),
            ])
            .split(area);

        let ready_text = Paragraph::new(Line::from(vec![Span::styled(
            "✨ Ready! ✨",
            Style::default()
                .fg(Color::Indexed(15))
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);

        frame.render_widget(ready_text, vertical_chunks[1]);
    }

    fn draw_logo_complete(&self, frame: &mut Frame, area: Rect, scale_factor: f32) {
        let vertical_constraints = if area.height < 15 {
            let available_height = area.height.saturating_sub(4); // Reserve 4 lines for ASCII
            let margin = available_height / 2;
            [
                Constraint::Length(margin), // Top margin for centering
                Constraint::Length(4),      // Compact ASCII height
                Constraint::Length(margin), // Bottom margin for centering
            ]
        } else {
            [
                Constraint::Percentage(15),
                Constraint::Min(16),
                Constraint::Percentage(25),
            ]
        };

        let horizontal_constraints = if area.width < 80 {
            let ascii_width = 19; // "AWESOME OMARCHY" is 15 chars, box is 19 chars
            let available_width = area.width.saturating_sub(ascii_width);
            let margin = available_width / 2;
            [
                Constraint::Length(margin),      // Left margin for centering
                Constraint::Length(ascii_width), // Compact ASCII width
                Constraint::Length(margin),      // Right margin for centering
            ]
        } else {
            [
                Constraint::Percentage(10),
                Constraint::Min(80),
                Constraint::Percentage(10),
            ]
        };

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(vertical_chunks[1]);

        let ascii_area = horizontal_chunks[1];

        let ascii_lines = if area.width < 80 || area.height < 20 {
            // Small terminal - always use compact version
            vec![
                Line::from(vec![
                    Span::styled(
                        "AWESOME ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "OMARCHY",
                        Style::default()
                            .fg(Color::Indexed(10))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![Span::styled(
                    "╔═══════╗ ╔══════╗",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Cyan)),
                    Span::styled("◆ ◆ ◆", Style::default().fg(Color::White)),
                    Span::styled(" ║ ║ ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        "TUI",
                        Style::default()
                            .fg(Color::Indexed(10))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" ║", Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![Span::styled(
                    "╚═══════╝ ╚══════╝",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
            ]
        } else if scale_factor >= 1.0 {
            self.get_full_ascii_art()
        } else if scale_factor >= 0.7 {
            self.get_medium_ascii_art()
        } else {
            self.get_compact_ascii_art()
        };

        let ascii_paragraph = Paragraph::new(ascii_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(ascii_paragraph, ascii_area);
    }

    fn get_full_ascii_art(&self) -> Vec<Line<'static>> {
        vec![
            // AWESOME in white - Fixed ASCII art with proper spacing
            Line::from(vec![Span::styled(
                "   ▄████████  ▄█     █▄     ████████     ▄████████   ▄██████▄    ▄▄▄▄███▄▄▄▄      ████████   ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "  ███    ███ ███     ███   ███    ███   ███    ███  ███    ███ ▄██▀▀▀███▀▀▀██▄   ███    ███ ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "  ███    ███ ███     ███   ███    █▀    ███    █▀   ███    ███ ███   ███   ███   ███    █▀  ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "  ███    ███ ███     ███  ▄███▄▄▄       ███         ███    ███ ███   ███   ███  ▄███▄▄▄     ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " ▀███████████ ███     ███ ▀▀███▀▀▀     ▀▀███████████ ███    ███ ███   ███   ███ ▀▀███▀▀▀     ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "  ███    ███ ███     ███   ███    █▄           ███  ███    ███ ███   ███   ███   ███    █▄  ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "  ███    ███ ███ ▄█▄ ███   ███    ███    ▄█    ███  ███    ███ ███   ███   ███   ███    ███ ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "  ███    █▀   ▀███▀███▀    ██████████  ▄████████▀    ▀██████▀   ▀█   ███   █▀    ██████████ ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            // OMARCHY in green
            Line::from(vec![Span::styled(
                "                 ▄▄▄",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " ▄█████▄    ▄███████████▄    ▄███████   ▄███████   ▄███████   ▄█   █▄    ▄█   █▄",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "███   ███  ███   ███   ███  ███   ███  ███   ███  ███   ███  ███   ███  ███   ███",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "███   ███  ███   ███   ███  ███   ███  ███   ███  ███   █▀   ███   ███  ███   ███",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "███   ███  ███   ███   ███ ▄███▄▄▄███ ▄███▄▄▄██▀  ███       ▄███▄▄▄███▄ ███▄▄▄███",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "███   ███  ███   ███   ███ ▀███▀▀▀███ ▀███▀▀▀▀    ███      ▀▀███▀▀▀███  ▀▀▀▀▀▀███",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "███   ███  ███   ███   ███  ███   ███ ██████████  ███   █▄   ███   ███  ▄██   ███",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "███   ███  ███   ███   ███  ███   ███  ███   ███  ███   ███  ███   ███  ███   ███",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " ▀█████▀    ▀█   ███   █▀   ███   █▀   ███   ███  ███████▀   ███   █▀    ▀█████▀",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
        ]
    }

    fn get_medium_ascii_art(&self) -> Vec<Line<'static>> {
        vec![
            // Using block characters that should render better
            Line::from(vec![Span::styled(
                "  ████████ ██       ██ ███████ ███████  ████████ ██ ██ ███████",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " ██    ██ ██   ██  ██  ██      ██       ██    ██ ████  ██     ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " ████████ ██   ██  ██  █████   ███████  ████████ ██ ██ █████  ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " ██    ██ ██ █ ██ ██   ██           ██  ██    ██ ██ ██ ██     ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                " ██    ██  ██████ ██    ███████ ███████  ██    ██ ██ ██ ███████",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled("", Style::default())]),
            Line::from(vec![Span::styled(
                " ████████ ██ ██ ████████ ████████  ████████ ██  ██ ██    ██",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "██    ██  ████   ██    ██ ██    ██ ██       ██  ██  ██  ██ ",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "████████  ██ ██  ████████ ████████ ██       ███████   ████  ",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "██    ██  ██ ██  ██    ██ ██   ██  ██       ██  ██     ██   ",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "██    ██  ██ ██  ██    ██ ██   ██   ████████ ██  ██     ██   ",
                Style::default()
                    .fg(Color::Indexed(10))
                    .add_modifier(Modifier::BOLD),
            )]),
        ]
    }

    fn get_compact_ascii_art(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(vec![
                Span::styled(
                    "AWESOME ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "OMARCHY",
                    Style::default()
                        .fg(Color::Indexed(10))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                "╔═══════╗ ╔══════╗",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("║ ", Style::default().fg(Color::Cyan)),
                Span::styled("◆ ◆ ◆", Style::default().fg(Color::White)),
                Span::styled(" ║ ║ ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "TUI",
                    Style::default()
                        .fg(Color::Indexed(10))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ║", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![Span::styled(
                "╚═══════╝ ╚══════╝",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
        ]
    }
}
