use jiff::Error as JiffError;
use sqlx::Error as SqlxError;
use uuid::Error as UuidError;
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("sqlx error: {0}")]
    Sql(#[from] SqlxError),
    #[error("jiff error: {0}")]
    Jiff(#[from] JiffError),
    #[error("uuid error: {0}")]
    Uuid(#[from] UuidError),
}
