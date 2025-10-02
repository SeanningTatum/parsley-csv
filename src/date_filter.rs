use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime, DateTime, FixedOffset};

use crate::types::DateFilter;

/// Parse date filter arguments from command line
pub fn parse_date_filter(args: &[String]) -> Result<Option<DateFilter>> {
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
pub fn apply_date_filter(
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
    fn test_parse_date_value() {
        // ISO 8601 date
        assert!(parse_date_value("2024-01-15").is_some());
        
        // ISO 8601 datetime
        assert!(parse_date_value("2024-01-15T14:30:00").is_some());
        
        // US format
        assert!(parse_date_value("01/15/2024").is_some());
        
        // Invalid date
        assert!(parse_date_value("not-a-date").is_none());
    }
}

