//! API error handling
//!
//! Provides error types and handling for the REST API.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use std::fmt;

/// API result type
pub type Result<T> = std::result::Result<T, Error>;

/// API error types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Error {
    pub code: u16,
    pub message: String,
    pub details: Option<String>,
}

impl Error {
    /// Create a new error
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Bad request error (400)
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }

    /// Unauthorized error (401)
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(401, message)
    }

    /// Forbidden error (403)
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(403, message)
    }

    /// Not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, message)
    }

    /// Method not allowed error (405)
    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(405, message)
    }

    /// Conflict error (409)
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(409, message)
    }

    /// Unprocessable entity error (422)
    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self::new(422, message)
    }

    /// Too many requests error (429)
    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::new(429, message)
    }

    /// Internal server error (500)
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }

    /// Not implemented error (501)
    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::new(501, message)
    }

    /// Bad gateway error (502)
    pub fn bad_gateway(message: impl Into<String>) -> Self {
        Self::new(502, message)
    }

    /// Service unavailable error (503)
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(503, message)
    }

    /// Gateway timeout error (504)
    pub fn gateway_timeout(message: impl Into<String>) -> Self {
        Self::new(504, message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error {}: {}", self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        let error_response = ErrorResponse {
            code: self.code,
            message: self.message,
            stacktraces: self.details.map(|d| vec![d]),
        };

        (status_code, Json(error_response)).into_response()
    }
}

/// Error response format
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub stacktraces: Option<Vec<String>>,
}

/// Convert various error types to API errors
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::bad_request("Invalid JSON").with_details(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::internal_server_error("IO error").with_details(err.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::internal_server_error("Task join error").with_details(err.to_string())
    }
}

/// Validation error details
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub rejected_value: Option<serde_json::Value>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
            rejected_value: None,
        }
    }

    pub fn with_value(mut self, value: serde_json::Value) -> Self {
        self.rejected_value = Some(value);
        self
    }
}

/// Validation result helper
pub fn validate_hex_string(value: &str, expected_length: Option<usize>) -> Result<()> {
    if !value.starts_with("0x") {
        return Err(Error::bad_request("Hex string must start with '0x'"));
    }

    let hex_part = &value[2..];
    if hex_part.chars().any(|c| !c.is_ascii_hexdigit()) {
        return Err(Error::bad_request("Invalid hex characters"));
    }

    if let Some(len) = expected_length {
        if hex_part.len() != len {
            return Err(Error::bad_request(format!(
                "Expected hex string length {}, got {}",
                len,
                hex_part.len()
            )));
        }
    }

    Ok(())
}

/// Validate slot number
pub fn validate_slot(slot_str: &str) -> Result<u64> {
    slot_str.parse::<u64>()
        .map_err(|_| Error::bad_request("Invalid slot number"))
}

/// Validate epoch number
pub fn validate_epoch(epoch_str: &str) -> Result<u64> {
    epoch_str.parse::<u64>()
        .map_err(|_| Error::bad_request("Invalid epoch number"))
}

/// Validate validator index
pub fn validate_validator_index(index_str: &str) -> Result<u64> {
    index_str.parse::<u64>()
        .map_err(|_| Error::bad_request("Invalid validator index"))
}

/// Validate committee index
pub fn validate_committee_index(index_str: &str) -> Result<u64> {
    index_str.parse::<u64>()
        .map_err(|_| Error::bad_request("Invalid committee index"))
}

/// Validate state identifier
pub fn validate_state_id(state_id: &str) -> Result<StateId> {
    match state_id {
        "head" => Ok(StateId::Head),
        "genesis" => Ok(StateId::Genesis),
        "finalized" => Ok(StateId::Finalized),
        "justified" => Ok(StateId::Justified),
        _ => {
            if state_id.starts_with("0x") {
                validate_hex_string(state_id, Some(64))?;
                Ok(StateId::Root(state_id.to_string()))
            } else {
                let slot = validate_slot(state_id)?;
                Ok(StateId::Slot(slot))
            }
        }
    }
}

/// Validate block identifier
pub fn validate_block_id(block_id: &str) -> Result<BlockId> {
    match block_id {
        "head" => Ok(BlockId::Head),
        "genesis" => Ok(BlockId::Genesis),
        "finalized" => Ok(BlockId::Finalized),
        _ => {
            if block_id.starts_with("0x") {
                validate_hex_string(block_id, Some(64))?;
                Ok(BlockId::Root(block_id.to_string()))
            } else {
                let slot = validate_slot(block_id)?;
                Ok(BlockId::Slot(slot))
            }
        }
    }
}

/// State identifier types
#[derive(Debug, Clone, PartialEq)]
pub enum StateId {
    Head,
    Genesis,
    Finalized,
    Justified,
    Slot(u64),
    Root(String),
}

/// Block identifier types
#[derive(Debug, Clone, PartialEq)]
pub enum BlockId {
    Head,
    Genesis,
    Finalized,
    Slot(u64),
    Root(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = Error::bad_request("Test error");
        assert_eq!(error.code, 400);
        assert_eq!(error.message, "Test error");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_error_with_details() {
        let error = Error::internal_server_error("Server error")
            .with_details("Detailed explanation");
        assert_eq!(error.code, 500);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_validate_hex_string() {
        assert!(validate_hex_string("0x1234", Some(4)).is_ok());
        assert!(validate_hex_string("0x", Some(0)).is_ok());
        assert!(validate_hex_string("1234", None).is_err()); // Missing 0x
        assert!(validate_hex_string("0xgg", None).is_err()); // Invalid hex
        assert!(validate_hex_string("0x12", Some(4)).is_err()); // Wrong length
    }

    #[test]
    fn test_validate_slot() {
        assert_eq!(validate_slot("123").unwrap(), 123);
        assert!(validate_slot("abc").is_err());
        assert!(validate_slot("-1").is_err());
    }

    #[test]
    fn test_validate_state_id() {
        assert_eq!(validate_state_id("head").unwrap(), StateId::Head);
        assert_eq!(validate_state_id("genesis").unwrap(), StateId::Genesis);
        assert_eq!(validate_state_id("123").unwrap(), StateId::Slot(123));
        assert!(validate_state_id("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").is_ok());
    }

    #[test]
    fn test_validate_block_id() {
        assert_eq!(validate_block_id("head").unwrap(), BlockId::Head);
        assert_eq!(validate_block_id("finalized").unwrap(), BlockId::Finalized);
        assert_eq!(validate_block_id("456").unwrap(), BlockId::Slot(456));
    }

    #[test]
    fn test_error_display() {
        let error = Error::bad_request("Test").with_details("More info");
        let display = format!("{}", error);
        assert!(display.contains("API Error 400"));
        assert!(display.contains("Test"));
        assert!(display.contains("More info"));
    }
}
