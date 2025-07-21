use sqlx::postgres::PgPoolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(#[from] PgPoolError),

    #[error("Query execution error: {0}")]
    Query(#[from] sqlx::Error),

    #[error("Invalid execution plan format: {0}")]
    InvalidPlanFormat(String),

    #[error("Plan parsing error: {0}")]
    PlanParsing(String),
}

pub type Result<T> = std::result::Result<T, DbError>;
