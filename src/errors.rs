use thiserror::Error;

#[derive(Error, Debug)]
pub enum EctusError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Invalid input: {0}")]
    Input(String),
    #[error("Backend error: {0}")]
    Backend(String),
}
