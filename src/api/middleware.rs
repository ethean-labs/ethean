//! API middleware implementation
//!
//! Provides middleware for rate limiting, authentication,
//! request validation, and other cross-cutting concerns.

use axum::{
    middleware::Next,
    http::{Request, Response, StatusCode, HeaderMap, header},
    response::Json,
    body::Body,
};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower::{Layer, Service};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
};

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
            window_size: Duration::from_secs(60),
        }
    }
}

/// Rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    clients: Arc<Mutex<HashMap<String, ClientState>>>,
}

#[derive(Debug, Clone)]
struct ClientState {
    requests: Vec<Instant>,
    last_request: Instant,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check_rate_limit(&self, client_id: &str) -> bool {
        let mut clients = self.clients.lock().unwrap();
        let now = Instant::now();
        
        let client_state = clients.entry(client_id.to_string()).or_insert_with(|| {
            ClientState {
                requests: Vec::new(),
                last_request: now,
            }
        });

        // Remove old requests outside the window
        client_state.requests.retain(|&req_time| {
            now.duration_since(req_time) < self.config.window_size
        });

        // Check if rate limit is exceeded
        if client_state.requests.len() >= self.config.requests_per_minute as usize {
            return false;
        }

        // Add current request
        client_state.requests.push(now);
        client_state.last_request = now;
        true
    }
}

/// Authentication middleware
pub async fn auth_middleware<B>(
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response<Body>, StatusCode> {
    // Check for API key in headers
    if let Some(api_key) = headers.get("x-api-key") {
        if let Ok(key_str) = api_key.to_str() {
            if is_valid_api_key(key_str) {
                return Ok(next.run(request).await);
            }
        }
    }

    // Check for Bearer token
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if is_valid_bearer_token(token) {
                    return Ok(next.run(request).await);
                }
            }
        }
    }

    // For development, allow requests without authentication
    // In production, this should return an error
    Ok(next.run(request).await)
}

/// Rate limiting middleware
pub async fn rate_limit_middleware<B>(
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response<Body>, StatusCode> {
    let client_id = get_client_id(&headers, &request);
    let rate_limiter = RateLimiter::new(RateLimitConfig::default());
    
    if !rate_limiter.check_rate_limit(&client_id) {
        let error_response = json!({
            "code": 429,
            "message": "Rate limit exceeded",
            "data": {
                "retry_after": 60
            }
        });
        
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// Request validation middleware
pub async fn validation_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response<Body>, StatusCode> {
    // Validate request headers and parameters
    let headers = request.headers();
    
    // Check Content-Type for POST/PUT requests
    let method = request.method();
    if method == "POST" || method == "PUT" {
        if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
            if let Ok(ct_str) = content_type.to_str() {
                if !ct_str.contains("application/json") && !ct_str.contains("application/octet-stream") {
                    return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
                }
            }
        }
    }

    // Validate request size
    if let Some(content_length) = headers.get(header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                if length > 10_000_000 { // 10MB limit
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    Ok(next.run(request).await)
}

/// CORS middleware configuration
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600))
}

/// Compression middleware
pub fn compression_layer() -> CompressionLayer {
    CompressionLayer::new()
}

/// Tracing middleware
pub fn tracing_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

/// Security headers middleware
pub async fn security_headers_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response<Body>, StatusCode> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    
    Ok(response)
}

/// Get client identifier for rate limiting
fn get_client_id<B>(headers: &HeaderMap, request: &Request<B>) -> String {
    // Try to get client IP from headers
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(ip_str) = forwarded_for.to_str() {
            return ip_str.split(',').next().unwrap_or("unknown").trim().to_string();
        }
    }
    
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Try to get from connection info (would need request extensions in real implementation)
    "127.0.0.1".to_string()
}

/// Validate API key (placeholder implementation)
fn is_valid_api_key(api_key: &str) -> bool {
    // In production, this should validate against a secure store
    !api_key.is_empty() && api_key.len() >= 32
}

/// Validate Bearer token (placeholder implementation)
fn is_valid_bearer_token(token: &str) -> bool {
    // In production, this should validate JWT or other token format
    !token.is_empty() && token.len() >= 32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        assert_eq!(limiter.config.requests_per_minute, 60);
    }

    #[test]
    fn test_rate_limit_check() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            burst_size: 2,
            window_size: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        
        // First request should pass
        assert!(limiter.check_rate_limit("test_client"));
        
        // Second request should pass
        assert!(limiter.check_rate_limit("test_client"));
        
        // Third request should fail
        assert!(!limiter.check_rate_limit("test_client"));
    }

    #[test]
    fn test_api_key_validation() {
        assert!(!is_valid_api_key(""));
        assert!(!is_valid_api_key("short"));
        assert!(is_valid_api_key("a".repeat(32).as_str()));
    }

    #[test]
    fn test_bearer_token_validation() {
        assert!(!is_valid_bearer_token(""));
        assert!(!is_valid_bearer_token("short"));
        assert!(is_valid_bearer_token("a".repeat(32).as_str()));
    }
}
