use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    DB(Box<dyn std::error::Error + Send + Sync>),

    #[error("Progress not found")]
    NotFound(String),
}

impl ServiceError {
    pub fn db(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        ServiceError::DB(Box::new(e))
    }
}
