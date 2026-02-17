use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use crate::config::WatchProfile;
use crate::pattern_builder::generate_regex_from_line;
use regex::Regex;
use std::sync::mpsc;
use std::thread;
use linemux::MuxedLines;

pub enum CurrentScreen {
    FilePicker,
    LogTrainer, 
    LiveMonitor,
    PatternBuilder,
    Exiting,
}

pub struct App {
    pub current_screen: CurrentScreen,
    
    // File browser state
    pub current_dir: PathBuf,
    pub files: Vec<PathBuf>,
    pub selected_file_index: usize,

    // Log viewer state
    pub selected_log_path: Option<PathBuf>,
    pub log_lines: Vec<String>,
    pub selected_log_index: usize,

    // Live monitor state
    pub live_lines: Vec<String>,
    pub matched_lines: Vec<(String, String)>, // (line, pattern_name)
    pub watch_profile: Option<WatchProfile>,
    pub compiled_patterns: Vec<(String, Regex)>, // (name, regex)
    
    // Pattern builder state
    pub current_pattern: String,
    pub pattern_name: String,
    pub test_matches: Vec<String>,
    
    // Communication channel for live updates
    pub line_receiver: Option<mpsc::Receiver<String>>,
}

impl App {
    pub fn new() -> App {
        let start_dir = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let mut app = App {
            current_screen: CurrentScreen::FilePicker,
            current_dir: start_dir,
            files: Vec::new(),
            selected_file_index: 0,

            selected_log_path: None,
            log_lines: Vec::new(),
            selected_log_index: 0,

            live_lines: Vec::new(),
            matched_lines: Vec::new(),
            watch_profile: None,
            compiled_patterns: Vec::new(),
            
            current_pattern: String::new(),
            pattern_name: String::new(),
            test_matches: Vec::new(),
            
            line_receiver: None,
        };
        app.refresh_files();
        app
    }

