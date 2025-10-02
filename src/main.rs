use anyhow::Result;
use rayon::prelude::*;
use std::env;
use std::path::Path;

use table_to_csv::{
    parse_sql_file, extract_insert_values, write_csv,
    parse_date_filter, apply_date_filter,
};

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