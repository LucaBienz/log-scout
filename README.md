# Log Scout

A real-time log monitoring tool with pattern matching and desktop notifications built in Rust.

## Features

🔍 **File Browser** - Navigate and select log files with an intuitive TUI
📊 **Log Viewer** - View the last 1000 lines of selected log files  
⚡ **Real-time Monitoring** - Watch log files for new entries as they're written
🎯 **Pattern Matching** - Create regex patterns to detect specific log events
🔧 **Pattern Builder** - Generate patterns from example log lines automatically
💾 **Profile Management** - Save and load watch profiles with custom patterns
🔔 **Match Highlighting** - Highlight lines that match your error patterns

## Usage

### Basic Navigation
- **↑/↓**: Navigate through files and log lines
- **ENTER**: Select file or create pattern from line  
- **q**: Quit or go back
- **ESC**: Go back to previous screen

### Log Viewer Mode
- **ENTER**: Create a pattern from the currently selected log line
- **l**: Start live monitoring of the current file
- **↑/↓**: Navigate through log lines

### Pattern Builder Mode
- **s**: Save the current pattern to the watch profile
- **t**: Test the pattern against loaded log lines
- View matching lines highlighted in green

### Live Monitor Mode
- Real-time display of new log entries
- Bottom panel shows pattern matches as they occur
- Automatic pattern matching against all saved patterns

## Architecture

- **TUI Interface**: Built with `ratatui` for responsive terminal UI
- **File Watching**: Uses `linemux` for efficient real-time file monitoring  
- **Pattern Matching**: Powered by the `regex` crate
- **Configuration**: JSON-based watch profiles with `serde`
- **Async Support**: Tokio runtime for background file monitoring

## Getting Started

```bash
# Build the project
cargo build --release

# Run log-scout
./target/release/log_scout

# Or run in development
cargo run
```

## Watch Profiles

Profiles are automatically saved as JSON files in the current directory:
- Pattern format: `name:regex_pattern`  
- Automatic profile creation based on selected log file
- Profiles include file path and all created patterns

## Pattern Generation

The pattern builder can automatically generate regex patterns from example log lines:
- Converts timestamps to generic patterns (`\d{4}-\d{2}-\d{2}`)
- Replaces PIDs and numbers with `\d+` 
- Escapes special regex characters
- Adds wildcard matching for variable content

## Dependencies

- `ratatui` - Terminal user interface
- `crossterm` - Cross-platform terminal manipulation
- `linemux` - Real-time file monitoring
- `regex` - Pattern matching engine
- `tokio` - Async runtime
- `serde` - Configuration serialization
- `notify-rust` - Desktop notifications (planned)
- `walkdir` - Directory traversal

## Next Steps

- [ ] Desktop notifications when patterns match
- [ ] Pattern editing in TUI
- [ ] Multiple file monitoring
- [ ] Log filtering and search
- [ ] Export/import watch profiles
- [ ] Performance metrics and statistics