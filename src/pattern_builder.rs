use regex::Regex;

pub fn generate_regex_from_line(line: &str) -> String {
    // If we can find a structural anchor (log level keyword or bracketed word),
    // build a generic pattern: generalize the prefix, keep the anchor, then `.*`.
    // This way the regex matches any line of that level/type regardless of message.
    if let Some((anchor_start, anchor_end)) = find_anchor(line) {
        let prefix = &line[..anchor_start];
        let anchor = &line[anchor_start..anchor_end];
        let mut pattern = generalize_prefix(prefix);
        pattern.push_str(&regex::escape(anchor));
        pattern.push_str(".*");
        return pattern;
    }

    // Fallback: escape the full line and generalize numbers/timestamps.
    let mut pattern = regex::escape(line);
    pattern = generalize_timestamps_and_numbers(pattern);
    pattern.push_str(".*");
    pattern
}

/// Finds the earliest structural anchor in the line and returns its byte range.
///
/// Two patterns are tried in order of priority:
///   1. Known severity keywords (case-insensitive, word-boundary): ERROR, WARN, …
///   2. Any bracketed identifier: [FAIL], [SEVERE], [MY_CUSTOM_LEVEL], …
///
/// Whichever appears earliest in the line wins.
fn find_anchor(line: &str) -> Option<(usize, usize)> {
    // Pattern 1: common severity keywords – case-insensitive so `error`, `Error`,
    // and `ERROR` all match.
    let keyword_re = Regex::new(
        r"(?i)\b(CRITICAL|WARNING|FATAL|TRACE|DEBUG|ERROR|WARN|INFO|SEVERE|FAIL(?:ED|URE)?|PANIC|EXCEPTION|ALERT|EMERG)\b"
    ).unwrap();

    // Pattern 2: bracket-enclosed word/acronym like [FAIL] [SEVERE] [MY_TYPE]
    let bracket_re = Regex::new(r"\[[A-Za-z][A-Za-z0-9_]+\]").unwrap();

    let mut result: Option<(usize, usize)> = None;

    for re in &[keyword_re, bracket_re] {
        if let Some(m) = re.find(line) {
            if result.map_or(true, |(s, _)| m.start() < s) {
                result = Some((m.start(), m.end()));
            }
        }
    }

    result
}

/// Escape and generalize the prefix (timestamps, numbers) that appears before
/// the anchor.
fn generalize_prefix(prefix: &str) -> String {
    let escaped = regex::escape(prefix);
    generalize_timestamps_and_numbers(escaped)
}

fn generalize_timestamps_and_numbers(mut pattern: String) -> String {
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    pattern = date_re.replace_all(&pattern, r"\d{4}-\d{2}-\d{2}").to_string();

    let time_re = Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
    pattern = time_re.replace_all(&pattern, r"\d{2}:\d{2}:\d{2}").to_string();

    let number_re = Regex::new(r"\b\d+\b").unwrap();
    pattern = number_re.replace_all(&pattern, r"\d+").to_string();

    pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_keyword() {
        let raw = "[2024-02-16] ERROR [1234] Connection Failed";
        let regex = generate_regex_from_line(raw);
        println!("Generated: {}", regex);
        assert!(regex.contains("ERROR.*"), "got: {regex}");
    }

    #[test]
    fn test_lowercase_keyword() {
        let raw = "2024-02-16 10:00:00 error something went wrong";
        let regex = generate_regex_from_line(raw);
        assert!(regex.contains("error.*"), "got: {regex}");
    }

    #[test]
    fn test_bracketed_custom_level() {
        let raw = "[2024-02-16] [SEVERE] disk quota exceeded";
        let regex = generate_regex_from_line(raw);
        // Should anchor on [SEVERE], not on the date bracket
        assert!(regex.contains(r"\[SEVERE\].*"), "got: {regex}");
    }

    #[test]
    fn test_no_anchor() {
        let raw = "some line without a level";
        let regex = generate_regex_from_line(raw);
        assert!(regex.ends_with(".*"), "got: {regex}");
    }
}