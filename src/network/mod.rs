//! Network module placeholder

/// Network result type
pub type Result<T> = std::result::Result<T, Error>;

/// Network errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection failed")]
    Connection,
    
    #[error("Message invalid")]
    InvalidMessage,
}
