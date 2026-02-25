mod pattern_builder;
mod config;
mod app;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use app::{App, CurrentScreen};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                CurrentScreen::LiveMonitor => {
                    // Process any pending live updates
                    app.process_live_updates();
                    
                    // Split screen: live lines on top, matched patterns on bottom
                    let monitor_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                        .split(chunks[0]);

                    // Live log lines
                    let live_items: Vec<ListItem> = app.live_lines.iter().rev().take(50).map(|line| {
                        ListItem::new(line.as_str())
                    }).collect();

                    let live_list = List::new(live_items)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" Live Monitor: {:?} ", 
                                app.selected_log_path.as_ref().unwrap().file_name().unwrap()))
                        );
                    f.render_widget(live_list, monitor_chunks[0]);

                    // Matched patterns
                    let matched_items: Vec<ListItem> = app.matched_lines.iter().rev().take(20).map(|(line, pattern)| {
                        ListItem::new(format!("[{}] {}", pattern, line))
                            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    }).collect();

                    let matched_list = List::new(matched_items)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" Pattern Matches ({}) ", app.matched_lines.len()))
                        );
                    f.render_widget(matched_list, monitor_chunks[1]);
                }
                CurrentScreen::PatternBuilder => {
                    let pattern_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Length(3), 
                            Constraint::Min(0)
                        ])
                        .split(chunks[0]);

                    // Pattern name input
                    let name_paragraph = Paragraph::new(format!("Pattern Name: {}", app.pattern_name))
                        .block(Block::default().borders(Borders::ALL).title(" Pattern Name "));
                    f.render_widget(name_paragraph, pattern_chunks[0]);

                    // Pattern regex input
                    let pattern_paragraph = Paragraph::new(app.current_pattern.as_str())
                        .block(Block::default().borders(Borders::ALL).title(" Regex Pattern "))
                        .wrap(Wrap { trim: false });
                    f.render_widget(pattern_paragraph, pattern_chunks[1]);

                    // Test matches
                    let test_items: Vec<ListItem> = app.test_matches.iter().map(|line| {
                        ListItem::new(line.as_str())
                            .style(Style::default().fg(Color::Green))
                    }).collect();

                    let test_list = List::new(test_items)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" Test Matches ({}) ", app.test_matches.len()))
                        );
                    f.render_widget(test_list, pattern_chunks[2]);
                }
                CurrentScreen::PatternManager => {
                    let patterns = if let Some(profile) = &app.watch_profile {
                        profile.error_patterns.iter().map(|entry| {
                            ListItem::new(format!("Name: {}  |  Re: / {} /", entry.name, entry.pattern))
                        }).collect()
                    } else {
                        vec![]
                    };
                    
                    let list = List::new(patterns)
                        .block(Block::default().borders(Borders::ALL).title(" Manage Patterns "))
                        .highlight_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        .highlight_symbol(">> "); // Indicates deletion
                        
                    let mut state = ListState::default();
                    state.select(Some(app.selected_pattern_index));
                    f.render_stateful_widget(list, chunks[0], &mut state);
                }
                CurrentScreen::Exiting => {}
            }
            
            // Draw Footer (Instructions)
            let footer_text = match app.current_screen {
                CurrentScreen::FilePicker => "↑/↓ navigate, ENTER select, q quit",
                CurrentScreen::LogTrainer => "↑/↓ navigate, ENTER create pattern, l live monitor, q back, ESC back",
                CurrentScreen::LiveMonitor => "p manage patterns, q back to picker, ESC back",
                CurrentScreen::PatternBuilder => "s save pattern, t test pattern, q back, ESC back",
                CurrentScreen::PatternManager => "↑/↓ select, d delete pattern, q/ESC back",
                CurrentScreen::Exiting => "",
            };
            let footer = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
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
                            KeyCode::Enter => app.create_pattern_from_line(),
                            KeyCode::Char('l') => app.start_live_monitoring(),
                            KeyCode::Char('q') => {
                                app.current_screen = CurrentScreen::FilePicker;
                            },
                            KeyCode::Esc => {
                                app.current_screen = CurrentScreen::FilePicker;
                            }
                            _ => {}
                        }
                    }
                    CurrentScreen::LiveMonitor => {
                        match key.code {
                            KeyCode::Char('p') => {
                                app.current_screen = CurrentScreen::PatternManager;
                            },
                            KeyCode::Char('q') => {
                                app.current_screen = CurrentScreen::FilePicker;
                            },
                            KeyCode::Esc => {
                                app.current_screen = CurrentScreen::LogTrainer;
                            }
                            _ => {}
                        }
                    }
                    CurrentScreen::PatternBuilder => {
                        match key.code {
                            KeyCode::Char('s') => {
                                app.save_pattern();
                                app.current_screen = CurrentScreen::LogTrainer;
                            },
                            KeyCode::Char('t') => {
                                app.test_pattern();
                            },
                            KeyCode::Char('q') => {
                                app.current_screen = CurrentScreen::FilePicker;
                            },
                            KeyCode::Esc => {
                                app.current_screen = CurrentScreen::LogTrainer;
                            }
                            _ => {}
                        }
                    }
                    CurrentScreen::PatternManager => {
                        match key.code {
                            KeyCode::Up => app.previous_pattern(),
                            KeyCode::Down => app.next_pattern(),
                            KeyCode::Char('d') => app.delete_selected_pattern(),
                            KeyCode::Char('q') | KeyCode::Esc => app.current_screen = CurrentScreen::LiveMonitor,
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