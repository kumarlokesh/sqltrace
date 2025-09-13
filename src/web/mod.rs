//! Web-related utilities and validation functions

use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

/// Validate a SQL query for basic syntax correctness
pub fn validate_query(query: &str) -> Result<(), String> {
    if query.trim().is_empty() {
        return Err("Query cannot be empty".to_string());
    }

    // Parse the SQL query
    let dialect = PostgreSqlDialect {};
    match Parser::parse_sql(&dialect, query) {
        Ok(statements) => {
            if statements.is_empty() {
                return Err("No valid SQL statements found".to_string());
            }

            // Check if all statements are SELECT statements
            for statement in &statements {
                match statement {
                    Statement::Query(_) => {
                        // This is a SELECT/WITH/etc query - allowed
                    }
                    _ => {
                        return Err("Only SELECT queries are supported for analysis".to_string());
                    }
                }
            }

            Ok(())
        }
        Err(e) => Err(format!("SQL parse error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_query_empty() {
        assert!(validate_query("").is_err());
        assert!(validate_query("   ").is_err());
    }

    #[test]
    fn test_validate_query_select() {
        assert!(validate_query("SELECT 1").is_ok());
        assert!(validate_query("SELECT * FROM users").is_ok());
        assert!(validate_query("SELECT u.name FROM users u WHERE u.id = 1").is_ok());
    }

    #[test]
    fn test_validate_query_non_select() {
        assert!(validate_query("INSERT INTO users (name) VALUES ('test')").is_err());
        assert!(validate_query("UPDATE users SET name = 'test'").is_err());
        assert!(validate_query("DELETE FROM users").is_err());
        assert!(validate_query("CREATE TABLE test (id INT)").is_err());
    }

    #[test]
    fn test_validate_query_invalid_syntax() {
        assert!(validate_query("SELECT FROM").is_err());
        assert!(validate_query("INVALID SQL").is_err());
    }
}
