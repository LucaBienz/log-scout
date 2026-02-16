use regex::Regex;

pub fn generate_regex_from_line(line: &str) -> String {
    // Escape special regex characters first (like [ ] ( ) . *)
    let mut pattern = regex::escape(line);

    // Replace Timestamp-like numbers (YYYY-MM-DD)
    // We look for 4 digits, hyphen, 2 digits, hyphen, 2 digits
    let date_re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    pattern = date_re.replace_all(&pattern, r"\d{4}-\d{2}-\d{2}").to_string();

    // Replace Time-like numbers (HH:MM:SS)
    let time_re = Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
    pattern = time_re.replace_all(&pattern, r"\d{2}:\d{2}:\d{2}").to_string();
    
    // Replace PIDs or IDs (integers inside brackets or standalone)
    // This is tricky: we want to replace specific numbers but keep the structure
    let number_re = Regex::new(r"\b\d+\b").unwrap();
    pattern = number_re.replace_all(&pattern, r"\d+").to_string();

    // Allow trailing text
    // If the line implies an error message, we usually want to match anything after
    pattern.push_str(".*");

    pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_builder() {
        let raw = "[2024-02-16] ERROR [1234] Connection Failed";
        let regex = generate_regex_from_line(raw);
        println!("Generated: {}", regex);
        // Expected: \[2024\-02\-16\] ERROR \[\d+\] Connection Failed.*
        // (Note: The exact output depends on how aggressive we regex::escape)
    }
}