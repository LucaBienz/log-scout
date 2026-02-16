mod pattern_builder;
mod config;
mod app;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use app::{App, CurrentScreen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App State
    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(f.size());

            match app.current_screen {
                CurrentScreen::FilePicker => {
                    let items: Vec<ListItem> = app.files.iter().map(|path| {
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                        let icon = if path.is_dir() { "📁" } else { "📄" };
                        let display_text = format!("{} {}", icon, file_name);
                        ListItem::new(display_text)
                    }).collect();

                    let items_list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(" Select Log File "))
                        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> ");

                    let mut state = ListState::default();
                    state.select(Some(app.selected_file_index));

                    f.render_stateful_widget(items_list, chunks[0], &mut state);
                }
                CurrentScreen::LogTrainer => {
                    let items: Vec<ListItem> = app.log_lines.iter().map(|line| {
                        ListItem::new(line.as_str())
                    }).collect();

                    let log_list = List::new(items)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" Inspecting: {:?} ", app.selected_log_path.as_ref().unwrap().file_name().unwrap()))
                        )
                        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> ");

                    let mut state = ListState::default();
                    state.select(Some(app.selected_log_index));
                    f.render_stateful_widget(log_list, chunks[0], &mut state);
                }
                CurrentScreen::Exiting => {}
            }
            
            // Draw Footer (Instructions)
            let footer = ratatui::widgets::Paragraph::new("Use ↑/↓ to navigate, ENTER to select, q to quit")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(footer, chunks[1]);

        })?;

        // Input Handling
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.current_screen {
                    CurrentScreen::FilePicker => {
                        match key.code {
                            KeyCode::Up => app.previous_file(),
                            KeyCode::Down => app.next_file(),
                            KeyCode::Enter => app.select_item(),
                            KeyCode::Char('q') => {
                                app.current_screen = CurrentScreen::Exiting;
                                break;
                            },
                            _ => {}
                        }
                    }
                    CurrentScreen::LogTrainer => {
                        match key.code {
                            KeyCode::Up => app.previous_log_line(),
                            KeyCode::Down => app.next_log_line(),
                            KeyCode::Char('q') => {
                                // Go BACK to file picker instead of quitting app
                                app.current_screen = CurrentScreen::FilePicker;
                            },
                            KeyCode::Esc => {
                                app.current_screen = CurrentScreen::FilePicker;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}