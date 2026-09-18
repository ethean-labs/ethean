//! Utility functions

/// Utility result type
pub type Result<T> = std::result::Result<T, Error>;

/// Utility errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error")]
    Parse,
    
    #[error("IO error")]
    Io,
}