    // Reads the current directory and populates 'self.files'
    pub fn refresh_files(&mut self) {
        self.files.clear();
        
        // Add ".." (Go Up) option if we are not at root
        if self.current_dir.parent().is_some() {
             self.files.push(self.current_dir.join(".."));
        }

        // Read directory
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                self.files.push(entry.path());
            }
        }
        
        // Sort cleanly (Directories first, then files)
        self.files.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });
        
        self.selected_file_index = 0;
    }

    // Navigation Logic
    pub fn next_file(&mut self) {
        if self.selected_file_index < self.files.len().saturating_sub(1) {
            self.selected_file_index += 1;
        }
    }

    pub fn previous_file(&mut self) {
        if self.selected_file_index > 0 {
            self.selected_file_index -= 1;
        }
    }

    // Navigation for the Log Trainer
    pub fn next_log_line(&mut self) {
        if !self.log_lines.is_empty() && self.selected_log_index < self.log_lines.len() - 1 {
            self.selected_log_index += 1;
        }
    }

    pub fn previous_log_line(&mut self) {
        if self.selected_log_index > 0 {
            self.selected_log_index -= 1;
        }
    }

    // "Enter" Key Logic
    pub fn select_item(&mut self) {
        if self.files.is_empty() { return; }

        let target = self.files[self.selected_file_index].clone();

        // Check if it is a directory OR the special ".." entry
        if target.is_dir() || target.ends_with("..") {
            if target.ends_with("..") {
                // Logic to go up one level
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                }
            } else {
                // Logic to enter a normal directory
                self.current_dir = target;
            }
            self.refresh_files();
        } else {
            // It's a file! Select it and switch screens
            self.selected_log_path = Some(target.clone());
            self.load_log_file(target);
            self.current_screen = CurrentScreen::LogTrainer;
        }
    }

    fn load_log_file(&mut self, path: PathBuf) {
        self.log_lines.clear();
        self.selected_log_index = 0;

        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            let all_lines: Vec<String> = reader.lines()
                .filter_map(Result::ok)
                .collect();

            let start = all_lines.len().saturating_sub(1000);
            self.log_lines = all_lines[start..].to_vec();
        }
    }

    // Start live monitoring of the selected log file
    pub fn start_live_monitoring(&mut self) {
        if let Some(path) = &self.selected_log_path {
            let path = path.clone();
            let (tx, rx) = mpsc::channel();
            
            // Spawn background thread for file monitoring
            thread::spawn(move || {
                let mut lines = MuxedLines::new().expect("Failed to create muxed lines");
                lines.add_file(&path).expect("Failed to add file to watcher");
                
                loop {
                    match lines.next_line() {
                        Ok(Some(line)) => {
                            if tx.send(line.line().to_string()).is_err() {
                                break; // Channel closed
                            }
                        }
                        Ok(None) => {
                            // No more lines, continue monitoring
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        Err(_) => break, // Error occurred
                    }
                }
            });
            
            self.line_receiver = Some(rx);
            self.current_screen = CurrentScreen::LiveMonitor;
        }
    }

    // Process incoming lines from live monitoring
    pub fn process_live_updates(&mut self) {
        if let Some(rx) = &self.line_receiver {
            while let Ok(line) = rx.try_recv() {
                self.live_lines.push(line.clone());
                
                // Check line against all compiled patterns
                for (pattern_name, regex) in &self.compiled_patterns {
                    if regex.is_match(&line) {
                        self.matched_lines.push((line.clone(), pattern_name.clone()));
                        // TODO: Send desktop notification
                    }
                }
                
                // Keep only last 1000 lines for performance
                if self.live_lines.len() > 1000 {
                    self.live_lines.remove(0);
                }
            }
        }
    }

    // Create pattern from currently selected log line
    pub fn create_pattern_from_line(&mut self) {
        if !self.log_lines.is_empty() {
            let selected_line = &self.log_lines[self.selected_log_index];
            self.current_pattern = generate_regex_from_line(selected_line);
            self.pattern_name = "New Pattern".to_string();
            self.test_pattern();
            self.current_screen = CurrentScreen::PatternBuilder;
        }
    }

    // Test current pattern against log lines
    pub fn test_pattern(&mut self) {
        self.test_matches.clear();
        if !self.current_pattern.is_empty() {
            if let Ok(regex) = Regex::new(&self.current_pattern) {
                for line in &self.log_lines {
                    if regex.is_match(line) {
                        self.test_matches.push(line.clone());
                    }
                }
            }
        }
    }

    // Save current pattern to watch profile
    pub fn save_pattern(&mut self) {
        if self.watch_profile.is_none() {
            let profile_name = self.selected_log_path
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "default".to_string());
                
            self.watch_profile = Some(WatchProfile {
                name: profile_name,
                file_path: self.selected_log_path.as_ref().unwrap().to_string_lossy().to_string(),
                error_patterns: Vec::new(),
            });
        }

        if let Some(profile) = &mut self.watch_profile {
            profile.error_patterns.push(format!("{}:{}", self.pattern_name, self.current_pattern));
            
            // Recompile patterns
            self.compile_patterns();
            
            // Save to file
            let filename = format!("{}.json", profile.name);
            if let Err(e) = profile.save(&filename) {
                eprintln!("Failed to save profile: {}", e);
            }
        }
    }

    // Compile all patterns in the watch profile
    pub fn compile_patterns(&mut self) {
        self.compiled_patterns.clear();
        if let Some(profile) = &self.watch_profile {
            for pattern_str in &profile.error_patterns {
                if let Some((name, pattern)) = pattern_str.split_once(':') {
                    if let Ok(regex) = Regex::new(pattern) {
                        self.compiled_patterns.push((name.to_string(), regex));
                    }
                }
            }
        }
    }

    // Load existing watch profile
    pub fn load_watch_profile(&mut self, filename: &str) {
        if let Ok(profile) = WatchProfile::load(filename) {
            self.watch_profile = Some(profile);
            self.compile_patterns();
        }
    }
}