# Parsley CSV

A Rust command-line tool that converts SQL database dumps to CSV files. Parsley CSV parses SQL files containing `CREATE TABLE` and `INSERT` statements and extracts the data into separate CSV files for each table.

## Features

- **SQL Parsing**: Automatically detects table schemas from `CREATE TABLE` statements
- **Data Extraction**: Extracts data from `INSERT` statements and converts to CSV format
- **Multiple Tables**: Handles databases with multiple tables, creating separate CSV files
- **Robust Value Parsing**: Properly handles quoted strings, escaped characters, and SQL functions like `replace()`
- **Error Handling**: Comprehensive error handling with helpful error messages
- **Cross-Platform**: Built with Rust for excellent performance and cross-platform compatibility

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)

### Install from crates.io

The easiest way to install Parsley CSV is via cargo:

```bash
cargo install parsley-csv
```

This will install the `parsley-csv` binary to your Cargo bin directory (typically `~/.cargo/bin/`).

### Building from Source

1. Clone or download this repository
2. Navigate to the project directory
3. Build the project:

```bash
cargo build --release
```

The executable will be created at `target/release/parsley-csv`.

## Usage

```bash
parsley-csv <sql_file>
```

Or if built from source:

```bash
./target/release/parsley-csv <sql_file>
```

Or using Cargo (when developing):

```bash
cargo run <sql_file>
```

### Examples

```bash
# Using the installed binary (after cargo install parsley-csv)
parsley-csv database.sql

# Convert a database dump to CSV files (when developing)
cargo run database.sql

# Using the compiled binary from source
./target/release/parsley-csv test.sql
```

## How It Works

1. **Schema Detection**: Parses `CREATE TABLE` statements to extract table names and column definitions
2. **Data Extraction**: Finds `INSERT` statements for each table and extracts the values
3. **Value Processing**: Handles SQL-specific formatting including:
   - Quoted strings (single and double quotes)
   - Escaped characters (`''` for single quotes, `""` for double quotes)
   - SQL functions like `replace()` for JSON data
4. **CSV Generation**: Creates properly formatted CSV files with headers and data

## Example

Given a SQL file like `test.sql`:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE
);

INSERT INTO users VALUES(1, 'Alice Smith', 'alice@example.com');
INSERT INTO users VALUES(2, 'Bob Johnson', 'bob@example.com');
```

Parsley CSV will generate `users.csv`:

```csv
id,name,email
1,Alice Smith,alice@example.com
2,Bob Johnson,bob@example.com
```

## Supported SQL Features

- `CREATE TABLE` statements with various column types
- `INSERT INTO ... VALUES` statements
- Single and double-quoted string values
- Escaped quotes in string values
- SQL `replace()` function calls
- Multi-line table definitions
- Foreign key constraints (ignored during parsing)

## Dependencies

- `csv` - CSV file reading and writing
- `regex` - Regular expression pattern matching
- `anyhow` - Error handling

## Testing

Run the test suite:

```bash
cargo test
```

The project includes unit tests for:
- Table column parsing
- Value cleaning and unescaping
- CSV value parsing

## Error Handling

The tool provides clear error messages for common issues:
- Missing command-line arguments
- File not found
- SQL parsing errors
- CSV writing errors

## License

This project is open source. Please check the license file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Example Output

When processing a SQL file, Parsley CSV will output:

```
Processing SQL file: database.sql
Found table: CallLogs with 9 columns
Found table: CallSession with 9 columns
Found table: d1_migrations with 3 columns
Created calllogs.csv with 35 rows
Created callsession.csv with 89 rows
Created d1_migrations.csv with 3 rows

Conversion complete!

Generated CSV files:
  - calllogs.csv
  - callsession.csv
  - d1_migrations.csv

To view the CSV files, you can use:
  cat calllogs.csv | head -5
  cat callsession.csv | head -5

Or open them in a spreadsheet application.
```
