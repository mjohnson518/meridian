//! Middleware components for the Meridian API
//!
//! Includes correlation ID propagation for distributed tracing
//! and rate limit headers for API responses.

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage};
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::{Context, Poll};
use uuid::Uuid;

/// Header name for correlation ID (standard)
pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";
/// Alternative header name (also commonly used)
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Pre-parsed header names to avoid runtime parsing failures
/// SECURITY: MED-006 FIX - No unwrap() in production code
fn correlation_id_header_name() -> &'static HeaderName {
    static HEADER: OnceLock<HeaderName> = OnceLock::new();
    HEADER.get_or_init(|| {
        HeaderName::from_static("x-correlation-id")
    })
}

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
            // SECURITY: MED-006 FIX - Use pre-parsed header name instead of unwrap()
            if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
                res.headers_mut()
                    .insert(correlation_id_header_name().clone(), header_value);
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

/// Pre-parsed rate limit header names to avoid runtime parsing failures
/// SECURITY: MED-006 FIX - No unwrap() in production code
fn ratelimit_limit_header_name() -> &'static HeaderName {
    static HEADER: OnceLock<HeaderName> = OnceLock::new();
    HEADER.get_or_init(|| HeaderName::from_static("x-ratelimit-limit"))
}

fn ratelimit_remaining_header_name() -> &'static HeaderName {
    static HEADER: OnceLock<HeaderName> = OnceLock::new();
    HEADER.get_or_init(|| HeaderName::from_static("x-ratelimit-remaining"))
}

fn ratelimit_reset_header_name() -> &'static HeaderName {
    static HEADER: OnceLock<HeaderName> = OnceLock::new();
    HEADER.get_or_init(|| HeaderName::from_static("x-ratelimit-reset"))
}

fn retry_after_header_name() -> &'static HeaderName {
    static HEADER: OnceLock<HeaderName> = OnceLock::new();
    HEADER.get_or_init(|| HeaderName::from_static("retry-after"))
}

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
            // SECURITY: MED-006 FIX - Use pre-parsed header names instead of unwrap()
            if let Ok(limit_val) = HeaderValue::from_str(&limit.to_string()) {
                res.headers_mut()
                    .insert(ratelimit_limit_header_name().clone(), limit_val);
            }
            if let Ok(remaining_val) = HeaderValue::from_str(&remaining.to_string()) {
                res.headers_mut()
                    .insert(ratelimit_remaining_header_name().clone(), remaining_val);
            }
            if let Ok(reset_val) = HeaderValue::from_str(&reset.to_string()) {
                res.headers_mut()
                    .insert(ratelimit_reset_header_name().clone(), reset_val);
            }

            // Add Retry-After header on 429 responses
            if res.status() == actix_web::http::StatusCode::TOO_MANY_REQUESTS {
                if let Ok(retry_val) = HeaderValue::from_str(&reset.to_string()) {
                    res.headers_mut()
                        .insert(retry_after_header_name().clone(), retry_val);
                }
            }

            Ok(res)
        })
    }
}

// ============================================================================
// HIGH-001: CSRF Protection Middleware (Origin/Referer Validation)
// ============================================================================

/// Header names for CSRF protection
pub const ORIGIN_HEADER: &str = "Origin";
pub const REFERER_HEADER: &str = "Referer";

/// Pre-parsed header names for CSRF protection
fn csrf_origin_header_name() -> &'static HeaderName {
    static HEADER: OnceLock<HeaderName> = OnceLock::new();
    HEADER.get_or_init(|| HeaderName::from_static("origin"))
}

fn csrf_referer_header_name() -> &'static HeaderName {
    static HEADER: OnceLock<HeaderName> = OnceLock::new();
    HEADER.get_or_init(|| HeaderName::from_static("referer"))
}

/// Configuration for CSRF protection
#[derive(Clone, Debug)]
pub struct CsrfConfig {
    /// Allowed origins (e.g., ["https://meridian.example.com"])
    pub allowed_origins: Vec<String>,
    /// Whether to enforce in development (default: false)
    pub enforce_in_dev: bool,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:8080".to_string(),
            ],
            enforce_in_dev: false,
        }
    }
}

impl CsrfConfig {
    /// Create config for production with specific allowed origins
    pub fn production(origins: Vec<String>) -> Self {
        Self {
            allowed_origins: origins,
            enforce_in_dev: true,
        }
    }
}

/// Middleware that validates Origin/Referer headers for state-changing requests.
///
/// This provides CSRF protection in addition to SameSite=Strict cookies:
/// - Validates Origin header on POST/PUT/PATCH/DELETE requests
/// - Falls back to Referer header if Origin is missing
/// - Allows requests without either header for API clients
///
/// Defense-in-depth layer - primary protection is SameSite=Strict cookies.
#[derive(Clone, Debug)]
pub struct CsrfProtectionMiddleware {
    config: CsrfConfig,
}

impl CsrfProtectionMiddleware {
    /// Create with default configuration (localhost origins allowed)
    pub fn new() -> Self {
        Self {
            config: CsrfConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CsrfConfig) -> Self {
        Self { config }
    }
}

impl Default for CsrfProtectionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfProtectionMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CsrfProtectionService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfProtectionService {
            service,
            config: self.config.clone(),
        }))
    }
}

/// The actual service that validates Origin/Referer headers
pub struct CsrfProtectionService<S> {
    service: S,
    config: CsrfConfig,
}

