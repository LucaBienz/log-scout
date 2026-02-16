use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub enum CurrentScreen {
    FilePicker,
    LogTrainer, 
    Exiting,
}

pub struct App {
    pub current_screen: CurrentScreen,
    
    // STATE
    pub current_dir: PathBuf,
    pub files: Vec<PathBuf>, // List of files in current dir
    pub selected_file_index: usize, // Which file is highlighted

    pub selected_log_path: Option<PathBuf>, // The final choice
    pub log_lines: Vec<String>,
    pub selected_log_index: usize,
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

}