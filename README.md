# Table to CSV

A Rust command-line tool that converts SQL database dumps to CSV files. Table to CSV parses SQL files containing `CREATE TABLE` and `INSERT` statements and extracts the data into separate CSV files for each table.

## Features

- **SQL Parsing**: Automatically detects table schemas from `CREATE TABLE` statements
- **Data Extraction**: Extracts data from `INSERT` statements and converts to CSV format
- **Multiple Tables**: Handles databases with multiple tables, creating separate CSV files
- **Date Filtering**: Filter rows by date range using `--date-filter` option
- **Parallel Processing**: Fast multi-threaded CSV generation using Rayon
- **Robust Value Parsing**: Properly handles quoted strings, escaped characters, and SQL functions like `replace()`
- **Error Handling**: Comprehensive error handling with helpful error messages
- **Cross-Platform**: Built with Rust for excellent performance and cross-platform compatibility

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)

### Install from crates.io

The easiest way to install Table to CSV is via cargo:

```bash
cargo install table-to-csv
```

This will install the `table-to-csv` binary to your Cargo bin directory (typically `~/.cargo/bin/`).

### Building from Source

1. Clone or download this repository
2. Navigate to the project directory
3. Build the project:

```bash
cargo build --release
```

The executable will be created at `target/release/table-to-csv`.

## Usage

### Basic Usage

```bash
table-to-csv <sql_file>
```

### With Date Filtering

```bash
table-to-csv <sql_file> --date-filter <column_name> <start_date> [end_date]
```

Or if built from source:

```bash
./target/release/table-to-csv <sql_file> [--date-filter <column_name> <start_date> [end_date]]
```

Or using Cargo (when developing):

```bash
cargo run <sql_file> [-- --date-filter <column_name> <start_date> [end_date]]
```

### Examples

```bash
# Using the installed binary (after cargo install table-to-csv)
table-to-csv database.sql

# Convert a database dump to CSV files (when developing)
cargo run database.sql

# Using the compiled binary from source
./target/release/table-to-csv test.sql

# Filter by date range (column name: createdAt, from 2024-01-01 to 2024-12-31)
table-to-csv database.sql --date-filter createdAt 2024-01-01 2024-12-31

# Filter from a start date to today (end date defaults to today)
table-to-csv database.sql --date-filter date 2023-06-15

# Using Cargo with date filter
cargo run database.sql -- --date-filter createdAt 2024-01-01
```

**Date Format**: YYYY-MM-DD  
**Note**: If `end_date` is not provided, it defaults to today's date

## How It Works

1. **Schema Detection**: Parses `CREATE TABLE` statements to extract table names and column definitions
2. **Data Extraction**: Finds `INSERT` statements for each table and extracts the values
3. **Value Processing**: Handles SQL-specific formatting including:
   - Quoted strings (single and double quotes)
   - Escaped characters (`''` for single quotes, `""` for double quotes)
   - SQL functions like `replace()` for JSON data
4. **Date Filtering** (optional): Filters rows based on date column values within specified date range
5. **Parallel Processing**: Uses Rayon to process multiple tables concurrently for better performance
6. **CSV Generation**: Creates properly formatted CSV files with headers and data

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

Table to CSV will generate `users.csv`:

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
- Date/timestamp columns for filtering (supports various date formats)

## Dependencies

- `csv` - CSV file reading and writing
- `regex` - Regular expression pattern matching
- `anyhow` - Error handling
- `rayon` - Parallel processing for improved performance
- `chrono` - Date and time parsing for date filtering

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
- Invalid date formats
- Date filter column not found

## Building Release Binaries

### Local Build Script

To build release binaries for all macOS architectures, use the included build script:

```bash
./build-release.sh
```

This will:
- Build for x86_64 (Intel Macs)
- Build for aarch64 (Apple Silicon Macs)
- Create a universal binary (works on both)
- Generate compressed archives
- Create SHA256 checksums

All distribution files will be in the `dist/` directory.

### Automated Releases with GitHub Actions

The project includes a GitHub Actions workflow that automatically builds binaries for:
- macOS (Intel and Apple Silicon)
- Linux (x86_64 and ARM64)
- Windows (x86_64)

#### **Automatic Release on Version Change (Recommended)**

Simply update the version in `Cargo.toml` and push to main:

```bash
# Edit Cargo.toml and change the version
# Example: version = "0.4.1"

git add Cargo.toml
git commit -m "Bump version to 0.4.1"
git push origin main
```

GitHub Actions will automatically:
- Detect the version change
- Create a git tag (e.g., `v0.4.1`)
- Build binaries for all platforms
- Create a GitHub Release with all downloadable archives

#### **Manual Release with Tag**

Alternatively, you can manually create and push a tag:

```bash
git tag v0.4.1
git push origin v0.4.1
```

#### **Create Tag from GitHub Dashboard**

You can also create a release directly from the GitHub web interface:
1. Go to "Releases" → "Create a new release"
2. Click "Choose a tag" → Type a new tag (e.g., `v0.4.1`)
3. Click "Create new tag on publish"
4. The workflow will automatically build and attach binaries to the release

## License

This project is open source. Please check the license file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Example Output

When processing a SQL file, Table to CSV will output:

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

### Example Output with Date Filtering

When using the date filter feature:

```
Date filter enabled:
  Column: createdAt
  Start date: 2024-01-01
  End date: 2024-12-31
Processing SQL file: database.sql
Found table: CallLogs with 9 columns
Found table: CallSession with 9 columns
Found table: d1_migrations with 3 columns
Created calllogs.csv with 15 rows
Created callsession.csv with 42 rows
Warning: No rows remain for table 'd1_migrations' after filtering - skipping

Conversion complete!

Generated CSV files:
  - calllogs.csv
  - callsession.csv

To view the CSV files, you can use:
  cat calllogs.csv | head -5
  cat callsession.csv | head -5

Or open them in a spreadsheet application.
```
