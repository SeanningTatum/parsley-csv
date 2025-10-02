use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::types::Table;

/// Parse SQL file and extract table schemas and data
pub fn parse_sql_file<P: AsRef<Path>>(sql_file_path: P) -> Result<(Vec<Table>, String)> {
    let content = fs::read_to_string(sql_file_path)
        .context("Failed to read SQL file")?;
    
    let mut tables = Vec::new();
    
    // Generic regex to match any CREATE TABLE statement
    let create_table_regex = Regex::new(r"(?i)(?s)CREATE TABLE\s+(\w+)\s*\((.*?)\)(?:\s|;)")?;
    
    for captures in create_table_regex.captures_iter(&content) {
        let table_name = captures.get(1).unwrap().as_str();
        let columns_text = captures.get(2).unwrap().as_str();
        let columns = parse_table_columns(columns_text);
        
        if !columns.is_empty() {
            let num_columns = columns.len();
            println!("Found table: {} with {} columns", table_name, num_columns);
            tables.push(Table {
                name: table_name.to_string(),
                columns,
            });
        }
    }
    
    Ok((tables, content))
}

/// Parse column definitions from CREATE TABLE statement
pub fn parse_table_columns(columns_text: &str) -> Vec<String> {
    let mut columns = Vec::new();
    
    // Join all lines and split by comma to handle multi-line column definitions
    let joined = columns_text.lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join(" ");
    
    // Split by comma and extract column names
    for part in joined.split(',') {
        let part = part.trim();
        if !part.is_empty() 
            && !part.starts_with("FOREIGN KEY") 
            && !part.starts_with("PRIMARY KEY") {
            // Extract the column name (first word)
            if let Some(first_word) = part.split_whitespace().next() {
                let col_name = first_word.trim();
                if !col_name.is_empty() && col_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    columns.push(col_name.to_string());
                }
            }
        }
    }
    
    columns
}

/// Extract INSERT VALUES from SQL for a specific table
pub fn extract_insert_values(content: &str, table_name: &str) -> Result<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    
    // Pattern to match INSERT statements - try both with and without quotes
    // This handles different SQL dialects (PostgreSQL uses ", MySQL uses `, SQLite supports both)
    let patterns = vec![
        format!(r#"(?s)INSERT INTO "{}" VALUES\((.*?)\);"#, regex::escape(table_name)),
        format!(r"(?s)INSERT INTO {} VALUES\((.*?)\);", regex::escape(table_name)),
        format!(r#"(?s)INSERT INTO `{}` VALUES\((.*?)\);"#, regex::escape(table_name)),
    ];
    
    for pattern in patterns.iter() {
        match Regex::new(pattern) {
            Ok(insert_regex) => {
                let matches = insert_regex.captures_iter(content);
                for captures in matches {
                    let values_str = captures.get(1).unwrap().as_str();
                    let values_str = handle_replace_function(values_str);
                    let values = parse_values(&values_str);
                    rows.push(values);
                }
                if !rows.is_empty() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to compile regex pattern: {}", e);
            }
        }
    }
    
    Ok(rows)
}

/// Handle replace() function in SQL values
fn handle_replace_function(values_str: &str) -> String {
    if !values_str.contains("replace(") {
        return values_str.to_string();
    }
    
    let replace_regex = Regex::new(r"(?s)replace\('(.*?)',.*?\)").unwrap();
    
    replace_regex.replace_all(values_str, |caps: &regex::Captures| {
        let json_content = caps.get(1)
            .map_or("", |m| m.as_str())
            .replace("\\'", "'")
            .replace("\\n", "\n");
        format!("'{}'", json_content)
    }).to_string()
}

/// Parse comma-separated values from INSERT statement
fn parse_values(values_str: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current_value = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    let chars: Vec<char> = values_str.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let char = chars[i];
        
        if !in_quotes {
            if char == '\'' || char == '"' {
                in_quotes = true;
                quote_char = char;
                current_value.push(char);
            } else if char == ',' {
                values.push(current_value.trim().to_string());
                current_value.clear();
            } else {
                current_value.push(char);
            }
        } else {
            if char == quote_char {
                // Check if it's an escaped quote
                if i + 1 < chars.len() && chars[i + 1] == quote_char {
                    current_value.push(char);
                    current_value.push(char);
                    i += 1; // Skip the next quote
                } else {
                    in_quotes = false;
                    quote_char = ' ';
                    current_value.push(char);
                }
            } else {
                current_value.push(char);
            }
        }
        
        i += 1;
    }
    
    // Add the last value
    if !current_value.is_empty() {
        values.push(current_value.trim().to_string());
    }
    
    // Clean up values
    values.into_iter()
        .map(|val| clean_value(val))
        .collect()
}

/// Clean up a single value (remove quotes, unescape)
fn clean_value(val: String) -> String {
    let val = val.trim();
    
    // Remove surrounding quotes if present
    let cleaned = if (val.starts_with('\'') && val.ends_with('\'')) || 
                    (val.starts_with('"') && val.ends_with('"')) {
        let inner = &val[1..val.len()-1];
        // Unescape doubled quotes
        inner.replace("''", "'").replace("\"\"", "\"")
    } else {
        val.to_string()
    };
    
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_table_columns() {
        let columns_text = r#"
            id TEXT PRIMARY KEY,
            callSessionId TEXT NOT NULL,
            type TEXT NOT NULL,
            FOREIGN KEY (callSessionId) REFERENCES CallSession(id)
        "#;
        
        let columns = parse_table_columns(columns_text);
        assert_eq!(columns, vec!["id", "callSessionId", "type"]);
    }
    
    #[test]
    fn test_clean_value() {
        assert_eq!(clean_value("'test'".to_string()), "test");
        assert_eq!(clean_value("\"test\"".to_string()), "test");
        assert_eq!(clean_value("'test''s'".to_string()), "test's");
        assert_eq!(clean_value("test".to_string()), "test");
    }
    
    #[test]
    fn test_parse_values() {
        let values_str = "'value1', 'value2', 'value''3'";
        let values = parse_values(values_str);
        assert_eq!(values, vec!["value1", "value2", "value'3"]);
    }
}

