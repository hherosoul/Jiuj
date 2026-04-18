use crate::constants::*;
use regex::Regex;

#[derive(Clone)]
struct SanitizeRule {
    name: String,
    pattern: Regex,
    placeholder: String,
}

#[derive(Clone)]
pub struct Sanitizer {
    rules: Vec<SanitizeRule>,
}

impl Sanitizer {
    pub fn new() -> Self {
        let rules = vec![
            SanitizeRule {
                name: "手机号".to_string(),
                pattern: Regex::new(r#"1[3-9]\d{9}"#).unwrap(),
                placeholder: "[手机号]".to_string(),
            },
            SanitizeRule {
                name: "身份证号".to_string(),
                pattern: Regex::new(r#"\d{17}[\dXx]"#).unwrap(),
                placeholder: "[身份证号]".to_string(),
            },
            SanitizeRule {
                name: "银行卡号".to_string(),
                pattern: Regex::new(r#"\d{16,19}"#).unwrap(),
                placeholder: "[银行卡号]".to_string(),
            },
            SanitizeRule {
                name: "邮箱地址".to_string(),
                pattern: Regex::new(r#"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"#).unwrap(),
                placeholder: "[邮箱地址]".to_string(),
            },
            SanitizeRule {
                name: "IP地址".to_string(),
                pattern: Regex::new(r#"(?:\d{1,3}\.){3}\d{1,3}"#).unwrap(),
                placeholder: "[IP地址]".to_string(),
            },
        ];

        Sanitizer { rules }
    }

    pub fn sanitize(&self, text: &str) -> String {
        let mut result = text.to_string();
        for rule in &self.rules {
            result = rule.pattern.replace_all(&result, &rule.placeholder).to_string();
        }
        result
    }

    pub fn truncate(&self, body: &str) -> (String, bool) {
        let chars: Vec<char> = body.chars().collect();
        if chars.len() > MAX_EMAIL_BODY_LENGTH {
            let truncated: String = chars[..MAX_EMAIL_BODY_LENGTH].iter().collect();
            (truncated, true)
        } else {
            (body.to_string(), false)
        }
    }

    pub fn process(&self, body: &str) -> (String, bool) {
        let text = Self::strip_html(body);
        let sanitized = self.sanitize(&text);
        self.truncate(&sanitized)
    }

    pub fn get_rules(&self) -> Vec<(&str, &str, &str)> {
        self.rules
            .iter()
            .map(|r| (r.name.as_str(), r.pattern.as_str(), r.placeholder.as_str()))
            .collect()
    }

    fn strip_html(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }
        result
    }
}

impl Default for Sanitizer {
    fn default() -> Self {
        Self::new()
    }
}
