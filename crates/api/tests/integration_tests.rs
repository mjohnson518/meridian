//! Integration tests for Meridian REST API

use actix_web::{test, web, App};
use meridian_api::{routes, AppState};
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Helper to create test app
async fn create_test_app() -> impl actix_web::dev::Service<
    actix_web::dev::ServiceRequest,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let state = Arc::new(AppState::new().await);

    test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await
}

#[actix_web::test]
async fn test_health_check() {
    let app = create_test_app().await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert!(body.get("version").is_some());
}

#[actix_web::test]
async fn test_create_single_currency_basket() {
    let app = create_test_app().await;

    let payload = json!({
        "name": "EUR Basket",
        "currency_code": "EUR",
        "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/single-currency")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201); // Created

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "EUR Basket");
    assert_eq!(body["basket_type"], "single_currency");
    assert!(body.get("id").is_some());
}

#[actix_web::test]
async fn test_create_custom_basket() {
    let app = create_test_app().await;

    let payload = json!({
        "name": "EUR-GBP Basket",
        "components": [
            {
                "currency_code": "EUR",
                "target_weight": "60.0",
                "min_weight": "55.0",
                "max_weight": "65.0",
                "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
            },
            {
                "currency_code": "GBP",
                "target_weight": "40.0",
                "min_weight": "35.0",
                "max_weight": "45.0",
                "chainlink_feed": "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5"
            }
        ],
        "rebalance_strategy": {
            "type": "threshold_based",
            "max_deviation_percent": "3.0"
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/custom")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "EUR-GBP Basket");
    assert_eq!(body["basket_type"], "custom_basket");
    assert_eq!(body["components"].as_array().unwrap().len(), 2);
}

#[actix_web::test]
async fn test_create_imf_sdr_basket() {
    let app = create_test_app().await;

    let mut feeds = HashMap::new();
    feeds.insert("USD".to_string(), "0x0000000000000000000000000000000000000001".to_string());
    feeds.insert("EUR".to_string(), "0xb49f677943BC038e9857d61E7d053CaA2C1734C1".to_string());
    feeds.insert("CNY".to_string(), "0xeF8A4aF35cd47424672E3C590aBD37FBB7A7759a".to_string());
    feeds.insert("JPY".to_string(), "0xBcE206caE7f0ec07b545EddE332A47C2F75bbeb3".to_string());
    feeds.insert("GBP".to_string(), "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5".to_string());

    let payload = json!({
        "name": "IMF SDR Basket",
        "chainlink_feeds": feeds
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/imf-sdr")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "IMF SDR Basket");
    assert_eq!(body["basket_type"], "imf_sdr");
    assert_eq!(body["components"].as_array().unwrap().len(), 5);
}

#[actix_web::test]
async fn test_list_baskets() {
    let app = create_test_app().await;

    // Create a basket first
    let payload = json!({
        "name": "EUR Basket",
        "currency_code": "EUR",
        "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/single-currency")
        .set_json(&payload)
        .to_request();
    test::call_service(&app, req).await;

    // List baskets
    let req = test::TestRequest::get()
        .uri("/api/v1/baskets")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["name"], "EUR Basket");
}

#[actix_web::test]
async fn test_get_basket_by_id() {
    let app = create_test_app().await;

    // Create a basket
    let payload = json!({
        "name": "GBP Basket",
        "currency_code": "GBP",
        "chainlink_feed": "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/single-currency")
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let created: serde_json::Value = test::read_body_json(resp).await;
    let basket_id = created["id"].as_str().unwrap();

    // Get basket by ID
    let uri = format!("/api/v1/baskets/{}", basket_id);
    let req = test::TestRequest::get().uri(&uri).to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "GBP Basket");
    assert_eq!(body["id"], basket_id);
}

#[actix_web::test]
async fn test_get_nonexistent_basket() {
    let app = create_test_app().await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let uri = format!("/api/v1/baskets/{}", fake_id);
    let req = test::TestRequest::get().uri(&uri).to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_invalid_basket_weights() {
    let app = create_test_app().await;

    // Weights sum to 110% (invalid)
    let payload = json!({
        "name": "Invalid Basket",
        "components": [
            {
                "currency_code": "EUR",
                "target_weight": "60.0",
                "min_weight": "55.0",
                "max_weight": "65.0",
                "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
            },
            {
                "currency_code": "GBP",
                "target_weight": "50.0",
                "min_weight": "45.0",
                "max_weight": "55.0",
                "chainlink_feed": "0x5c0Ab2d9b5a7ed9f470386e82BB36A3613cDd4b5"
            }
        ],
        "rebalance_strategy": {
            "type": "none"
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/custom")
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 500); // Internal error (basket validation failed)
}

#[actix_web::test]
async fn test_cors_headers() {
    let app = create_test_app().await;

    let req = test::TestRequest::options()
        .uri("/health")
        .insert_header(("Origin", "http://localhost:3000"))
        .insert_header(("Access-Control-Request-Method", "GET"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // CORS should allow the request
    assert!(resp.status().is_success() || resp.status().as_u16() == 204);
}

