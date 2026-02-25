# Log Scout

Tired of manually monitoring log files and missing critical errors? Log Scout is a real-time log monitoring tool that watches your files and alerts you the moment something goes wrong.

![Language](https://img.shields.io/badge/language-Rust-dca282.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-gray.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.5.0-success.svg)

## Why Log Scout?

- **Tired of missing production errors?** Log Scout monitors your files 24/7 and highlights matches in real-time.

- **Tired of writing complex regex patterns?** Select any log line and Log Scout automatically generates the pattern for you.

- **Tired of checking multiple log files manually?** Browse files with an intuitive interface and switch between them instantly.

- **Tired of command-line tools that are hard to use?** Clean terminal interface with simple keyboard shortcuts.

## Features

- **File Browser** - Navigate and select log files with an intuitive TUI
- **Log Viewer** - View the last 1000 lines of any log file  
- **Real-time Monitoring** - Watch log files for new entries as they're written
- **Desktop Notifications** - Get native OS alerts when a pattern matches (Windows/Linux/macOS)
- **Pattern Builder** - Generate regex patterns from example log lines automatically
- **Pattern Manager** - View and delete active patterns on the fly
- **Auto-Start** - Automatically resumes monitoring if a saved profile exists
- **Profile Management** - Save, load, and reset watch profiles

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/log_scout

# Or run in development
cargo run
```

## How to Use

### Navigate Files
- Use **up/down arrows** to browse files and directories
- Press **ENTER** to select a file or enter a directory
- Press **q** to quit

### Monitor Logs
1. **Select a log file** from the browser
2. **Browse historical log lines** with up/down arrows
3. **Find an error line** you want to monitor for
4. **Press ENTER** on that line to create a pattern
5. **Press s** to save the pattern
6. **Press l** to start live monitoring

**Note:** If a profile already exists for a log file, Log Scout will skip these steps and auto-start the Live Monitor.

### Managing Patterns
- While monitoring, press **p** to open the Pattern Manager
- Use **up/down** to select a pattern
- Press **d** to delete a pattern instantly
- Press **q** to return to monitoring

### Watch Profiles & Auto-Start
Profiles are automatically saved as JSON files in the app directory.
- **Auto-Start**: If Log Scout finds a profile on startup, it jumps immediately to the Live Monitor.
- **Reset**: To clear a profile and start over, press **r** while in the Live Monitor.

## Pattern Generation

Select any log line in the viewer and Log Scout builds a regex that matches all similar lines — not just that one message.

The generator looks for a **structural anchor** (the part that marks a line as an error) and makes everything after it generic:

```
[2024-02-16 14:23:45] ERROR [1234] Connection failed
                ↓
\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\] ERROR.*
```

### What it detects as an anchor

| Format | Example | Detected anchor |
|--------|---------|-----------------|
| Common severity keywords (any case) | `ERROR`, `error`, `Warning`, `FATAL` | the keyword itself |
| Extended keywords | `CRITICAL`, `SEVERE`, `PANIC`, `EXCEPTION`, `ALERT`, `TRACE`, `EMERG`, `FAILED`, `FAILURE` | the keyword itself |
| Bracket-enclosed identifiers | `[FAIL]`, `[SEVERE]`, `[MY_LEVEL]` | the full `[…]` token |

Once the anchor is found, the prefix (timestamps, PIDs, brackets) is generalized and the rest of the line is replaced with `.*`, so the pattern fires on **any** line at that level — not just the one you selected.

### What it generalizes in the prefix

- **Dates** `2024-02-16` → `\d{4}-\d{2}-\d{2}`
- **Times** `14:23:45` → `\d{2}:\d{2}:\d{2}`
- **Numbers / PIDs** `1234` → `\d+`
- **Special characters** `[`, `]`, `.`, `(` etc. are escaped automatically

### What it won't detect

- **Free-form severity words** that don't match the keyword list and aren't bracketed — e.g. `severity=high` or `level: urgent`. The pattern will still be generated but will match that exact message rather than all lines of that type.
- **Multi-line log entries** — only the selected line is used to build the pattern.
- **Non-ASCII or emoji status indicators** — anchoring only works on ASCII word characters.



## Technical Details

**Built with:**
- `ratatui` - Terminal user interface framework
- `crossterm` - Cross-platform terminal control
- `linemux` - Efficient real-time file monitoring
- `regex` - Pattern matching engine
- `tokio` - Async runtime for background monitoring
- `notify-rust` - Cross-platform desktop notifications

## Keyboard Shortcuts

### File Browser
| Key | Action |
|-----|--------|
| ↑/↓ | Navigate files |
| ENTER | Select file/directory |
| q | Quit |

### Log Viewer
| Key | Action |
|-----|--------|
| ↑/↓ | Navigate log lines |
| ENTER | Create pattern from selected line |
| l | Start live monitoring |
| q | Back to file browser |

### Live Monitor
| Key | Action |
|-----|--------|
| p | Open Pattern Manager |
| r | Reset (Delete Profile & Restart) |
| q | Back to file browser |
| ESC | Back to log viewer |

### Pattern Manager
| Key | Action |
|-----|--------|
| ↑/↓ | Select pattern |
| d | Delete selected pattern |
| q | Back to monitoring |

### Pattern Builder
| Key | Action |
|-----|--------|
| s | Save pattern |
| t | Test pattern |
| q | Back to file browser |
| ESC | Back to log viewer |

## Requirements

- Rust 1.70+
- Terminal with color support

## Future Enhancements

- Multiple file monitoring simultaneously
- Log filtering and search functionality
- Pattern editing within the TUI
- Export/import watch profiles
- Performance metrics and statistics
