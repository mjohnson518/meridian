//! Middleware components for the Meridian API
//!
//! Includes correlation ID propagation for distributed tracing.

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
