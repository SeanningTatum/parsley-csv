use anyhow::{Context, Result};
use csv::Writer;
use rayon::prelude::*;
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use chrono::{NaiveDate, NaiveDateTime, DateTime, FixedOffset};

/// Represents a database table with its name and column names
#[derive(Debug)]
struct Table {
    name: String,
    columns: Vec<String>,
}

/// Represents a date filter configuration
#[derive(Debug, Clone)]
struct DateFilter {
    column_name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

fn main() -> Result<()> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if SQL file path was provided
    if args.len() < 2 {
        eprintln!("Usage: {} <sql_file> [--date-filter <column_name> <start_date> [end_date]]", args[0]);
        eprintln!("\nExample:");
        eprintln!("  cargo run database.sql");
        eprintln!("  cargo run database.sql --date-filter createdAt 2024-01-01");
        eprintln!("  cargo run database.sql --date-filter createdAt 2024-01-01 2024-12-31");
        eprintln!("  ./parsley-csv database.sql --date-filter date 2023-06-15");
        eprintln!("\nDate format: YYYY-MM-DD");
        eprintln!("Note: If end_date is not provided, it defaults to today's date");
        std::process::exit(1);
    }
    
    let sql_file = &args[1];
    
    // Check if file exists
    if !Path::new(sql_file).exists() {
        eprintln!("Error: File '{}' does not exist", sql_file);
        std::process::exit(1);
    }
    
    // Parse date filter if provided
    let date_filter = parse_date_filter(&args)?;
    
    if let Some(ref filter) = date_filter {
        println!("Date filter enabled:");
        println!("  Column: {}", filter.column_name);
        println!("  Start date: {}", filter.start_date);
        println!("  End date: {}", filter.end_date);
    }
    
    println!("Processing SQL file: {}", sql_file);
    
    // Parse SQL file
    let (tables, content) = parse_sql_file(sql_file)?;
    
    // Extract and write CSV for each table (in parallel)
    let csv_files: Vec<String> = tables.par_iter()
        .filter_map(|table| {
            match extract_insert_values(&content, &table.name) {
                Ok(rows) => {
                    // Apply date filter if specified
                    let filtered_rows = if let Some(ref filter) = date_filter {
                        match apply_date_filter(&table.columns, &rows, filter) {
                            Ok(filtered) => filtered,
                            Err(e) => {
                                eprintln!("Error applying date filter to table '{}': {}", table.name, e);
                                return None;
                            }
                        }
                    } else {
                        rows
                    };
                    
                    if !filtered_rows.is_empty() {
                        let csv_filename = format!("{}.csv", table.name.to_lowercase());
                        match write_csv(&csv_filename, &table.columns, &filtered_rows) {
                            Ok(_) => Some(csv_filename),
                            Err(e) => {
                                eprintln!("Error writing CSV for table '{}': {}", table.name, e);
                                None
                            }
                        }
                    } else {
                        println!("Warning: No rows remain for table '{}' after filtering - skipping", table.name);
                        None
                    }
                }
                Err(e) => {
                    eprintln!("Error extracting values for table '{}': {}", table.name, e);
                    None
                }
            }
        })
        .collect();
    
    println!("\nConversion complete!");
    if !csv_files.is_empty() {
        println!("\nGenerated CSV files:");
        for file in &csv_files {
            println!("  - {}", file);
        }
        println!("\nTo view the CSV files, you can use:");
        for file in csv_files.iter().take(2) {
            println!("  cat {} | head -5", file);
        }
        println!("\nOr open them in a spreadsheet application.");
    }
    
    Ok(())
}

