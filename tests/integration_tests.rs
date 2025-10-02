use parsley_csv::{parse_sql_file, extract_insert_values, write_csv, parse_table_columns};
use std::fs;
use std::path::Path;

#[test]
fn test_parse_test_sql() {
    let test_sql_path = "test.sql";
    
    // Parse the SQL file
    let result = parse_sql_file(test_sql_path);
    assert!(result.is_ok(), "Failed to parse test.sql: {:?}", result.err());
    
    let (tables, _content) = result.unwrap();
    
    // Should have 2 tables: users and products
    assert_eq!(tables.len(), 2, "Expected 2 tables but found {}", tables.len());
    
    // Check users table
    let users_table = tables.iter().find(|t| t.name == "users");
    assert!(users_table.is_some(), "users table not found");
    let users_table = users_table.unwrap();
    assert_eq!(users_table.columns.len(), 3, "users table should have 3 columns");
    assert_eq!(users_table.columns[0], "id");
    assert_eq!(users_table.columns[1], "name");
    assert_eq!(users_table.columns[2], "email");
    
    // Check products table
    let products_table = tables.iter().find(|t| t.name == "products");
    assert!(products_table.is_some(), "products table not found");
    let products_table = products_table.unwrap();
    assert_eq!(products_table.columns.len(), 4, "products table should have 4 columns");
    assert_eq!(products_table.columns[0], "id");
    assert_eq!(products_table.columns[1], "name");
    assert_eq!(products_table.columns[2], "price");
    assert_eq!(products_table.columns[3], "category");
}

#[test]
fn test_extract_insert_values_from_test_sql() {
    let test_sql_path = "test.sql";
    let content = fs::read_to_string(test_sql_path).expect("Failed to read test.sql");
    
    // Extract users data
    let users_rows = extract_insert_values(&content, "users").expect("Failed to extract users data");
    assert_eq!(users_rows.len(), 3, "Expected 3 user rows");
    
    // Check first user
    assert_eq!(users_rows[0].len(), 3);
    assert_eq!(users_rows[0][0], "1");
    assert_eq!(users_rows[0][1], "Alice Smith");
    assert_eq!(users_rows[0][2], "alice@example.com");
    
    // Check second user
    assert_eq!(users_rows[1][0], "2");
    assert_eq!(users_rows[1][1], "Bob Johnson");
    assert_eq!(users_rows[1][2], "bob@example.com");
    
    // Extract products data
    let products_rows = extract_insert_values(&content, "products").expect("Failed to extract products data");
    assert_eq!(products_rows.len(), 4, "Expected 4 product rows");
    
    // Check first product
    assert_eq!(products_rows[0].len(), 4);
    assert_eq!(products_rows[0][0], "1");
    assert_eq!(products_rows[0][1], "Laptop");
    assert_eq!(products_rows[0][2], "999.99");
    assert_eq!(products_rows[0][3], "Electronics");
    
    // Check last product
    assert_eq!(products_rows[3][0], "4");
    assert_eq!(products_rows[3][1], "Headphones");
    assert_eq!(products_rows[3][2], "79.99");
    assert_eq!(products_rows[3][3], "Electronics");
}

#[test]
fn test_write_csv_from_test_sql() {
    let test_sql_path = "test.sql";
    let content = fs::read_to_string(test_sql_path).expect("Failed to read test.sql");
    
    // Extract and write users table
    let users_rows = extract_insert_values(&content, "users").expect("Failed to extract users data");
    let users_headers = vec!["id".to_string(), "name".to_string(), "email".to_string()];
    let users_csv = "test_users_output.csv";
    
    let result = write_csv(users_csv, &users_headers, &users_rows);
    assert!(result.is_ok(), "Failed to write users CSV: {:?}", result.err());
    
    // Verify the CSV file was created
    assert!(Path::new(users_csv).exists(), "CSV file was not created");
    
    // Read and verify the CSV content
    let csv_content = fs::read_to_string(users_csv).expect("Failed to read CSV");
    let lines: Vec<&str> = csv_content.lines().collect();
    
    // Should have header + 3 data rows
    assert_eq!(lines.len(), 4, "Expected 4 lines in CSV");
    assert_eq!(lines[0], "id,name,email");
    assert!(lines[1].contains("Alice Smith"));
    assert!(lines[2].contains("Bob Johnson"));
    assert!(lines[3].contains("Charlie Brown"));
    
    // Clean up
    fs::remove_file(users_csv).ok();
}

#[test]
fn test_parse_table_columns_with_constraints() {
    let columns_text = r#"
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT UNIQUE
    "#;
    
    let columns = parse_table_columns(columns_text);
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0], "id");
    assert_eq!(columns[1], "name");
    assert_eq!(columns[2], "email");
}

#[test]
fn test_end_to_end_conversion() {
    let test_sql_path = "test.sql";
    
    // Parse SQL file
    let (tables, content) = parse_sql_file(test_sql_path).expect("Failed to parse SQL");
    
    // Process each table
    for table in &tables {
        let rows = extract_insert_values(&content, &table.name)
            .expect(&format!("Failed to extract data for {}", table.name));
        
        if !rows.is_empty() {
            let csv_filename = format!("test_{}.csv", table.name.to_lowercase());
            write_csv(&csv_filename, &table.columns, &rows)
                .expect(&format!("Failed to write CSV for {}", table.name));
            
            // Verify file exists and has content
            assert!(Path::new(&csv_filename).exists());
            let metadata = fs::metadata(&csv_filename).expect("Failed to get file metadata");
            assert!(metadata.len() > 0, "CSV file is empty");
            
            // Clean up
            fs::remove_file(&csv_filename).ok();
        }
    }
}

#[test]
fn test_numeric_and_decimal_values() {
    let test_sql_path = "test.sql";
    let content = fs::read_to_string(test_sql_path).expect("Failed to read test.sql");
    
    let products_rows = extract_insert_values(&content, "products").expect("Failed to extract products data");
    
    // Verify decimal values are correctly parsed
    assert_eq!(products_rows[0][2], "999.99");
    assert_eq!(products_rows[1][2], "12.50");
    assert_eq!(products_rows[2][2], "5.99");
}

