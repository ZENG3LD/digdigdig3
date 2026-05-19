use thiserror::Error;

#[derive(Debug, Error)]
pub enum StationError {
    #[error("station not yet built")]
    NotBuilt,
    #[error("subscription failed: {0}")]
    Subscribe(String),
    #[error("core error: {0}")]
    Core(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StationError>;
