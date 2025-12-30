//! Middleware components for the Meridian API
//!
//! Includes correlation ID propagation for distributed tracing
//! and rate limit headers for API responses.

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::HeaderValue;
use actix_web::{Error, HttpMessage};
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

/// Header name for correlation ID (standard)
pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";
/// Alternative header name (also commonly used)
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Correlation ID stored in request extensions
#[derive(Clone, Debug)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    /// Get the correlation ID as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Middleware for propagating correlation IDs across request boundaries.
///
/// This middleware:
/// 1. Checks for X-Correlation-ID or X-Request-ID in incoming headers
/// 2. Generates a new UUID v4 if neither header is present
/// 3. Stores the correlation ID in request extensions for handler access
/// 4. Adds the correlation ID to response headers
///
/// # Usage
/// ```ignore
/// App::new()
///     .wrap(CorrelationIdMiddleware)
///     // ... other middleware
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct CorrelationIdMiddleware;

impl CorrelationIdMiddleware {
    /// Create a new correlation ID middleware instance
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for CorrelationIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CorrelationIdMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CorrelationIdMiddlewareService { service }))
    }
}

/// The actual service that handles correlation ID logic
pub struct CorrelationIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CorrelationIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract or generate correlation ID
        let correlation_id = extract_or_generate_correlation_id(&req);

        // Store in request extensions for handler access
        req.extensions_mut()
            .insert(CorrelationId(correlation_id.clone()));

        // Log with correlation ID for tracing
        tracing::debug!(
            correlation_id = %correlation_id,
            method = %req.method(),
            path = %req.path(),
            "Request started"
        );

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // Add correlation ID to response headers
            if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
                res.headers_mut()
                    .insert(CORRELATION_ID_HEADER.parse().unwrap(), header_value);
            }

            tracing::debug!(
                correlation_id = %correlation_id,
                status = %res.status().as_u16(),
                "Request completed"
            );

            Ok(res)
        })
    }
}

/// Extract correlation ID from request headers or generate a new one
fn extract_or_generate_correlation_id(req: &ServiceRequest) -> String {
    // Try X-Correlation-ID first
    if let Some(value) = req.headers().get(CORRELATION_ID_HEADER) {
        if let Ok(id) = value.to_str() {
            if !id.is_empty() && id.len() <= 128 {
                return id.to_string();
            }
        }
    }

    // Try X-Request-ID as fallback
    if let Some(value) = req.headers().get(REQUEST_ID_HEADER) {
        if let Ok(id) = value.to_str() {
            if !id.is_empty() && id.len() <= 128 {
                return id.to_string();
            }
        }
    }

    // Generate new UUID v4
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler(req: actix_web::HttpRequest) -> HttpResponse {
        let correlation_id = req
            .extensions()
            .get::<CorrelationId>()
            .map(|c| c.as_str().to_string())
            .unwrap_or_default();
        HttpResponse::Ok().body(correlation_id)
    }

    #[actix_web::test]
    async fn test_generates_correlation_id_when_missing() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware::new())
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        assert!(resp.headers().contains_key(CORRELATION_ID_HEADER));

        let correlation_id = resp
            .headers()
            .get(CORRELATION_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        // Should be a valid UUID
        assert!(Uuid::parse_str(correlation_id).is_ok());
    }

    #[actix_web::test]
    async fn test_preserves_incoming_correlation_id() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware::new())
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let incoming_id = "test-correlation-id-12345";
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header((CORRELATION_ID_HEADER, incoming_id))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let outgoing_id = resp
            .headers()
            .get(CORRELATION_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        assert_eq!(outgoing_id, incoming_id);
    }

    #[actix_web::test]
    async fn test_accepts_x_request_id_header() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware::new())
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let incoming_id = "request-id-from-gateway";
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header((REQUEST_ID_HEADER, incoming_id))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let outgoing_id = resp
            .headers()
            .get(CORRELATION_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        assert_eq!(outgoing_id, incoming_id);
    }

    #[actix_web::test]
    async fn test_correlation_id_available_in_handler() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware::new())
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let incoming_id = "handler-accessible-id";
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header((CORRELATION_ID_HEADER, incoming_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(body_str, incoming_id);
    }

    #[actix_web::test]
    async fn test_rejects_oversized_correlation_id() {
        let app = test::init_service(
            App::new()
                .wrap(CorrelationIdMiddleware::new())
                .route("/", web::get().to(test_handler)),
        )
        .await;

        // 200 characters - too long
        let long_id = "a".repeat(200);
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header((CORRELATION_ID_HEADER, long_id.as_str()))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should generate a new UUID instead
        let outgoing_id = resp
            .headers()
            .get(CORRELATION_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        // Generated UUID, not the long string
        assert!(Uuid::parse_str(outgoing_id).is_ok());
    }
}

// ============================================================================
// HIGH-010: Rate Limit Headers Middleware
// ============================================================================

/// Rate limit header names per RFC 6585 and draft-ietf-httpapi-ratelimit-headers
pub const RATELIMIT_LIMIT_HEADER: &str = "X-RateLimit-Limit";
pub const RATELIMIT_REMAINING_HEADER: &str = "X-RateLimit-Remaining";
pub const RATELIMIT_RESET_HEADER: &str = "X-RateLimit-Reset";

/// Configuration for rate limit headers
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub limit: u32,
    /// Window size in seconds
    pub window_secs: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        // Match governor config: 2 req/sec * 60 = ~120/minute, burst 10
        Self {
            limit: 120,
            window_secs: 60,
        }
    }
}