impl<S, B> Service<ServiceRequest> for CsrfProtectionService<S>
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
        let method = req.method().clone();

        // Only check state-changing methods
        let is_state_changing = matches!(
            method.as_str(),
            "POST" | "PUT" | "PATCH" | "DELETE"
        );

        if !is_state_changing {
            // GET, HEAD, OPTIONS - safe methods, allow through
            return Box::pin(self.service.call(req));
        }

        // Check for development mode
        let is_production = std::env::var("RUST_ENV")
            .map(|v| v == "production")
            .unwrap_or(false);

        if !is_production && !self.config.enforce_in_dev {
            // Skip validation in development
            return Box::pin(self.service.call(req));
        }

        // Extract origin or referer
        let origin = req
            .headers()
            .get(csrf_origin_header_name())
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let referer = req
            .headers()
            .get(csrf_referer_header_name())
            .and_then(|v| v.to_str().ok())
            .and_then(extract_origin_from_url);

        let request_origin = origin.or(referer);
        let allowed_origins = self.config.allowed_origins.clone();
        let path = req.path().to_string();

        // Allow requests without Origin/Referer (API clients, curl, etc.)
        // SameSite=Strict cookies provide protection for browser requests
        if request_origin.is_none() {
            tracing::debug!(
                path = %path,
                method = %method,
                "CSRF: No Origin/Referer header, allowing request (API client)"
            );
            return Box::pin(self.service.call(req));
        }

        let origin_str = request_origin.unwrap();
        let is_allowed = allowed_origins.iter().any(|allowed| {
            origin_str == *allowed || origin_str.starts_with(allowed)
        });

        if is_allowed {
            tracing::debug!(
                origin = %origin_str,
                path = %path,
                "CSRF: Origin validated"
            );
            Box::pin(self.service.call(req))
        } else {
            tracing::warn!(
                origin = %origin_str,
                path = %path,
                "CSRF: Invalid origin rejected"
            );
            Box::pin(async {
                Err(actix_web::error::ErrorForbidden("Invalid request origin"))
            })
        }
    }
}

/// Extract origin (scheme + host) from a full URL
fn extract_origin_from_url(url: &str) -> Option<String> {
    // Parse URL and extract scheme://host:port
    if let Ok(parsed) = url::Url::parse(url) {
        let scheme = parsed.scheme();
        let host = parsed.host_str()?;
        let port = parsed.port();

        match port {
            Some(p) => Some(format!("{}://{}:{}", scheme, host, p)),
            None => Some(format!("{}://{}", scheme, host)),
        }
    } else {
        None
    }
}

#[cfg(test)]
mod csrf_tests {
    use super::*;
    use actix_web::{test as actix_test, web, App, HttpResponse};

    async fn post_handler() -> HttpResponse {
        HttpResponse::Ok().body("OK")
    }

    async fn get_handler() -> HttpResponse {
        HttpResponse::Ok().body("OK")
    }

    #[actix_web::test]
    async fn test_allows_get_requests() {
        let config = CsrfConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            enforce_in_dev: true,
        };
        let app = actix_test::init_service(
            App::new()
                .wrap(CsrfProtectionMiddleware::with_config(config))
                .route("/", web::get().to(get_handler)),
        )
        .await;

        let req = actix_test::TestRequest::get().uri("/").to_request();
        let resp = actix_test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_allows_valid_origin() {
        std::env::set_var("RUST_ENV", "production");

        let config = CsrfConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            enforce_in_dev: true,
        };
        let app = actix_test::init_service(
            App::new()
                .wrap(CsrfProtectionMiddleware::with_config(config))
                .route("/", web::post().to(post_handler)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/")
            .insert_header(("Origin", "https://example.com"))
            .to_request();
        let resp = actix_test::call_service(&app, req).await;

        std::env::remove_var("RUST_ENV");
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_rejects_invalid_origin() {
        std::env::set_var("RUST_ENV", "production");

        let config = CsrfConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            enforce_in_dev: true,
        };
        let app = actix_test::init_service(
            App::new()
                .wrap(CsrfProtectionMiddleware::with_config(config))
                .route("/", web::post().to(post_handler)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/")
            .insert_header(("Origin", "https://evil.com"))
            .to_request();

        // Use try_call_service since middleware returns an error
        let resp = actix_test::try_call_service(&app, req).await;

        std::env::remove_var("RUST_ENV");
        // The middleware returns an error which results in 403
        assert!(resp.is_err());
    }

    #[actix_web::test]
    async fn test_allows_requests_without_origin_api_clients() {
        std::env::set_var("RUST_ENV", "production");

        let config = CsrfConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            enforce_in_dev: true,
        };
        let app = actix_test::init_service(
            App::new()
                .wrap(CsrfProtectionMiddleware::with_config(config))
                .route("/", web::post().to(post_handler)),
        )
        .await;

        // API client (curl, etc.) without Origin header
        let req = actix_test::TestRequest::post().uri("/").to_request();
        let resp = actix_test::call_service(&app, req).await;

        std::env::remove_var("RUST_ENV");
        // Allowed - SameSite=Strict cookies protect browser requests
        assert!(resp.status().is_success());
    }

    #[test]
    fn test_extract_origin_from_url() {
        assert_eq!(
            extract_origin_from_url("https://example.com/path/to/resource"),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            extract_origin_from_url("http://localhost:3000/api/test"),
            Some("http://localhost:3000".to_string())
        );
        assert_eq!(
            extract_origin_from_url("invalid-url"),
            None
        );
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
