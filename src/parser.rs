use regex::Regex;
use std::collections::HashMap;
use std::fs;

/// Represents a value in a ThethaCore configuration.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Represents the entire ThethaCore configuration.
#[derive(Debug, Clone)]
pub struct ThethaCoreConfig {
    /// Keys are section paths (e.g., "database" or "database/advanced").
    pub sections: HashMap<String, HashMap<String, Value>>,
}

impl ThethaCoreConfig {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    /// Parse configuration from a file path.
    pub fn parse_from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|_| format!("❌ Error: Could not read file '{}'", path))?;
        Self::parse(&content)
    }

    /// Parse a configuration from an input string.
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut config = ThethaCoreConfig::new();
        // current_sections holds nested section names.
        let mut current_sections: Vec<String> = Vec::new();

        // Updated regex: capture anything until the first ">".
        let section_regex = Regex::new(r"^<([^>]+)>$").unwrap();
        let kv_regex = Regex::new(r"^(\w+)\s*==\s*(.+)$").unwrap();

        for (line_num, line) in input.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments.
            if trimmed.is_empty() || trimmed.starts_with("#") || trimmed.starts_with("//") {
                continue;
            }

            // Section header
            if let Some(caps) = section_regex.captures(trimmed) {
                let section_text = caps.get(1).unwrap().as_str();
                // Split nested section names on '<'
                current_sections = section_text
                    .split('<')
                    .map(|s| s.trim().to_string())
                    .collect();

                // Create a single section key by joining nested names with "/"
                let section_key = current_sections.join("/");
                config.sections.entry(section_key).or_insert(HashMap::new());
                continue;
            }

            // Key-Value pair
            if let Some(caps) = kv_regex.captures(trimmed) {
                let key = caps.get(1).unwrap().as_str().to_string();
                let value_str = caps.get(2).unwrap().as_str().trim();

                let value = parse_value(value_str, line_num + 1)?;

                // Ensure we're inside a section.
                if current_sections.is_empty() {
                    return Err(format!(
                        "❌ Error on line {}: Key-value pair found outside of a section",
                        line_num + 1
                    ));
                }
                let section_key = current_sections.join("/");
                if let Some(section) = config.sections.get_mut(&section_key) {
                    section.insert(key, value);
                } else {
                    return Err(format!(
                        "❌ Error on line {}: Section '{}' not initialized",
                        line_num + 1,
                        section_key
                    ));
                }
            } else {
                return Err(format!("❌ Syntax error on line {}: '{}'", line_num + 1, trimmed));
            }
        }

        Ok(config)
    }
}

/// Parse a value string into a Value, with detailed error messages.
fn parse_value(value_str: &str, line_num: usize) -> Result<Value, String> {
    // Precompiled regex patterns.
    let boolean_null_regex = Regex::new(r"^(True|False|Null)$").unwrap();
    let array_regex = Regex::new(r"^\[(.*)\]$").unwrap();
    let object_regex = Regex::new(r"^\{(.*)\}$").unwrap();

    // Check for boolean or null.
    if boolean_null_regex.is_match(value_str) {
        match value_str {
            "True" => return Ok(Value::Boolean(true)),
            "False" => return Ok(Value::Boolean(false)),
            "Null" => return Ok(Value::Null),
            _ => unreachable!(),
        }
    }
    // String literal: must be enclosed in double quotes.
    else if value_str.starts_with('"') && value_str.ends_with('"') {
        return Ok(Value::String(
            value_str[1..value_str.len() - 1].to_string(),
        ));
    }
    // Try parsing as integer.
    else if let Ok(num) = value_str.parse::<i64>() {
        return Ok(Value::Integer(num));
    }
    // Try parsing as float.
    else if let Ok(num) = value_str.parse::<f64>() {
        return Ok(Value::Float(num));
    }
    // Array: [item1, item2, ...]
    else if let Some(caps) = array_regex.captures(value_str) {
        let items_str = caps.get(1).unwrap().as_str();
        let items: Result<Vec<Value>, String> = if items_str.trim().is_empty() {
            Ok(vec![])
        } else {
            items_str
                .split(',')
                .map(|s| parse_value(s.trim(), line_num))
                .collect()
        };
        return items.map(Value::Array);
    }
    // Object: { key1 == value1, key2 == value2 }
    else if let Some(caps) = object_regex.captures(value_str) {
        let content = caps.get(1).unwrap().as_str();
        let mut object = HashMap::new();
        if content.trim().is_empty() {
            return Ok(Value::Object(object));
        }
        for pair in content.split(',') {
            let kv: Vec<&str> = pair.split("==").map(|s| s.trim()).collect();
            if kv.len() != 2 {
                return Err(format!(
                    "❌ Syntax error on line {}: Invalid object pair '{}'",
                    line_num, pair
                ));
            }
            // Optionally remove surrounding quotes from keys.
            let key = if kv[0].starts_with('"') && kv[0].ends_with('"') {
                &kv[0][1..kv[0].len() - 1]
            } else {
                kv[0]
            };
            let val = parse_value(kv[1], line_num)?;
            object.insert(key.to_string(), val);
        }
        return Ok(Value::Object(object));
    }

    Err(format!(
        "❌ Syntax error on line {}: Unable to parse value '{}'",
        line_num, value_str
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config() {
        let input = r#"
        <general>
        app_name == "TestApp"
        version == 1.0
        enabled == True
        "#;
        let config = ThethaCoreConfig::parse(input).unwrap();
        let general = config.sections.get("general").unwrap();
        assert_eq!(
            general.get("app_name"),
            Some(&Value::String("TestApp".to_string()))
        );
    }

    #[test]
    fn test_nested_sections() {
        let input = r#"
        <database<advanced>>
        pool_size == 10
        timeout == 30
        "#;
        let config = ThethaCoreConfig::parse(input).unwrap();
        // Expect nested section to be stored as "database/advanced"
        let section = config.sections.get("database/advanced").unwrap();
        assert_eq!(section.get("pool_size"), Some(&Value::Integer(10)));
        assert_eq!(section.get("timeout"), Some(&Value::Integer(30)));
    }

    #[test]
    fn test_array_parsing() {
        let input = r#"
        <data>
        items == ["one", "two", "three"]
        "#;
        let config = ThethaCoreConfig::parse(input).unwrap();
        if let Some(Value::Array(arr)) = config.sections.get("data").unwrap().get("items") {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::String("one".to_string()));
        } else {
            panic!("Failed to parse array");
        }
    }

    #[test]
    fn test_object_parsing() {
        let input = r#"
        <api>
        headers == { "Authorization" == "Bearer token", "Content-Type" == "application/json" }
        "#;
        let config = ThethaCoreConfig::parse(input).unwrap();
        if let Some(Value::Object(obj)) = config.sections.get("api").unwrap().get("headers") {
            assert_eq!(
                obj.get("Authorization"),
                Some(&Value::String("Bearer token".to_string()))
            );
            assert_eq!(
                obj.get("Content-Type"),
                Some(&Value::String("application/json".to_string()))
            );
        } else {
            panic!("Failed to parse object");
        }
    }
}