/// Middleware that adds rate limit headers to all responses.
///
/// Headers added:
/// - X-RateLimit-Limit: Maximum requests allowed per window
/// - X-RateLimit-Remaining: Approximate remaining requests (based on window)
/// - X-RateLimit-Reset: Seconds until window resets
///
/// Note: This provides informative headers. The actual rate limiting is
/// handled by actix-governor middleware.
#[derive(Clone, Debug)]
pub struct RateLimitHeadersMiddleware {
    config: RateLimitConfig,
}

impl RateLimitHeadersMiddleware {
    /// Create a new rate limit headers middleware with default config
    pub fn new() -> Self {
        Self {
            config: RateLimitConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self { config }
    }
}

impl Default for RateLimitHeadersMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitHeadersService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitHeadersService {
            service,
            config: self.config.clone(),
        }))
    }
}

/// The actual service that adds rate limit headers
pub struct RateLimitHeadersService<S> {
    service: S,
    config: RateLimitConfig,
}

impl<S, B> Service<ServiceRequest> for RateLimitHeadersService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        let limit = self.config.limit;
        let window_secs = self.config.window_secs;

        Box::pin(async move {
            let mut res = fut.await?;

            // Calculate approximate remaining (simplified - actual tracking in governor)
            // For 429 responses, remaining is 0
            let remaining = if res.status() == actix_web::http::StatusCode::TOO_MANY_REQUESTS {
                0
            } else {
                // Approximate remaining based on configured limit
                // Real value would require per-client tracking shared with governor
                limit.saturating_sub(1)
            };

            // Calculate reset time (seconds until next window)
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let window_start = now - (now % window_secs as u64);
            let reset = window_start + window_secs as u64 - now;

            // Add rate limit headers
            if let Ok(limit_val) = HeaderValue::from_str(&limit.to_string()) {
                res.headers_mut()
                    .insert(RATELIMIT_LIMIT_HEADER.parse().unwrap(), limit_val);
            }
            if let Ok(remaining_val) = HeaderValue::from_str(&remaining.to_string()) {
                res.headers_mut()
                    .insert(RATELIMIT_REMAINING_HEADER.parse().unwrap(), remaining_val);
            }
            if let Ok(reset_val) = HeaderValue::from_str(&reset.to_string()) {
                res.headers_mut()
                    .insert(RATELIMIT_RESET_HEADER.parse().unwrap(), reset_val);
            }

            // Add Retry-After header on 429 responses
            if res.status() == actix_web::http::StatusCode::TOO_MANY_REQUESTS {
                if let Ok(retry_val) = HeaderValue::from_str(&reset.to_string()) {
                    res.headers_mut()
                        .insert("Retry-After".parse().unwrap(), retry_val);
                }
            }

            Ok(res)
        })
    }
}

#[cfg(test)]
mod rate_limit_tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("OK")
    }

    async fn rate_limited_handler() -> HttpResponse {
        HttpResponse::TooManyRequests().body("Too Many Requests")
    }

    #[actix_web::test]
    async fn test_adds_rate_limit_headers() {
        let app = test::init_service(
            App::new()
                .wrap(RateLimitHeadersMiddleware::new())
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        assert!(resp.headers().contains_key(RATELIMIT_LIMIT_HEADER));
        assert!(resp.headers().contains_key(RATELIMIT_REMAINING_HEADER));
        assert!(resp.headers().contains_key(RATELIMIT_RESET_HEADER));

        // Check limit value matches config
        let limit = resp
            .headers()
            .get(RATELIMIT_LIMIT_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(limit, "120");
    }

    #[actix_web::test]
    async fn test_adds_retry_after_on_429() {
        let app = test::init_service(
            App::new()
                .wrap(RateLimitHeadersMiddleware::new())
                .route("/", web::get().to(rate_limited_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::TOO_MANY_REQUESTS
        );
        assert!(resp.headers().contains_key("retry-after"));

        // Remaining should be 0 on 429
        let remaining = resp
            .headers()
            .get(RATELIMIT_REMAINING_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(remaining, "0");
    }

    #[actix_web::test]
    async fn test_custom_config() {
        let config = RateLimitConfig {
            limit: 1000,
            window_secs: 3600,
        };
        let app = test::init_service(
            App::new()
                .wrap(RateLimitHeadersMiddleware::with_config(config))
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        let limit = resp
            .headers()
            .get(RATELIMIT_LIMIT_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(limit, "1000");
    }
}
