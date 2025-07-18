//! Configuration module placeholder

/// Config result type
pub type Result<T> = std::result::Result<T, Error>;

/// Config errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid config")]
    Invalid,
    
    #[error("File not found")]
    FileNotFound,
}