/// Parse SQL file and extract table schemas and data
fn parse_sql_file<P: AsRef<Path>>(sql_file_path: P) -> Result<(Vec<Table>, String)> {
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
fn parse_table_columns(columns_text: &str) -> Vec<String> {
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
fn extract_insert_values(content: &str, table_name: &str) -> Result<Vec<Vec<String>>> {
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

/// Write data to CSV file
fn write_csv(filename: &str, headers: &[String], rows: &[Vec<String>]) -> Result<()> {
    let mut writer = Writer::from_path(filename)
        .context("Failed to create CSV file")?;
    
    // Write headers
    writer.write_record(headers)
        .context("Failed to write CSV headers")?;
    
    // Write data rows
    for row in rows {
        writer.write_record(row)
            .context("Failed to write CSV row")?;
    }
    
    writer.flush()
        .context("Failed to flush CSV writer")?;
    
    println!("Created {} with {} rows", filename, rows.len());
    
    Ok(())
}

/// Parse date filter arguments from command line
fn parse_date_filter(args: &[String]) -> Result<Option<DateFilter>> {
    // Look for --date-filter flag
    if let Some(pos) = args.iter().position(|arg| arg == "--date-filter") {
        // Need at least 2 more arguments: column_name, start_date
        // end_date is optional and defaults to today
        if args.len() < pos + 3 {
            anyhow::bail!(
                "Error: --date-filter requires at least 2 arguments: <column_name> <start_date> [end_date]\n\
                Example: --date-filter createdAt 2024-01-01\n\
                Example: --date-filter createdAt 2024-01-01 2024-12-31"
            );
        }
        
        let column_name = args[pos + 1].clone();
        let start_date_str = &args[pos + 2];
        
        // Parse start date
        let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
            .context(format!("Invalid start date '{}'. Use format: YYYY-MM-DD", start_date_str))?;
        
        // Parse end date or use today's date
        let end_date = if args.len() > pos + 3 {
            let end_date_str = &args[pos + 3];
            NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d")
                .context(format!("Invalid end date '{}'. Use format: YYYY-MM-DD", end_date_str))?
        } else {
            // Default to today's date
            chrono::Local::now().date_naive()
        };
        
        // Validate date range
        if start_date > end_date {
            anyhow::bail!("Error: Start date must be before or equal to end date");
        }
        
        Ok(Some(DateFilter {
            column_name,
            start_date,
            end_date,
        }))
    } else {
        Ok(None)
    }
}

/// Apply date filter to rows
fn apply_date_filter(
    headers: &[String],
    rows: &[Vec<String>],
    filter: &DateFilter,
) -> Result<Vec<Vec<String>>> {
    // Find the column index for the date column
    let column_index = headers.iter().position(|h| h == &filter.column_name)
        .ok_or_else(|| anyhow::anyhow!("Column '{}' not found in table headers", filter.column_name))?;
    
    // Filter rows based on date range
    let filtered: Vec<Vec<String>> = rows.iter()
        .filter(|row| {
            if column_index >= row.len() {
                return false;
            }
            
            let date_value = &row[column_index];
            
            // Try to parse the date value
            match parse_date_value(date_value) {
                Some(date) => {
                    date >= filter.start_date && date <= filter.end_date
                }
                None => {
                    eprintln!("Warning: Could not parse date value '{}', excluding row", date_value);
                    false
                }
            }
        })
        .cloned()
        .collect();
    
    Ok(filtered)
}

/// Parse a date value from various formats
fn parse_date_value(value: &str) -> Option<NaiveDate> {
    // Try to parse ISO 8601 with timezone first (most common in databases)
    if let Ok(datetime) = DateTime::<FixedOffset>::parse_from_rfc3339(value) {
        return Some(datetime.date_naive());
    }
    
    // Try various date formats without timezone
    let formats = vec![
        "%Y-%m-%d",           // 2024-01-15
        "%Y-%m-%d %H:%M:%S",  // 2024-01-15 14:30:00
        "%Y-%m-%dT%H:%M:%S",  // 2024-01-15T14:30:00 (ISO 8601)
        "%Y-%m-%d %H:%M:%S%.f", // 2024-01-15 14:30:00.123
        "%Y-%m-%dT%H:%M:%S%.f", // 2024-01-15T14:30:00.123
        "%m/%d/%Y",           // 01/15/2024
        "%d/%m/%Y",           // 15/01/2024
    ];
    
    // Try to parse as NaiveDateTime
    for format in &formats[..5] {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, format) {
            return Some(datetime.date());
        }
    }
    
    // Try to parse as just a date (no time component)
    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(value, format) {
            return Some(date);
        }
    }
    
    // Try to parse timestamps (milliseconds since epoch)
    if let Ok(timestamp) = value.parse::<i64>() {
        // Try both seconds and milliseconds
        if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp / 1000, 0) {
            return Some(datetime.date_naive());
        }
        if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp, 0) {
            return Some(datetime.date_naive());
        }
    }
    
    None
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