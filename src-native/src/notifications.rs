use regex::Regex;

struct CompiledPattern {
    regex: Regex,
    name: String,
}

pub struct DetectionResult {
    pub matched: bool,
    pub pattern: String,
}

pub struct NotificationDetector {
    patterns: Vec<CompiledPattern>,
    keyword_filter: Regex,
}

impl NotificationDetector {
    pub fn new() -> Self {
        let patterns = vec![
            CompiledPattern {
                regex: Regex::new(r"[❯›]\s*$").unwrap(),
                name: String::from("claude-prompt"),
            },
            CompiledPattern {
                regex: Regex::new(r"(?i)\?\s*\(y/n\)").unwrap(),
                name: String::from("claude-confirm"),
            },
            CompiledPattern {
                regex: Regex::new(r"(?i)waiting for").unwrap(),
                name: String::from("claude-waiting"),
            },
            CompiledPattern {
                regex: Regex::new(r"(?i)\(y\) to approve").unwrap(),
                name: String::from("claude-permission"),
            },
            CompiledPattern {
                regex: Regex::new(r"\? for shortcuts").unwrap(),
                name: String::from("codex-prompt"),
            },
            CompiledPattern {
                regex: Regex::new(r"(?i)what should codex do").unwrap(),
                name: String::from("codex-waiting"),
            },
            CompiledPattern {
                regex: Regex::new(r">>>\s*$").unwrap(),
                name: String::from("gemini-prompt"),
            },
        ];

        let keyword_filter =
            Regex::new(r"(?i)[❯›?]|waiting|permission|approve|shortcuts|codex|>>>").unwrap();

        Self {
            patterns,
            keyword_filter,
        }
    }

    pub fn detect(&self, line: &str) -> DetectionResult {
        if !self.keyword_filter.is_match(line) {
            return DetectionResult {
                matched: false,
                pattern: String::new(),
            };
        }

        for pat in &self.patterns {
            if pat.regex.is_match(line) {
                return DetectionResult {
                    matched: true,
                    pattern: pat.name.clone(),
                };
            }
        }

        DetectionResult {
            matched: false,
            pattern: String::new(),
        }
    }
}

pub fn send_desktop_notification(
    terminal_name: &str,
    pattern_name: &str,
) -> anyhow::Result<()> {
    notify_rust::Notification::new()
        .summary("gmux")
        .body(&format!("{}: {}", terminal_name, pattern_name))
        .show()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_claude_prompt() {
        let detector = NotificationDetector::new();
        let result = detector.detect("❯ ");
        assert!(result.matched);
        assert_eq!(result.pattern, "claude-prompt");
    }

    #[test]
    fn detects_claude_confirm() {
        let detector = NotificationDetector::new();
        let result = detector.detect("Do you want to continue? (y/n)");
        assert!(result.matched);
        assert_eq!(result.pattern, "claude-confirm");
    }

    #[test]
    fn no_match_on_normal_output() {
        let detector = NotificationDetector::new();
        let result = detector.detect("$ ls -la");
        assert!(!result.matched);
    }

    #[test]
    fn keyword_filter_rejects_early() {
        let detector = NotificationDetector::new();
        let result = detector.detect("compiling crate version 1.0");
        assert!(!result.matched);
    }

    #[test]
    fn detects_claude_permission() {
        let detector = NotificationDetector::new();
        let result = detector.detect("Press (y) to approve this action");
        assert!(result.matched);
        assert_eq!(result.pattern, "claude-permission");
    }

    #[test]
    fn detects_gemini_prompt() {
        let detector = NotificationDetector::new();
        let result = detector.detect(">>> ");
        assert!(result.matched);
        assert_eq!(result.pattern, "gemini-prompt");
    }

    #[test]
    fn detects_codex_prompt() {
        let detector = NotificationDetector::new();
        let result = detector.detect("? for shortcuts");
        assert!(result.matched);
        assert_eq!(result.pattern, "codex-prompt");
    }

    #[test]
    fn detects_codex_waiting() {
        let detector = NotificationDetector::new();
        let result = detector.detect("What should Codex do next?");
        assert!(result.matched);
        assert_eq!(result.pattern, "codex-waiting");
    }

    #[test]
    fn detects_claude_waiting() {
        let detector = NotificationDetector::new();
        let result = detector.detect("Waiting for your response...");
        assert!(result.matched);
        assert_eq!(result.pattern, "claude-waiting");
    }
}
