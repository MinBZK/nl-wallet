use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("connection error: {0}")]
    Connection(#[source] Box<dyn Error + Send + Sync>),
    #[error("transaction error: {0}")]
    Transaction(#[source] Box<dyn Error + Send + Sync>),
    #[error("execution error: {0}")]
    Execution(#[source] Box<dyn Error + Send + Sync>),
    #[error("not found: {0}")]
    NotFound(String),
}
