//! API module placeholder

/// API result type
pub type Result<T> = std::result::Result<T, Error>;

/// API errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error")]
    Http,
    
    #[error("Invalid request")]
    InvalidRequest,
}
