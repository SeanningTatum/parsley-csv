use anyhow::{Context, Result};
use csv::Writer;

/// Write data to CSV file
pub fn write_csv(filename: &str, headers: &[String], rows: &[Vec<String>]) -> Result<()> {
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

