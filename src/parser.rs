use std::collections::HashMap;
use std::fs;
use regex::Regex;

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

#[derive(Debug, Clone)]
pub struct ThethaCoreConfig {
    pub sections: HashMap<String, HashMap<String, Value>>,
}

impl ThethaCoreConfig {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    pub fn parse_from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|_| format!("Could not read file: {}", path))?;
        Self::parse(&content)
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        let mut config = ThethaCoreConfig::new();
        let mut current_section: Option<String> = None;
        let section_regex = Regex::new(r"^<([\w<>]+)>$").unwrap();
        let kv_regex = Regex::new(r"^(\w+)\s*==\s*(.+)$").unwrap();
        let boolean_null_regex = Regex::new(r"^(True|False|Null)$").unwrap();

        for line in input.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("#") || trimmed.starts_with("//") {
                continue;
            }

            if let Some(caps) = section_regex.captures(trimmed) {
                let section_name = caps.get(1).unwrap().as_str().to_string();
                config.sections.entry(section_name.clone()).or_insert(HashMap::new());
                current_section = Some(section_name);
                continue;
            }

            if let Some(caps) = kv_regex.captures(trimmed) {
                let key = caps.get(1).unwrap().as_str().to_string();
                let value_str = caps.get(2).unwrap().as_str().trim();

                let value = if boolean_null_regex.is_match(value_str) {
                    match value_str {
                        "True" => Value::Boolean(true),
                        "False" => Value::Boolean(false),
                        "Null" => Value::Null,
                        _ => unreachable!(),
                    }
                } else if value_str.starts_with('"') && value_str.ends_with('"') {
                    Value::String(value_str[1..value_str.len() - 1].to_string())
                } else if let Ok(num) = value_str.parse::<i64>() {
                    Value::Integer(num)
                } else if let Ok(num) = value_str.parse::<f64>() {
                    Value::Float(num)
                } else {
                    return Err(format!("Invalid value format: {}", value_str));
                };

                if let Some(ref section) = current_section {
                    config.sections.get_mut(section).unwrap().insert(key, value);
                } else {
                    return Err("Key-value pair found outside of a section".to_string());
                }
            } else {
                return Err(format!("Syntax error on line: {}", trimmed));
            }
        }

        Ok(config)
    }
}
