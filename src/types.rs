use chrono::NaiveDate;

/// Represents a database table with its name and column names
#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
}

/// Represents a date filter configuration
#[derive(Debug, Clone)]
pub struct DateFilter {
    pub column_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

