use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Runtime(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn runtime(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::Runtime(Box::new(e))
    }
}
